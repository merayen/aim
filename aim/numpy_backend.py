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


def _oscillator_clock_default_voice(
	node_context: NodeContext,
	node: Node,
	init_code: list[str],
	process_code: list[str],
	func: str,
):
	"""
	Common oscillator clock for all of the oscillators
	"""
	clock = create_variable()
	clock_array = create_variable()
	func %= {'clock_array': clock_array}

	init_code.append(f"{clock} = 0.0")
	init_code.append(f"{node.output._variable} = Signal()")
	process_code.append(f"global {clock}")
	process_code.append(
		f"{clock_array} = {clock} + "
		f"np.cumsum(_ONES * "
		f"({node.frequency} / {node_context.sample_rate}))"
	)
	process_code.append(f"{node.output._variable}.data[0] = {func}")
	process_code.append(f"{clock} = {clock_array}[-1]")


def _oscillator_clock_multi_voice(
	node_context: NodeContext,
	node: Node,
	init_code: list[str],
	process_code: list[str],
	func: str,
):
	clock = create_variable()
	clock_array = create_variable()
	func %= {'clock_array': clock_array}

	init_code.append(f"{clock} = defaultdict(lambda: 0.0)")
	init_code.append(f"{node.output._variable} = Signal()")

	# Note that we only respect voices on the frequency-input
	# If e.g another input port has other voices, we just ignore them. This may or may not be wanted.
	# TODO merayen remove voices that disappears on the input
	process_code.extend(
		[
			f"for voice_id, data in {node.frequency._variable}.data.items():",
			f"	{clock_array} = {clock}[voice_id] +"
			f"	np.cumsum(_ONES * (data / {node_context.sample_rate}))",
			f"	{clock}[voice_id] = {clock_array}[-1] % 1",
			f"	{node.output._variable}.data[voice_id] = {func}",
		]
	)


def numpy_sine(node_context: NodeContext, node: sine, init_code: list[str], process_code: list[str]) -> None:
	if isinstance(node.frequency, (int, float)):
		_oscillator_clock_default_voice(
			node_context,
			node,
			init_code,
			process_code,
			"np.sin(%(clock_array)s * np.pi * 2)",
		)
	elif isinstance(node.frequency, Outlet):
		if node.frequency.datatype == DataType.SIGNAL:
			_oscillator_clock_multi_voice(
				node_context,
				node,
				init_code,
				process_code,
				"np.sin(%(clock_array)s * np.pi * 2)",
			)
		else:
			unsupported(node)
	else:
		unsupported(node)


def numpy_square(
	node_context: NodeContext,
	node: sine,
	init_code: list[str],
	process_code: list[str],
) -> None:
	if isinstance(node.frequency, (int, float)):
		# When frequency input is a literal value, we always use the default voice 0
		if node.duty is None:
			_oscillator_clock_default_voice(
				node_context,
				node,
				init_code,
				process_code,
				f"(%(clock_array)s %% 1 >= 0.5).astype('float32')",
			)
		if isinstance(node.duty, (int, float)):
			_oscillator_clock_default_voice(
				node_context,
				node,
				init_code,
				process_code,
				f"(%(clock_array)s %% 1 >= {node.duty}).astype('float32')",
			)
		elif isinstance(node.duty, Outlet):
			if node.duty.datatype == DataType.SIGNAL:
				_oscillator_clock_default_voice(
					node_context,
					node,
					init_code,
					process_code,
					f"(%(clock_array)s %% 1 > {node.duty._variable.data.get(0, _ONES)}).astype('float32')",
				)
			else:
				unsupported(node)
	elif isinstance(node.frequency, Outlet):
		if node.duty is None:
			_oscillator_clock_multi_voice(
				node_context,
				node,
				init_code,
				process_code,
				f"(%(clock_array)s %% 1 > 0.5).astype('float32')",
			)
		elif isinstance(node.duty, (int, float)):
			_oscillator_clock_multi_voice(
				node_context,
				node,
				init_code,
				process_code,
				f"(%(clock_array)s %% 1 > {node.duty}).astype('float32')",
			)
		elif isinstance(node.duty, Outlet):
			if node.duty.datatype == DataType.SIGNAL:
				_oscillator_clock_multi_voice(
					node_context,
					node,
					init_code,
					process_code,
					f"(%(clock_array)s %% 1 > {node.duty._variable.data.get(voice_id, _ONES)}).astype('float32')",
				)
			else:
				unsupported(node)
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
				f"{node.output._variable} = Signal(data={{0:"
				f"np.zeros({node_context.frame_count}) + {eval('node.in0'+op+'node.in1')}}})"
		)
	elif isinstance(node.in0, Outlet) and isinstance(node.in1, Outlet):
		# TODO merayen remove voices that disappears on the input
		process_code.append(f"{node.output._variable} = Signal()")
		if node.in0.datatype == DataType.SIGNAL and node.in1.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in set({node.in0._variable}.data.keys()).union({node.in1._variable}.data.keys()):",
					f"	{node.output._variable}.data[voice_id] = {node.in0._variable}.data.get(voice_id, _SILENCE) {op} {node.in1._variable}.data.get(voice_id, _SILENCE)",
				]
			)
		else:
			unsupported(node)
	elif isinstance(node.in0, (int, float)) and isinstance(node.in1, Outlet):
		# TODO merayen remove voices that disappears on the input
		process_code.append(f"{node.output._variable} = Signal()")
		if node.in1.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in {node.in1._variable}.data:",
					f"	{node.output._variable}.data[voice_id] = {node.in0} {op} {node.in1._variable}.data.get(voice_id, _SILENCE)",
				]
			)
		else:
			unsupported(node)
	elif isinstance(node.in0, Outlet) and isinstance(node.in1, (int, float)):
		# TODO merayen remove voices that disappears on the input
		process_code.append(f"{node.output._variable} = Signal()")
		if node.in0.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in {node.in0._variable}.data:",
					f"	{node.output._variable}.data[voice_id] = {node.in0._variable}.data.get(voice_id, _SILENCE) {op} {node.in1}",
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
		f"_SILENCE = np.zeros({frame_count}, dtype='float32')",
		f"_ONES = np.ones({frame_count}, dtype='float32')",
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

		init_code.append(f"# {node.__class__.__name__}")
		process_code.append(f"# {node.__class__.__name__}")
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

	assert np.all(run_code("out(add(0,1) + 5 + add(2,0) / add(4,0) * 2)")["unnamed_0"].data[0] == 1 + 5 + 2 / 4 * 2)


def test_sub_node() -> None:
	import numpy as np
	r = run_code("out(sub(20,5.0))")["unnamed_0"].data[0]
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
