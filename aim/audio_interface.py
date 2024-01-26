import sounddevice as sd
import threading

CHANNEL_COUNT = 2

event = threading.Event()

def callback(outdata, frames, time, status):
	# Is this check bad? Could frames vary?
	assert frames == frame_count

	if status.output_underflow:
		print('Output underflow: increase blocksize?')
		raise sd.CallbackAbort

	assert not status, status

	# TODO merayen handle midi outputs too... Send to hardware devices?
	output = numpy_process().values()

	output_channels = {0: np.zeros(frame_count, dtype='float32'), 1: np.zeros(frame_count, dtype='float32')}

	if not any(1 for out in output for voice_id in out.voices):
		assert np.all(_SILENCE == 0.0)

		outdata[:,0] = _SILENCE
		outdata[:,1] = _SILENCE
		return

	for out in output:
		if isinstance(out, Signal):
			for voice_id, voice in out.voices.items():

				# TODO merayen read channel_map instead of using voice_id directly as channel_index
				output_channels[voice_id % CHANNEL_COUNT] += voice * .1

	outdata[:,0] = output_channels[0]
	outdata[:,1] = output_channels[1]


with sd.OutputStream(
	samplerate=sample_rate,
	blocksize=frame_count,
	channels=CHANNEL_COUNT,
	dtype='float32',
	callback=callback,
	finished_callback=event.set,
) as stream:
	event.wait()  # Wait until playback is finished
