from dataclasses import dataclass
from aim.nodes import (
	Node, create_variable, Outlet, Context, DataType,
	delay, sine, out, CompilationContext,
)
from typing import Any


@dataclass
class NodeContext:
	frame_count: int  # Samples per buffer
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
		process_code.append(f"{clock} = {clock_array}[-1] % 1")

	elif isinstance(node.frequency, Outlet):
		func %= {'clock_array': clock_array, "voice_id": "voice_id"}
		init_code.append(f"{clock} = defaultdict(lambda: 0.0)")
		init_code.append(f"{node.output._variable} = Signal()")

		if node.frequency.datatype == DataType.SIGNAL:
			# Note that we only respect voices on the frequency-input
			# If e.g another input port has other voices, we just ignore them. This may or may not be
			# wanted.
			# TODO merayen remove voices that disappears on the input
			process_code.extend(
				[
					f"for voice_id, voice in {node.frequency._variable}.voices.items():",
					f"	{clock_array} = {clock}[voice_id] +"
					f"	np.cumsum(_ONES * (voice / {node_context.sample_rate}))",
					f"	{clock}[voice_id] = {clock_array}[-1] % 1",  # Save position for next time
					f"	{node.output._variable}.voices[voice_id] = {func}",
				]
			)

		elif node.frequency.datatype == DataType.MIDI:
			# XXX this code could be moved out and used by other nodes
			midi_decoder_func = create_variable()
			init_code.append(f"def {midi_decoder_func}(byte):")
			init_code.append(f"	if byte & 128: {midi_decoder_func}.data = [byte]")  # Command
			init_code.append(f"	elif {midi_decoder_func}.data: {midi_decoder_func}.data.append(byte)")  # Data

			init_code.append(f"	if len({midi_decoder_func}.data) == 3:")  # Datas with 3 packets
			init_code.append(f"		if {midi_decoder_func}.data[0] == 144:")  # Key down
			init_code.append(f"			{midi_decoder_func}.frequency = 440 * 2**(({midi_decoder_func}.data[1] - 69) / 12)")
			init_code.append(f"			{midi_decoder_func}.amplitude = {midi_decoder_func}.data[2] / 127")
			init_code.append(f"			{midi_decoder_func}.last_key = {midi_decoder_func}.data[1]")

			init_code.append(f"		elif {midi_decoder_func}.data[0] == 128 and {midi_decoder_func}.data[1] == {midi_decoder_func}.last_key:")  # Key up
			init_code.append(f"			{midi_decoder_func}.amplitude = 0")

			init_code.append(f"{midi_decoder_func}.amplitude = 0")
			init_code.append(f"{midi_decoder_func}.frequency = 0")
			init_code.append(f"{midi_decoder_func}.data = None")
			init_code.append(f"{midi_decoder_func}.last_key = None")

			process_code.extend(
				[
					f"for voice_id, voice in {node.frequency._variable}.voices.items():",
					 "	for frame, midi in voice:",
					f"		{midi_decoder_func}(midi)",
					f"	{clock_array} = {clock}[voice_id] +"
					f"	np.cumsum(_ONES * ({midi_decoder_func}.frequency / {node_context.sample_rate}))",
					f"	{clock}[voice_id] = {clock_array}[-1] % 1",  # Save position for next time
					f"	{node.output._variable}.voices[voice_id] = ({func}) * {midi_decoder_func}.amplitude",
				]
			)
		else:
			unsupported(node)

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


