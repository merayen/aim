from dataclasses import dataclass
from aim.nodes import (
	Node, create_variable, Outlet, Context, build_node_graph, execution_order, DataType,
	sine, out,
)
from typing import Any


@dataclass
class NodeContext:
	frame_count: int
	sample_rate: int


def numpy_sine(node_context: NodeContext, node: sine, init_code: list[str], process_code: list[str]) -> None:
	clock = create_variable()
	clock_array = create_variable()

	# Clock for each voice
	init_code.append(f"{clock} = defaultdict(lambda: 0.0)")

	if isinstance(node.frequency, (int, float)):
		# Fixed value input. We create our own, single voice. Simplified logic
		voice_id_variable = create_variable()
		init_code.extend(
			[
				f"{voice_id_variable} = create_voice()",
				f"{node.output._variable} = Signal(data={{{voice_id_variable}: None}}, channel_map={{{voice_id_variable}: 0}})",
			]
		)
		process_code.append(
			f"{clock_array} = {clock}[{voice_id_variable}] + "
			f"np.cumsum(np.ones({node_context.frame_count}, dtype='float32') * "
			f"({node.frequency} * np.pi * 2 / {node_context.sample_rate}))"
		)
		process_code.append(f"{node.output._variable}.data[{voice_id_variable}] = np.sin({clock_array})")
		process_code.append(f"{clock}[{voice_id_variable}] = {clock_array}[-1]")
	elif isinstance(node.frequency, Outlet):
		# These ones may contain multiple voices.
		# We only forward the voices.
		if node.frequency.datatype == DataType.SIGNAL:
			init_code.append("{node.output._variable = Signal()}")
			process_code.extend(
				[
					f"for voice_id, data in {node.frequency._variable}.data.items():",
					f"	{clock_array} = {clock} + "
					f"	np.cumsum(np.ones({node_context.frame_count}, dtype='float32') * "
					f"	(data * np.pi * 2 / {node_context.sample_rate}))",
					f"	{clock}[voice_id] = {clock_array}[-1] % (np.pi * 2)",
				]
			)
		else:
			unsupported(node)
	else:
		unsupported(node)


def _numpy_math(
	node_context: NodeContext,
	node: Node,
	init_code: list[str],
	process_code: list[str],
) -> None:
	from aim.nodes import add, sub, mul, div
	op = {
		add: "+",
		sub: "-",
		mul: "*",
		div: "/",
	}[node.__class__]

	if isinstance(node.in0, (int, float)) and isinstance(node.in1, (int, float)):
		# Number never changes, sum it only once
		init_code.append(
				f"{node.output._variable} = Signal(data={{create_voice():"
				f"np.zeros({node_context.frame_count}) + {eval('node.in0'+op+'node.in1')}}})"
		)
	elif isinstance(node.in0, Outlet) and isinstance(node.in1, Outlet):
		process_code.append(f"{node.output._variable} = Signal()")
		if node.in0.datatype == DataType.SIGNAL and node.in1.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in set({node.in0._variable}.data.keys()).intersection({node.in1._variable}.data.keys()):",
					f"	{node.output._variable}.data[voice_id] = {node.in0._variable}.data[voice_id] {op} {node.in1._variable}.data[voice_id]",
				]
			)
		else:
			unsupported(node)
	elif isinstance(node.in0, (int, float)) and isinstance(node.in1, Outlet):
		process_code.append(f"{node.output._variable} = Signal()")
		if node.in1.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in {node.in1._variable}.data:",
					f"	{node.output._variable}.data[voice_id] = {node.in0} {op} {node.in1._variable}.data[voice_id]",
				]
			)
		else:
			unsupported(node)
	elif isinstance(node.in0, Outlet) and isinstance(node.in1, (int, float)):
		process_code.append(f"{node.output._variable} = Signal()")
		if node.in0.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in {node.in0._variable}.data:",
					f"	{node.output._variable}.data[voice_id] = {node.in0._variable}.data[voice_id] {op} {node.in1}",
				]
			)
		else:
			unsupported(node)
	else:
		unsupported(node)


numpy_add = _numpy_math
numpy_sub = _numpy_math
numpy_mul = _numpy_math
numpy_div = _numpy_math


def numpy_poly(node_context: NodeContext, node: out, init_code: list[str], process_code: list[str]) -> None:
	pass


