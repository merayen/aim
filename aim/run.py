"""
Runs the aim synthesizer.

See ui.py for running the ui.
"""
import json
import os
import subprocess
from threading import Thread

from aim.nodes import build_node_graph, execution_order, CompilationContext, Context, Node
from aim.numpy_backend import compile_to_numpy


class CompileAndRun(Thread):
	def __init__(self, context: Context):
		super().__init__()
		self.__running = True
		self.context = context

		self.start()


	def stop(self):
		self.__running = False
		self.join()

	def run(self):

		graph, node_ids = build_node_graph(self.context)
		order = execution_order(graph)

		compilation_context = CompilationContext(self.context, graph, node_ids, order)
		code: str = compile_to_numpy(compilation_context)

		# Add code that plays back
		with open(f"{os.path.split(__file__)[0]}{os.path.sep}audio_interface.py") as f:
			code += "\n" + f.read()

		with open(".numpy_program.py", "w") as f:
			f.write(code)

		# Initialize all the listeners that listens to data from each node
		listeners = _init_listeners(order, node_ids)

		# Start a new python interpreter that executes the code
		# TODO merayen maybe support a daemon that receives this code and executes it, and that allows for module loading and de-loading
		with subprocess.Popen(["python3", ".numpy_program.py"], stdout=subprocess.PIPE, universal_newlines=True) as process:
			try:
				while self.__running and process.poll() is None:
					line = process.stdout.readline()
					try:
						node_data = json.loads(line)
						if node_data == {"status": 0}:
							# Sent for every frame processed.
							# This unblocks readline() so that we can yield to the caller.
							pass
						elif node_data.get("debug"):  # Print to stdout
							print(f"DEBUG: Node {node_data['name']} says: {node_data['data']}")
						else:
							listeners[node_data["node_id"]].receive(**node_data["data"])
					except json.decoder.JSONDecodeError:
						print("Invalid JSON data from created program: {}")
						break

			except KeyboardInterrupt:
				pass

			process.kill()


def _init_listeners(order: list[int], node_ids: dict[str, Node]) -> dict:
	import aim.listeners

	result = {}
	for node_id in order:
		if listener := getattr(aim.listeners, f"{node_ids[node_id].__class__.__name__}_listener", None):
			result[node_id] = listener(node_ids[node_id])

	return result