def numpy_midi(
	node_context: NodeContext,
	node: Node,
	init_code: list[str],
	process_code: list[str],
) -> None:
	init_code.append(f"{node.midi._variable} = Midi(voices={{0: []}})")

	introduce(init_code, ["import queue"])
	introduce(init_code, ["import threading"])

	queue = create_variable()
	init_code.append(f"{queue} = queue.Queue()")

	# Create separate thread that only listens for midi data
	midi_listener_func = create_variable()
	init_code.append(f"def {midi_listener_func}():")
	init_code.append( '	stream = open("/dev/snd/midiC4D0", "rb")')
	init_code.append( "	while 1:")
	init_code.append( "		data = stream.read(1)")
	init_code.append(f"		{queue}.put((time.time(), next(iter(data))))")  # Terrible inefficient?
	init_code.append(f"threading.Thread(target={midi_listener_func}).start()")

	# Receive data from the thread above
	data = create_variable()
	process_code.append(f"{node.midi._variable}.voices[0].clear()")
	process_code.append(f"{node.midi._variable}.raw.clear()")
	process_code.append( "while 1:")
	process_code.append( "	try:")
	process_code.append(f"		{data} = {queue}.get_nowait()")
	process_code.append(f"		{node.midi._variable}.voices[0].append((0, {data}[1]))")  # TODO merayen calculate better timing data
	process_code.append(f"		{node.midi._variable}.raw.append((0, {data}[1]))")  # TODO merayen calculate better timing data
	process_code.append( "	except queue.Empty:")
	process_code.append( "		break")