def numpy_out(node_context: NodeContext, node: out, init_code: list[str], process_code: list[str]) -> None:
	assert node.name
	assert "'" not in node.name

	if isinstance(node.input, (int, float)):
		voice_id_variable = create_variable()
		init_code.append(f"{voice_id_variable} = create_voice()")
		process_code.extend(
			[
				f"output['{node.name}'] = Signal(",
				f"	data=np.zeros({node_context.frame_count}, dtype='float32') + {node.input}",
				")",
			]
		)

	elif isinstance(node.input, Outlet):
		process_code.extend(
			[
				f"output['{node.name}'] = {node.input._variable}",
			]
		)
	else:
		raise NotImplementedError("Support other types of input")  # TODO merayen support other types of input for out.input


def compile_to_numpy(context: Context, frame_count: int = 512, sample_rate: int = 48000) -> str:
	# Start backwards and create dependency graph
	graph, node_ids = build_node_graph(context)

	order = execution_order(graph)

	init_code = []
	process_code = []

	init_code = [
		"import numpy as np",
		"from collections import defaultdict",
		"from dataclasses import dataclass, field",
		"process_counter = -1",
		"voice_identifier = -1",
		"def create_voice():",
		"	global voice_identifier",
		"	voice_identifier += 1",
		"	return voice_identifier",
		"@dataclass",
		"class Signal:",
		"	data: dict = field(default_factory=lambda:{})",  # TODO merayen rename data to voices?
		"	channel_map: dict = field(default_factory=lambda:{})",
	]

	process_code = [
		"global process_counter",
		"process_counter += 1",
		#f"if not (process_counter % {frame_count}):"
		#f"\tprint(round(process_counter*{frame_count} / {sample_rate}), 'seconds')",
		"output = {}",
	]

	node_context = NodeContext(
		frame_count=frame_count,
		sample_rate=sample_rate,
	)

	for node_id in order:
		node = node_ids[node_id]
		assert isinstance(node, Node), (type(node), Node)

		func = globals().get(f"numpy_{node.__class__.__name__}")

		if not func:
			raise NotImplementedError(f"Node {node.__class__} is not supported in the numpy_backend")

		func(node_context, node, init_code, process_code)

	# Return back data
	process_code.append(
		"return output"
	)

	code = "\n" + "\n".join(init_code)
	code += f"\nsample_rate = {sample_rate}"
	code += f"\nframe_count = {frame_count}"
	code += "\ndef numpy_process():\n" + "\n".join(f"\t{x}" for x in process_code)

	return code


def unsupported(node: Node):
	input_text = "\n".join(f"\t{k}: {v}" for k,v in node._inlets.items())
	raise Exception(f"Node {node.__class__.__name__} does not support inputs:\n{input_text}")


def test_sine_node() -> None:
	from aim.nodes import load, OutNode
	for x in (
		"out(sine())",
		"out(sine().output)",
	):
		context: Context = load(x)
		assert len(context.out_nodes) == 1
		assert isinstance(context.out_nodes[0], OutNode)
		outlet = context.out_nodes[0].input

		assert isinstance(outlet, Outlet)
		assert outlet.node.frequency == 440

		from aim.numpy_backend import compile_to_numpy
		code: str = compile_to_numpy(context, sample_rate=10, frame_count=10)

		code += "\nresult = numpy_process()"

		a = {}
		exec(code, a)


def test_math_nodes() -> None:
	import numpy as np

	assert np.all(run_code("out(add(0,1) + 5 + add(2,0) / add(4,0) * 2)")["unnamed_0"] == 1 + 5 + 2 / 4 * 2)

	# TODO merayen verify output of all

def test_sub_node() -> None:
	import numpy as np
	r = run_code("out(sub(20,5.0))")["unnamed_0"].data
	breakpoint()  # TODO merayen remove
	assert np.all(r == 15)
	run_code("out(sine(440) + 5)")
	run_code("out(sub(in0=5, in1=sine(440)))")
	run_code("out(sub(in0=sine(440), in1=5))")
	run_code("out(sine(440) + sine(880))")

	# TODO merayen verify output of all

def test_out_node() -> None:
	import numpy as np

	assert np.all(run_code("out(5*5)")["unnamed_0"].data == 25)


def run_code(code: str, frame_count=10, sample_rate=48000) -> Any:
	from aim.nodes import load

	context: Context = load(code)
	code: str = compile_to_numpy(context, frame_count=10, sample_rate=sample_rate)

	code += "\nresult = numpy_process()"

	a = {}
	exec(code, a)

	return a["result"]


if __name__ == '__main__':
	for x in dir():
		if x.startswith("test_"):
			exec(f"{x}()")
