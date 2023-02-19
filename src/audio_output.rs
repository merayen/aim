use std::sync::mpsc::{Sender, Receiver};

// TODO merayen pulseaudio: support pa_stream_set_write_callback, may give us lower latencies?
// TODO merayen pulseaudio: example https://gist.github.com/toroidal-code/8798775
// TODO merayen pulseaudio: documentation https://freedesktop.org/software/pulseaudio/doxygen/stream_8h.html
// TODO merayen pulseaudio: support pa_stream_set_read_callback when we want to read audio 

#[repr(C)]
pub struct ao_sample_format {
	bits: std::os::raw::c_int,
	rate: std::os::raw::c_int,
	channels: std::os::raw::c_int,
	byte_format: std::os::raw::c_int,
	matrix: *mut std::os::raw::c_void,
}


const AO_FMT_LITTLE: std::os::raw::c_int = 1;

#[allow(dead_code)]
extern {
	fn ao_initialize();
	fn ao_default_driver_id() -> std::os::raw::c_int;
	fn ao_open_live(
		driver_id: std::os::raw::c_int,
		ao_sample_format: *mut ao_sample_format,
		ao_option: *mut std::os::raw::c_void
	) -> *mut std::os::raw::c_void;
	fn ao_play(device: *mut std::os::raw::c_void, buffer: *const i8, buf_size: usize) -> i32;
	fn ao_close(device: *mut std::os::raw::c_void) -> i32;
	fn ao_shutdown();
}

enum Action {
	Play { raw_buffer: Vec<i8>},
	End(),
}

pub struct AudioOutput {
	bits: i32,
	channels: i32,
	tx: Sender<Action>,
}

impl AudioOutput {
	pub fn new(channels: i32, rate: i32) -> AudioOutput {
		let (tx, rx): (Sender<Action>, Receiver<Action>) = std::sync::mpsc::channel();

		std::thread::spawn(move || {
			let device: *mut std::os::raw::c_void;

			let mut format = ao_sample_format {
				bits: 16,
				channels: channels,
				rate: rate,
				byte_format: AO_FMT_LITTLE,
				matrix: std::ptr::null_mut(),
			};

			unsafe {
				ao_initialize();

				let default_driver = ao_default_driver_id();
				device = ao_open_live(default_driver, &mut format, std::ptr::null_mut());

				assert_ne!(device, std::ptr::null_mut(), "Device is null");
			}

			'outer: loop {
				if let Ok(op) = rx.recv() {
					match op {
						Action::Play { raw_buffer } => unsafe {
								ao_play(device, raw_buffer.as_ptr(), raw_buffer.len());
						},
						Action::End() => {break 'outer},
					};
				}
			}

			println!("Shutting down audio_output thread");

			unsafe {
				assert!(ao_close(device) == 1);
				ao_shutdown();
			};
		});
	
		AudioOutput {
			bits: 16,
			channels: channels,
			tx: tx,
		}
	}

	pub fn output(&self, buffer: &Vec<f32>) {
		// TODO merayen memory leak: we need to poll if output needs more data?
		let mut raw_buffer = vec![0i8; ((self.bits as usize) / 8usize * buffer.len()).try_into().unwrap()];

		if buffer.len() as i32 % self.channels != 0 {
			panic!("Unexpected buffer length. Must be divisible by channel count");
		}

		for i in 0..buffer.len() / (self.channels as usize) {
			let sample = (0.75 * 32768.0 * buffer[i * (self.channels as usize)]) as i32;
			raw_buffer[2*i] = (sample & 0xff) as i8;
			raw_buffer[2*i+1] = ((sample >> 8) & 0xff) as i8;

			// TODO merayen multichannel: does not support multiple channels yet
			//let sample = (0.75 * 32768.0 * buffer[i * (self.channels as usize) + 1]) as i32;
			//raw_buffer[4*i+3] = ((sample >> 8) & 0xff) as i8;
			//raw_buffer[4*i+2] = (sample & 0xff) as i8;
		}

		self.tx.send(Action::Play {raw_buffer}).unwrap();
	}

	pub fn end(&self) {
		self.tx.send(Action::End()).unwrap();
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_audio() {
		let rate = 8000;
		let channels = 2;

		let mut buffer = vec![0f32; (rate * channels / 10) as usize];

		for i in 0..buffer.len() / 2 {
			buffer[i * 2 + 0] = (2f32 * std::f32::consts::PI * 220f32 * ((i as f32) / (rate as f32))).sin();
			buffer[i * 2 + 1] = (2f32 * std::f32::consts::PI * 440f32 * ((i as f32) / (rate as f32))).sin();
		}

		let ao = AudioOutput::new(channels, rate);

		ao.output(&buffer);
		std::thread::sleep(std::time::Duration::from_millis(1000));
		//ao.output(&buffer);
		//std::thread::sleep(std::time::Duration::new(2, 0));
		ao.end();
	}
}
