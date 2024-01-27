"""
Runs the aim synthesizer.

See ui.py for running the ui.
"""
import json
import os
import subprocess
import queue

from aim.nodes import build_node_graph, execution_order, CompilationContext, Context
from aim.numpy_backend import compile_to_numpy


class CompileAndRun:
	def __init__(self, context: Context):
		self.context = context
		self._messages_to_listeners = queue.Queue()
		self._running = True

		graph, self._node_ids = build_node_graph(self.context)
		self._order = execution_order(graph)

		compilation_context = CompilationContext(self.context, graph, self._node_ids, self._order)
		code: str = compile_to_numpy(compilation_context)

		# Add code that plays back
		with open(f"{os.path.split(__file__)[0]}{os.path.sep}audio_interface.py") as f:
			code += "\n" + f.read()

		with open(".numpy_program.py", "w") as f:
			f.write(code)

		self._listeners = self.init_listeners()

		import threading
		self._thread = threading.Thread(target=self.mainloop)
		self._thread.start()

	def stop(self):
		self._running = False
		self._thread.join()

	def mainloop(self):
		# Start a new python interpreter that executes the code
		# TODO merayen maybe support a daemon that receives this code and executes it, and that allows for module loading and de-loading
		with subprocess.Popen(["python3", ".numpy_program.py"], stdout=subprocess.PIPE, universal_newlines=True) as process:
			try:
				while self._running and process.poll() is None:
					# XXX This should probably have some timeout, in case underlaying program halts or goes
					# into an endless loop.
					line = process.stdout.readline()
					try:
						node_data = json.loads(line)
					except json.decoder.JSONDecodeError:
						print(f"Program stdout is not json. Please use debug_print() instead.")
					else:
						if node_data == {"status": 0}:
							pass
						elif node_data.get("debug"):  # Print to stdout
							print(f"DEBUG: Node {node_data['name']} says: {node_data['data']}")
						else:
							self._messages_to_listeners.put(node_data)

			except KeyboardInterrupt:
				pass

			process.kill()

	def init_listeners(self) -> dict:
		import aim.listeners

		result = {}
		for node_id in self._order:
			if listener := getattr(aim.listeners, f"{self._node_ids[node_id].__class__.__name__}_listener", None):
				result[node_id] = listener(self._node_ids[node_id])

		return result

	def mainloop_mainthread(self):
		while 1:  # Until ctrl-c
			message = self._messages_to_listeners.get()
			self._listeners[message["node_id"]].receive(**message["data"])
