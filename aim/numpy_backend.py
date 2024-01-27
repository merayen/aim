from dataclasses import dataclass
from aim.nodes import (
	Node, create_variable, Outlet, Context, DataType,
	sine, out, CompilationContext,
)
from typing import Any


@dataclass
class NodeContext:
	frame_count: int
	sample_rate: int


def _oscillator_clock(
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

	if isinstance(node.frequency, (int, float)):
		func %= {'clock_array': clock_array, "voice_id": 0}
		init_code.append(f"{clock} = 0.0")
		init_code.append(f"{node.output._variable} = Signal()")
		process_code.append(f"global {clock}")
		process_code.append(
			f"{clock_array} = {clock} + "
			f"np.cumsum(_ONES * ({node.frequency} / {node_context.sample_rate}))"
		)
		process_code.append(f"{node.output._variable}.voices[0] = {func}")
		process_code.append(f"{clock} = {clock_array}[-1]")

	elif isinstance(node.frequency, Outlet):
		func %= {'clock_array': clock_array, "voice_id": "voice_id"}
		init_code.append(f"{clock} = defaultdict(lambda: 0.0)")
		init_code.append(f"{node.output._variable} = Signal()")

		# Note that we only respect voices on the frequency-input
		# If e.g another input port has other voices, we just ignore them. This may or may not be wanted.
		# TODO merayen remove voices that disappears on the input
		process_code.extend(
			[
				f"for voice_id, voice in {node.frequency._variable}.voices.items():",
				f"	{clock_array} = {clock}[voice_id] +"
				f"	np.cumsum(_ONES * (voice / {node_context.sample_rate}))",
				f"	{clock}[voice_id] = {clock_array}[-1] % 1",
				f"	{node.output._variable}.voices[voice_id] = {func}",
			]
		)

		# Remove voices that has disappeared
		process_code.extend(
			[
				f"for voice_id in set({node.output._variable}.voices) - set({node.frequency._variable}.voices):",
				f"	{clock}.pop(voice_id)",
				f"	{node.output._variable}.voices.pop(voice_id)",
			]
		)
	else:
		unsupported(node)


def numpy_sine(
	node_context: NodeContext,
	node: sine,
	init_code: list[str],
	process_code: list[str],
) -> None:
	_oscillator_clock(
		node_context,
		node,
		init_code,
		process_code,
		"np.sin(%(clock_array)s * np.pi * 2)",
	)


def numpy_square(
	node_context: NodeContext,
	node: sine,
	init_code: list[str],
	process_code: list[str],
) -> None:
	if node.duty is None:
		_oscillator_clock(
			node_context,
			node,
			init_code,
			process_code,
			"(%(clock_array)s %% 1 >= 0.5).astype('float32') * 2 - 1",
		)
	elif isinstance(node.duty, (int, float)):
		_oscillator_clock(
			node_context,
			node,
			init_code,
			process_code,
			f"(%(clock_array)s %% 1 >= {node.duty}).astype('float32') * 2 - 1",
		)
	elif isinstance(node.duty, Outlet):
		if node.duty.datatype == DataType.SIGNAL:
			_oscillator_clock(
				node_context,
				node,
				init_code,
				process_code,
				f"(%(clock_array)s %% 1 > {node.duty._variable}.voices.get(%(voice_id)s, _ONES)).astype('float32') * 2 - 1",
			)
		else:
			unsupported(node)
	else:
		unsupported(node)


def numpy_saw(
	node_context: NodeContext,
	node: sine,
	init_code: list[str],
	process_code: list[str],
) -> None:
	_oscillator_clock(
		node_context,
		node,
		init_code,
		process_code,
		"(%(clock_array)s %% 1) * 2.0 - 1.0",
	)


def numpy_noise(
	node_context: NodeContext,
	node: sine,
	init_code: list[str],
	process_code: list[str],
) -> None:
	init_code.append(f"{node.output._variable} = Signal()")

	if isinstance(node.voices, Outlet):
		process_code.extend(
			[
				"for voice_id in {node.voices._variable}.voices:",
					f"	{node.output._variable}.voices[0] = random.random(({node_context.frame_count},), dtype='float32') * 2 - 1",
			]
		)

		# Remove voices that are extinct
		process_code.extend(
			[
				f"for voice_id in set({node.output._variable}.voices) - set({node.voices._variable}.voices)",
				f"	{node.output._variable}.voices.pop(voice_id)",
			]
		)
	else:
		process_code.append(
			f"{node.output._variable}.voices[0] = random.random(({node_context.frame_count},), dtype='float32') * 2 - 1"
		)


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
				f"{node.output._variable} = Signal(voices={{0:"
				f"np.zeros({node_context.frame_count}) + {eval('node.in0'+op+'node.in1')}}})"
		)
	elif isinstance(node.in0, Outlet) and isinstance(node.in1, Outlet):
		# TODO merayen remove voices that disappears on the input
		init_code.append(f"{node.output._variable} = Signal()")
		if node.in0.datatype == DataType.SIGNAL and node.in1.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in set({node.in0._variable}.voices.keys()).union({node.in1._variable}.voices.keys()):",
					f"	{node.output._variable}.voices[voice_id] = {node.in0._variable}.voices.get(voice_id, _SILENCE) {op} {node.in1._variable}.voices.get(voice_id, _SILENCE)",
				]
			)
		else:
			unsupported(node)
	elif isinstance(node.in0, (int, float)) and isinstance(node.in1, Outlet):
		# TODO merayen remove voices that disappears on the input
		init_code.append(f"{node.output._variable} = Signal()")
		if node.in1.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in {node.in1._variable}.voices:",
					f"	{node.output._variable}.voices[voice_id] = {node.in0} {op} {node.in1._variable}.voices.get(voice_id, _SILENCE)",
				]
			)
		else:
			unsupported(node)
	elif isinstance(node.in0, Outlet) and isinstance(node.in1, (int, float)):
		# TODO merayen remove voices that disappears on the input
		init_code.append(f"{node.output._variable} = Signal()")
		if node.in0.datatype == DataType.SIGNAL:
			process_code.extend(
				[
					f"for voice_id in {node.in0._variable}.voices:",
					f"	{node.output._variable}.voices[voice_id] = {node.in0._variable}.voices.get(voice_id, _SILENCE) {op} {node.in1}",
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


def numpy_mix(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	in0_voices = create_variable()
	in1_voices = create_variable()

	# TODO merayen how should we mix channel_map if in0 and in1 has different maps
	init_code.append(f"{node.output._variable} = Signal()")

	if isinstance(node.in0, Outlet):
		process_code.append(f"{in0_voices} = set({node.in0._variable}.voices)")
	if isinstance(node.in1, Outlet):
		process_code.append(f"{in1_voices} = set({node.in1._variable}.voices)")

	if isinstance(node.in0, Outlet) and isinstance(node.in1, Outlet):
		process_code.append(f"for voice_id in {in0_voices}.intersection({in1_voices}):")
	elif isinstance(node.in0, Outlet):
		process_code.append(f"for voice_id in {in0_voices}:")
	elif isinstance(node.in1, Outlet):
		process_code.append(f"for voice_id in {in1_voices}:")
	else:
		process_code.append("for voice_id in [0]:")
	process_code.append(f"	{node.output._variable}.voices[voice_id] = (")

	if isinstance(node.fac, (int, float)):
		fac = (max(min(node.fac, 1), -1) + 1) / 2
	elif isinstance(node.fac, Outlet):
		fac = f"{node.fac._variable}.voices[voice_id]"
	else:
		unsupported(node)

	if isinstance(node.in0, Outlet):
		process_code.append(f"		{node.in0._variable}.voices[voice_id] * (1-{fac}) +")
	elif isinstance(node.in0, (int, float)):
		process_code.append(f"		{node.in0} * (1-{fac}) +")
	else:
		unsupported(node)

	if isinstance(node.in1, Outlet):
		process_code.append(f"		{node.in1._variable}.voices[voice_id] * {fac}")
	elif isinstance(node.in0, (int, float)):
		process_code.append(f"		{node.in1} * {fac}")
	else:
		unsupported(node)

	process_code.append("	)")

	# Remove voices that are extinct
	if isinstance(node.in0, Outlet) and isinstance(node.in1, Outlet):
		process_code.append(f"for voice_id in set({node.output._variable}.voices) - {in0_voices} - {in1_voices}:")
	elif isinstance(node.in0, Outlet):
		process_code.append(f"for voice_id in set({node.output._variable}.voices) - {in0_voices}:")
	elif isinstance(node.in1, Outlet):
		process_code.append(f"for voice_id in set({node.output._variable}.voices) - {in1_voices}:")
	else:
		# lol code
		process_code.append("if 0:")
	process_code.append(f"	{node.output._variable}.voices.pop(voice_id)")


def numpy_slew(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	raise NotImplementedError("")  # TODO merayen implement


def numpy_trigger(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	current_value = create_variable()

	init_code.append(f"{node.output._variable} = Signal()")
	init_code.append(f"{current_value} = defaultdict(lambda: 0.0)")

	method = introduce_method(
		init_code,
		[
			"import numba",
			"@numba.njit",
			"def METHOD_NAME(current, input_array, output_array, start_array, stop_array):",
			"	assert len(input_array) == len(output_array) == len(start_array) == len(stop_array)",
			"",
			"	for i in range(len(input_array)):",
			"		if input_array[i] >= start_array[i]:",
			"			current = 1.0",
			"		elif input_array[i] < stop_array[i]:",
			"			current = 0.0",
			"		output_array[i] = current",
			"	return current",
		]
	)

	if isinstance(node.value, Outlet):

		if isinstance(node.start, (int, float)):
			start = create_variable()
			init_code.append(f"{start} = np.zeros({node_context.frame_count}) + {node.start}")

		if isinstance(node.stop, (int, float)):
			stop = create_variable()
			init_code.append(f"{stop} = np.zeros({node_context.frame_count}) + {node.stop}")

		process_code.append(f"for voice_id, voice in {node.value._variable}.voices.items():")
		process_code.append(f"	if voice_id not in {node.output._variable}.voices:")
		process_code.append(f"		{node.output._variable}.voices[voice_id] = np.zeros({node_context.frame_count})")
		process_code.append(f"	{current_value}[voice_id] = {method}(")
		process_code.append(f"		{current_value}[voice_id],")
		process_code.append("		voice,")
		process_code.append(f"		{node.output._variable}.voices[voice_id],")

		if isinstance(node.start, (int, float)):
			process_code.append(f"		{start},")
		elif isinstance(node.start, Outlet):
			process_code.append(f"		{node.start._variable}.voices[voice_id],")
		else:
			unsupported(node)

		if isinstance(node.stop, (int, float)):
			process_code.append(f"		{stop},")
		elif isinstance(node.stop, Outlet):
			process_code.append(f"		{node.stop._variable}.voices[voice_id],")
		else:
			unsupported(node)

		process_code.append(")")

		# Remove voices that has disappeared
		process_code.extend(
			[
				f"for voice_id in set({node.output._variable}.voices) - set({node.value._variable}.voices):",
				f"	{node.output._variable}.voices.pop(voice_id)",
			]
		)
	else:
		unsupported(node)


def numpy_clip(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	init_code.append(f"{node.output._variable} = Signal()")

	if isinstance(node.value, Outlet):
		if isinstance(node.minimum, (int, float)) and isinstance(node.maximum, (int, float)):
			process_code.append(f"for voice_id, voice in {node.value._variable}.voices.items():")
			process_code.append(f"	{node.output._variable}.voices[voice_id] = np.clip(voice, {node.minimum}, {node.maximum})")
		else:
			unsupported(node)
	else:
		unsupported(node)

	# Remove voices that has disappeared
	process_code.extend(
		[
			f"for voice_id in set({node.output._variable}.voices) - set({node.value._variable}.voices):",
			f"	{node.output._variable}.voices.pop(voice_id)",
		]
	)


def numpy_poly(node_context: NodeContext, node: out, init_code: list[str], process_code: list[str]) -> None:
	pass


def numpy_audiofile(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	# TODO merayen implement high quality resampling when node.speed is else than "1"

	init_code.append(f"{node.output._variable} = Signal()")

	if not node.channel_paths:
		return  # No audio to play back. Process nothing

	audio_sample_count = create_variable()
	audio_data = create_variable()

	if node.voices is None:
		# No voicing. Simple implementation with single playback
		playback_position = create_variable()

		init_code.append(f"{playback_position} = 0")
		init_code.append(f"{audio_data} = {{}}")

		# YOLO-load all channel data into memory
		# XXX Implement smart-buffering of audio files in the future
		for i, (channel_index, channel_path) in enumerate(node.channel_paths.items()):
			assert "'" not in channel_path


			init_code.append(f"{audio_data}[{channel_index}] = np.fromfile('{channel_path}', dtype='float32')")

			if not i:
				init_code.append(f"{audio_sample_count} = len({audio_data}[{channel_index}])")
			else:
				init_code.append(f"assert {audio_sample_count} == len({audio_data}[{channel_index}])")

		process_code.append(f"global {playback_position}")

		# If we are done playing, clear all output voices
		process_code.append(f"if {playback_position}+1 >= {audio_sample_count}:")
		process_code.append(f"	{node.output._variable}.voices.clear()")
		process_code.append("else:")
		process_code.append("	pass")

		# Output each channel to its own voice
		# XXX Maybe we could do this more effective using numpy directly...?
		for channel_index in node.channel_paths:
			process_code.append(
				f"	{node.output._variable}.voices[{channel_index}] = "
				f"{audio_data}[{channel_index}][{playback_position}:{playback_position}+{node_context.frame_count}]"
			)

			# When at the end of the audio signal, pad with silence
			process_code.extend(
				[
					f"	if len({node.output._variable}.voices[{channel_index}]) != {node_context.frame_count}:",
					f"		{node.output._variable}.voices[{channel_index}] = "
					f"np.pad({node.output._variable}.voices[{channel_index}], pad_width=(0,{node_context.frame_count}-len({node.output._variable}.voices[{channel_index}])))",
				]
			)

		# Increase the playback position, if not at the end already
		process_code.append(f"	{playback_position} += {node_context.frame_count}")
	else:
		# TODO merayen how to support multiple voices of this node that already creates voices? stack them?
		unsupported(node)


def numpy_oscilloscope(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	# TODO merayen Implement triggering. Probably a trigger that starts individually per voice

	buffer = create_variable()
	samples_filled = create_variable()
	is_triggering = create_variable()
	trigger_at = create_variable()
	last_update = create_variable()  # Last time we updated the oscilloscope view
	pl = create_variable()

	init_code.append(f"{buffer} = {{}}")
	init_code.append(f"{samples_filled} = 0")
	init_code.append(f"{is_triggering} = {{}}")
	init_code.append(f"{trigger_at} = {{}}")  # Where in the current buffer to start triggering
	init_code.append(f"{last_update} = 0")
	init_code.append(f"import pylab as {pl}")
	init_code.append(f"{pl}.ion()")

	process_code.append(f"global {samples_filled}")
	process_code.append(f"global {last_update}")
	process_code.append(f"global {is_triggering}")
	process_code.append(f"global {trigger_at}")

	if isinstance(node.value, Outlet):
		# We just forward the whole Outlet, as we are only forwarding the data
		init_code.append(f"{node.output._variable} = {node.value._variable}")

		if isinstance(node.time_div, (int, float)):
			# Look for new voices and create buffers for them
			buffer_size = int(node_context.sample_rate * max(1E-4, min(1, node.time_div)))
			process_code.append(f"for voice_id in set({node.value._variable}.voices) - set({buffer}):")
			process_code.append(f"	{buffer}[voice_id] = np.zeros({buffer_size})")
			process_code.append(f"	{trigger_at}[voice_id] = -1")
			process_code.append(f"	{is_triggering}[voice_id] = False")
		else:
			unsupported(node)

		# Remove dead voices
		process_code.append(f"for voice_id in set({buffer}) - set({node.value._variable}.voices):")
		process_code.append(f"	{buffer}.pop(voice_id)")

		# Decide if we are to start triggering.
		# Scans each voice's buffer for trigger point.
		if isinstance(node.trigger, (int, float)):
			# TODO merayen iterate each voice to find trigger points
			process_code.append(f"for voice_id, voice in {node.value._variable}.voices.items():")
			process_code.append(f"	if {is_triggering}: continue")
			process_code.append(f"	{trigger_at}[voice_id] = np.argmax(voice >= {node.trigger})")
			# A very special case as argmax does not return -1 when not found
			process_code.append(f"	if {trigger_at}[voice_id] == 0 and voice[0] < {node.trigger}:")
			# Nope, didn't find trigger point after all
			process_code.append(f"		{trigger_at}[voice_id] = -1")
		else:
			unsupported(node)

		if isinstance(node.time_div, (int, float)):
			# Add samples to the buffer until it is full
			process_code.append(f"if {samples_filled} < {buffer_size} and {node.value._variable}.voices:")
			process_code.append(f"	for voice_id, voice in {node.value._variable}.voices.items():")
			# TODO merayen implement listening for the trigger_at here
			process_code.append(
				f"		{buffer}[voice_id]["
				f"{samples_filled}:{samples_filled} + min({node_context.frame_count}, {buffer_size} - {samples_filled})] = "
				f"voice[:min({node_context.frame_count}, {buffer_size} - {samples_filled})]"
			)

			# Update the counter for samples sampled
			process_code.append(f"	{samples_filled} += {node_context.frame_count}")
		else:
			unsupported(node)

		# Check if we have enough samples and if it is time to update oscilloscope view
		if isinstance(node.time_div, (int, float)):
			process_code.append(f"if {samples_filled} >= {buffer_size} and {last_update} + 0.05 < time.time():")
			process_code.append(f"	{last_update} = time.time()")
			process_code.append(f"	{is_triggering} = False")
			process_code.append(
				"	" + emit_data(node, f"	{{'plot_data': {{voice_id: voices.tolist() for voice_id, voices in {buffer}.items()}} }}")
			)
			process_code.append(f"	{samples_filled} = 0")
		else:
			unsupported(node)

	else:
		unsupported(node)


def numpy_out(node_context: NodeContext, node: out, init_code: list[str], process_code: list[str]) -> None:
	assert node.name
	assert "'" not in node.name

	if isinstance(node.input, (int, float)):
		voice_id_variable = create_variable()
		init_code.append(f"{voice_id_variable} = create_voice()")
		process_code.extend(
			[
				f"output['{node.name}'] = Signal(",
				f"	voices=np.zeros({node_context.frame_count}, dtype='float32') + {node.input}",
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


def compile_to_numpy(
	compilation_context: CompilationContext,
	frame_count: int = 512,
	sample_rate: int = 48000,
) -> str:
	assert isinstance(compilation_context, CompilationContext)
	# Start backwards and create dependency graph
	init_code = []
	process_code = []

	init_code = [
		"import numpy as np",
		"import json, sys, time",
		"from collections import defaultdict",
		"from dataclasses import dataclass, field",
		f"_SILENCE = np.zeros({frame_count}, dtype='float32')",
		f"_ONES = np.ones({frame_count}, dtype='float32')",
		"process_counter = -1",
		"voice_identifier = 0",  # Note: All dynamically created voices starts at 1. 0 is the special default voice
		"def create_voice():",
		"	global voice_identifier",
		"	voice_identifier += 1",
		"	return voice_identifier",
		"@dataclass",
		"class Signal:",
		"	voices: dict = field(default_factory=lambda:{})",
		"	channel_map: dict = field(default_factory=lambda:{})",
		"random = np.random.default_rng()",
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

	for node_id in compilation_context.order:
		node = compilation_context.node_ids[node_id]
		assert isinstance(node, Node), (type(node), Node)

		func = globals().get(f"numpy_{node.__class__.__name__}")

		if not func:
			raise NotImplementedError(f"Node {node.__class__} is not supported in the numpy_backend")

		init_code.append(f"# {node.__class__.__name__}")
		process_code.append(f"# {node.__class__.__name__}")
		func(node_context, node, init_code, process_code)

	# Emit statistic data to stdout and return data
	process_code.extend(
		[
			"""print('{"status": 0}')""",  # Notify that we have processed a buffer
			"sys.stdout.flush()",
			"return output",
		]
	)

	code = "\n".join(init_code)
	code += f"\nsample_rate = {sample_rate}"
	code += f"\nframe_count = {frame_count}"
	code += "\ndef numpy_process():\n" + "\n".join(f"\t{x}" for x in process_code)

	return code


def introduce_method(init_code: list[str], lines: list[str]):
	if lines in introduce_method.lines.values():
		return next(k for k,v in introduce_method.items() if v == lines) # Already defined

	method_name = create_variable()
	
	init_code.extend([x.replace("METHOD_NAME", method_name) for x in lines])

	introduce_method.lines[method_name] = lines

	return method_name
introduce_method.lines = {}


def emit_data(node: Node, code: str) -> str:
	"""
	Emit data from node to parent process

	E.g, the plot data for an oscilloscope.
	"""
	return f"print(json.dumps({{'node_id': {id(node)}, 'name': '{node.__class__.__name__}', 'data': {code}}}))"


def debug_print(node: Node, process_code: list[str], code: str):
	"""
	Output data straight to stdout

	Only meant for when developing nodes, not normal usage.
	"""
	# TODO merayen probably only do this when "aim --debug"
	process_code.append(f"print(json.dumps({{'node': '{id(node)}', 'name': '{node.__class__.__name__}', 'debug': True, 'data': {code}}}))")


def unsupported(node: Node):
	input_text = "\n".join(f"\t{k}: {v}" for k,v in node._inlets.items())
	raise Exception(f"Node {node.__class__.__name__} does not support inputs:\n{input_text}")


def test_sine_node() -> None:
	from aim.nodes import load, OutNode, build_node_graph, execution_order

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
		graph, node_ids = build_node_graph(context)
		order = execution_order(graph)
		compilation_context = CompilationContext(context, graph, node_ids, order)
		code: str = compile_to_numpy(compilation_context, sample_rate=10, frame_count=10)

		code += "\nresult = numpy_process()"

		a = {}
		exec(code, a)


def test_math_nodes() -> None:
	import numpy as np

	assert np.all(run_code_for_testing("out(add(0,1) + 5 + add(2,0) / add(4,0) * 2)")["unnamed_0"].voices[0] == 1 + 5 + 2 / 4 * 2)


def test_sub_node() -> None:
	import numpy as np
	r = run_code_for_testing("out(sub(20,5.0))")["unnamed_0"].voices[0]
	assert np.all(r == 15)
	run_code_for_testing("out(sine(440) + 5)")
	run_code_for_testing("out(sub(in0=5, in1=sine(440)))")
	run_code_for_testing("out(sub(in0=sine(440), in1=5))")
	run_code_for_testing("out(sine(440) + sine(880))")

	# TODO merayen verify output of all

def test_out_node() -> None:
	import numpy as np

	assert np.all(run_code_for_testing("out(5*5)")["unnamed_0"].voices == 25)


def run_code_for_testing(code: str, frame_count=10, sample_rate=48000) -> Any:
	from aim.nodes import load, build_node_graph, execution_order

	context: Context = load(code)
	graph, node_ids = build_node_graph(context)
	order = execution_order(graph)
	compilation_context = CompilationContext(context, graph, node_ids, order)
	code: str = compile_to_numpy(compilation_context, frame_count=10, sample_rate=sample_rate)

	code += "\nresult = numpy_process()"

	a = {}
	exec(code, a)

	return a["result"]


if __name__ == '__main__':
	for x in dir():
		if x.startswith("test_"):
			exec(f"{x}()")