def numpy_print(
	node_context: NodeContext,
	node: Node,
	init_code: list[str],
	process_code: list[str],
) -> None:
	if node.input.datatype == DataType.MIDI:
		midi_event = create_variable()
		process_code.append(f"for voice_id, voice in {node.input._variable}.voices.items():")
		process_code.append(f"	for {midi_event} in voice:")
		process_code.append("		" + debug_print(node, f'f"voice={{voice_id}}" + str({midi_event})'))
	elif node.input.datatype == DataType.SIGNAL:
		previous_voice_count = create_variable()
		init_code.append(f"{previous_voice_count} = 0")
		process_code.append(f"global {previous_voice_count}")
		process_code.append(f"if {previous_voice_count} != len({node.input._variable}.voices):")
		process_code.append(f"	{previous_voice_count} = len({node.input._variable}.voices)")
		process_code.append("	" + debug_print(node, f'f"voices={previous_voice_count}"'))
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
				f"for voice_id in set({node.output._variable}.voices) - set({node.voices._variable}.voices):",
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

	method = introduce(
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

		if isinstance(node.on, (int, float)):
			on = create_variable()
			init_code.append(f"{on} = np.zeros({node_context.frame_count}) + {node.on}")

		if isinstance(node.off, (int, float)):
			off = create_variable()
			init_code.append(f"{off} = np.zeros({node_context.frame_count}) + {node.off}")

		process_code.append(f"for voice_id, voice in {node.value._variable}.voices.items():")
		process_code.append(f"	if voice_id not in {node.output._variable}.voices:")
		process_code.append(f"		{node.output._variable}.voices[voice_id] = np.zeros({node_context.frame_count})")
		process_code.append(f"	{current_value}[voice_id] = {method}(")
		process_code.append(f"		{current_value}[voice_id],")
		process_code.append("		voice,")
		process_code.append(f"		{node.output._variable}.voices[voice_id],")

		if isinstance(node.on, (int, float)):
			process_code.append(f"		{on},")
		elif isinstance(node.on, Outlet):
			process_code.append(f"		{node.on._variable}.voices[voice_id],")
		else:
			unsupported(node)

		if isinstance(node.off, (int, float)):
			process_code.append(f"		{off},")
		elif isinstance(node.off, Outlet):
			process_code.append(f"		{node.off._variable}.voices[voice_id],")
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


def numpy_score(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	init_code.append(f"{node.output._variable} = Midi()")

	if node.score is None:
		return
	elif isinstance(node.score, str):
		raise NotImplementedError("")  # TODO merayen implement
	else:
		unsupported(node)


def numpy_unison(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	if isinstance(node.max_voices, int):
		pass
	else:
		unsupported(node)

	if isinstance(node.input, (int, float)):
		init_code.append(f"{node.output._variable} = Signal()")
		if isinstance(node.voices, int):
			init_code.append(f"for _ in range({node.voices}):")
			init_code.append(f"	{node.output._variable}.voices[create_voice()] = _ONES + {node.input}")
	else:
		unsupported(node)

def numpy_polyphonic(
	node_context: NodeContext,
	node: out,
	init_code: list[str],
	process_code: list[str],
) -> None:
	if node.input is None:
		return

	if not isinstance(node.input, Outlet) or node.input.datatype != DataType.MIDI:
		unsupported(node)

	if not isinstance(node.max_voices, int):
		unsupported(node)

	packets = create_variable()
	init_code.append(f"{packets} = []")

	pushed_keys = create_variable()
	init_code.append(f"{pushed_keys} = {{}}")  # Format: {key id: voice id}

	# Accumulated states
	states = create_variable()
	init_code.append(f"{states} = []")

	# Create output port and forward the raw midi data
	init_code.append(f"{node.midi._variable} = Midi(raw={node.input._variable}.raw)")

	frame = create_variable()
	byte = create_variable()
	voice = create_variable()

	# Clear up any data that we sent last time
	process_code.append(f"for {voice} in {node.midi._variable}.voices.values():")
	process_code.append(f"	{voice}.clear()")

	# We only support voice 0, everything else is ignored
	process_code.append(f"for {frame}, {byte} in {node.input._variable}.voices.get(0) or []:")
	process_code.append(f"	if {byte} & 128:")
	process_code.append(f"		{packets}.clear()")
	process_code.append(f"	{packets}.append(({frame}, {byte}))")

	# Process accumulated data
	new_voice_id = create_variable()
	process_code.append(f"	if len({packets}) == 3:")  # 3 bytes package handling
	process_code.append(f"		if {packets}[0][1] == 144 and len({node.input._variable}.voices) + 1 < {node.max_voices}:")  # Key down, spawn a new voice
	process_code.append(f"			{new_voice_id} = create_voice()")
	process_code.append(f"			{node.midi._variable}.voices[{new_voice_id}] = {states} + {packets}")
	process_code.append(f"			{pushed_keys}[{packets}[1][1]] = {new_voice_id}")

	process_code.append(f"		elif {packets}[0][1] == 128 and {packets}[1][1] in {pushed_keys}:")  # Key up
	process_code.append(f"			{node.midi._variable}.voices.pop({pushed_keys}[{packets}[1][1]])")  # TODO merayen remove the voice in a more gentle manner (send key-up at correct frame number, and then dispose the voice on next cycle)
	process_code.append(f"			{pushed_keys}.pop({packets}[1][1])")
	process_code.append(f"			{packets}.clear()")
	# TODO merayen store states data like pitch bend etc. make sure to replace existing ones


def numpy_delay(
	node_context: NodeContext,
	node: delay,
	init_code: list[str],
	process_code: list[str],
) -> None:
	if isinstance(node.time, float):
		pass
	else:
		unsupported(node)


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
	fps = 30

	clock = create_variable()
	buffer = create_variable()
	samples_filled = create_variable()
	trigger_high = create_variable()
	trigger_low = create_variable()
	waiting_period = create_variable()

	init_code.append(f"{clock} = {{}}")  # Sample clock for each voice. Always increases. Used for comparing offsets like waiting_period
	init_code.append(f"{waiting_period} = {{}}")  # How much we must wait before we search for trigger, limits fps
	init_code.append(f"{trigger_high} = {{}}")  # Where in the current buffer to start triggering. Offset from clock-variable
	init_code.append(f"{trigger_low} = {{}}")  # When a voice value has gone below the trigger_low thresholds, allowing trigging again. Offset from clock-variable
	init_code.append(f"{samples_filled} = {{}}")  # How much of the buffer we have filled
	init_code.append(f"{buffer} = {{}}")  # Output buffer for each voice, sent to the parent process

	if isinstance(node.trigger_low, (int, float)):
		pass
	elif node.trigger_low is None:
		node.trigger_low = node.trigger
	else:
		unsupported(node)

	if isinstance(node.value, Outlet):
		# We just forward the whole Outlet, as we are only forwarding the data
		init_code.append(f"{node.output._variable} = {node.value._variable}")

		if isinstance(node.time_div, (int, float)):
			# Look for new voices and create buffers for them
			buffer_size = int(node_context.sample_rate * max(1E-4, min(1, node.time_div)))
			process_code.append(f"for voice_id in set({node.value._variable}.voices) - set({buffer}):")
			process_code.append(f"	{clock}[voice_id] = 0")
			process_code.append(f"	{waiting_period}[voice_id] = 0")
			process_code.append(f"	{trigger_high}[voice_id] = {2**63-1}")
			process_code.append(f"	{trigger_low}[voice_id] = {2**63-1}")
			process_code.append(f"	{samples_filled}[voice_id] = {2**63-1}")
			process_code.append(f"	{buffer}[voice_id] = np.zeros({buffer_size})")
		else:
			unsupported(node)

		# Remove dead voices
		process_code.append(f"for voice_id in set({buffer}) - set({node.value._variable}.voices):")
		process_code.append(f"	{clock}.pop(voice_id)")
		process_code.append(f"	{buffer}.pop(voice_id)")
		process_code.append(f"	{waiting_period}.pop(voice_id)")
		process_code.append(f"	{trigger_high}.pop(voice_id)")
		process_code.append(f"	{trigger_low}.pop(voice_id)")
		process_code.append(f"	{samples_filled}.pop(voice_id)")
		process_code.append("	" + emit_data(node, "{'voice_id': voice_id, 'samples': []}"))

		# Scan for trigger points. First low point, then high point.
		# Scans each voice's buffer for trigger points.
		if isinstance(node.trigger, (int, float)):
			process_code.append(f"for voice_id, voice in {node.value._variable}.voices.items():")

			# Too early to start looking? This is to limit fps
			process_code.append(f"	if {waiting_period}[voice_id] > {clock}[voice_id]: continue")

			trigger_index = create_variable()

			# Search for trigger_low value if not timed out or already found
			process_code.append(f"	if {trigger_low}[voice_id] == {2**63-1}:")
			process_code.append(f"		{trigger_index} = np.argmax(voice < {node.trigger_low})")
			process_code.append(f"		if {trigger_index} > 0 or voice[0] < {node.trigger_low}:")
			process_code.append(f"			{trigger_low}[voice_id] = {trigger_index} + {clock}[voice_id]")
			process_code.append( "		else: continue")  # Low not found, no reason to look for high below

			# Search for trigger_high value if not timed out or already found
			process_code.append(f"	if {trigger_low}[voice_id] != {2**63-1} and {trigger_high}[voice_id] == {2**63-1}:")
			trigger_high_offset = create_variable()
			process_code.append(f"		{trigger_high_offset} = max(0, {trigger_low}[voice_id] - {clock}[voice_id]) if {trigger_low}[voice_id] < {node_context.frame_count} + {clock}[voice_id] else 0")
			process_code.append(f"		assert 0 <= {trigger_high_offset} < {node_context.frame_count}, {trigger_high_offset}")
			process_code.append(f"		{trigger_index} = np.argmax(voice[{trigger_high_offset}:] >= {node.trigger})")
			process_code.append(f"		if {trigger_index} > 0 or voice[{trigger_high_offset}] >= {node.trigger}:")
			process_code.append(f"			{trigger_high}[voice_id] = {trigger_index} + {clock}[voice_id] + {trigger_high_offset}")
			process_code.append(f"			{samples_filled}[voice_id] = 0")
			process_code.append(f"	assert {trigger_high}[voice_id] >= {trigger_low}[voice_id], f'trigger_high={{{trigger_low}[voice_id]}} > trigger_high={{{trigger_high}[voice_id]}}'")
		else:
			unsupported(node)

		if isinstance(node.time_div, (int, float)):
			# Add samples to the buffer until it is full. For each voice.
			process_code.append(f"for voice_id, voice in {node.value._variable}.voices.items():")

			process_code.append(
				f"	if {trigger_high}[voice_id] == {2**63-1}: continue"
			)

			read_offset = create_variable()
			write_offset = create_variable()
			size = create_variable()
			write_stop = create_variable()
			process_code.append(f"	{read_offset} = max(0, {trigger_high}[voice_id] - {clock}[voice_id]) if {samples_filled}[voice_id] == 0 else 0")
			process_code.append(f"	{size} = min({buffer_size} - {samples_filled}[voice_id], {node_context.frame_count} - {read_offset})")
			process_code.append(f"	{write_offset} = {samples_filled}[voice_id]")
			process_code.append(f"	{write_stop} = {samples_filled}[voice_id] + {size}")
			process_code.append(f"	assert -1 < {size} <= {node_context.frame_count}, {size}")
			process_code.append(f"	assert -1 < {write_offset} < {buffer_size}, {write_offset}")
			process_code.append(f"	assert {write_stop} > -1, {write_stop}")
			process_code.append(
				f"	{buffer}[voice_id][{write_offset}:{write_offset} + {size}] = voice[{read_offset}:{read_offset} + {size}]"
			)

			# Update the counter for samples sampled, which decides when to stop sampling
			process_code.append(f"	{samples_filled}[voice_id] += {size}")
		else:
			unsupported(node)

		# Check if we have enough samples and if it is time to update oscilloscope view
		if isinstance(node.time_div, (int, float)):
			process_code.append(f"for voice_id, samples in {buffer}.items():")
			process_code.append(f"	if {samples_filled}[voice_id] < {buffer_size}: continue")
			process_code.append(
				"	" + emit_data(node, "{'voice_id': voice_id, 'samples': samples.tolist() }")
			)
			process_code.append(f"	{samples_filled}[voice_id] = 0")
			process_code.append(f"	{clock}[voice_id] = 0")
			process_code.append(f"	{waiting_period}[voice_id] = {clock}[voice_id] + {round(node_context.sample_rate * (1/fps) - buffer_size)}")
			process_code.append(f"	{trigger_high}[voice_id] = {2**63-1}")
			process_code.append(f"	{trigger_low}[voice_id] = {2**63-1}")
		else:
			unsupported(node)

		# Update clock
		process_code.append(f"for voice_id in {buffer}:")
		process_code.append(f"	{clock}[voice_id] += {node_context.frame_count}")

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
		"@dataclass",
		"class Midi:",
		"	voices: dict[int, list[tuple[int, int]]] = field(default_factory=lambda:{})",
		"	raw: list[int, bytes] = field(default_factory=lambda:[])",  # All data. For still transferring pitch wheel data etc.
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


def introduce(init_code: list[str], lines: list[str]):
	if lines in introduce.lines.values():
		return next(k for k,v in introduce.items() if v == lines) # Already defined

	method_name = create_variable()

	init_code.extend([x.replace("METHOD_NAME", method_name) for x in lines])

	introduce.lines[method_name] = lines

	return method_name
introduce.lines = {}


def emit_data(node: Node, code: str) -> str:
	"""
	Emit data from node to parent process

	E.g, the plot data for an oscilloscope.
	"""
	return f"print(json.dumps({{'node_id': {id(node)}, 'name': '{node.__class__.__name__}', 'data': {code}}}))"


def debug_print(node: Node, code: str):
	"""
	Output data straight to stdout

	Only meant for when developing nodes, not normal usage.
	"""
	# TODO merayen probably only do this when "aim --debug"
	return f"print(json.dumps({{'node': '{id(node)}', 'name': '{node.__class__.__name__}', 'debug': True, 'data': {code}}}))"


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
