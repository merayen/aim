import sounddevice as sd
import threading


event = threading.Event()

def callback(outdata, frames, time, status):
	assert frames == frame_count

	if status.output_underflow:
		print('Output underflow: increase blocksize?')
		raise sd.CallbackAbort

	assert not status, status

	data = (numpy_process()*.1).reshape(frame_count, 1).tobytes(order="c")

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
