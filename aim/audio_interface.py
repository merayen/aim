import sounddevice as sd
import threading


event = threading.Event()

_SILENCE = np.zeros(frame_count, dtype="float32")

def callback(outdata, frames, time, status):
	assert frames == frame_count

	if status.output_underflow:
		print('Output underflow: increase blocksize?')
		raise sd.CallbackAbort

	assert not status, status

	# TODO merayen handle channels and channel_map
	# TODO merayen handle midi outputs too... Send to hardware devices?
	output = numpy_process().values()
	data = (sum((voice for out in output for voice in out.data.values() if isinstance(out, Signal)), _SILENCE)*.1).reshape(frame_count, 1).tobytes(order="c")

	if len(data) < len(outdata):  # End of stream, zero the last part
		outdata[:len(data)] = data
		outdata[len(data):] = b'\x00' * (len(outdata) - len(data))
		raise sd.CallbackStop
	else:
		assert len(outdata) == len(data), f"{len(outdata)=!r} != {len(data)=!r}"
		outdata[:] = data

with sd.RawOutputStream(
	samplerate=sample_rate,
	blocksize=frame_count,
	channels=1,
	dtype='float32',
	callback=callback,
	finished_callback=event.set,
) as stream:
	event.wait()  # Wait until playback is finished
