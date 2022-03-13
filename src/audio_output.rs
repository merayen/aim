use std::sync::mpsc::{Sender, Receiver};

#[repr(C)]
pub struct ao_sample_format {
	bits: std::os::raw::c_int,
	rate: std::os::raw::c_int,
	channels: std::os::raw::c_int,
	byte_format: std::os::raw::c_int,
	matrix: *mut std::os::raw::c_void,
}


const AO_FMT_LITTLE: std::os::raw::c_int = 1;
const AO_FMT_BIG: std::os::raw::c_int = 2;
const AO_FMT_NATIVE: std::os::raw::c_int = 4;

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

pub struct AudioOutput {
	bits: i32,
	channels: i32,
	rate: i32,
	tx: Sender<Vec<i8>>,
}

impl AudioOutput {
	pub fn new(channels: i32, rate: i32) -> AudioOutput {


		let (tx, rx): (Sender<Vec<i8>>, Receiver<Vec<i8>>) = std::sync::mpsc::channel();

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

				assert!(device != std::ptr::null_mut(), "Device is null");
			}

			loop {
				for raw_buffer in rx.recv() {
					unsafe {
						ao_play(device, raw_buffer.as_ptr(), raw_buffer.len());
					}
				}

			}

			unsafe {
				assert!(ao_close(device) == 1);
				ao_shutdown();
			}
		});
	
		AudioOutput {
			bits: 16,
			channels: channels,
			rate: rate,
			tx: tx,
		}
	}

	pub fn output(&self, buffer: &Vec<f32>) {
		let mut raw_buffer = vec![0i8; ((self.bits as usize) / 8usize * buffer.len()).try_into().unwrap()];

		for i in 0..buffer.len() / (self.channels as usize) {
			let sample = (0.75 * 32768.0 * buffer[i * (self.channels as usize)]) as i32;
			raw_buffer[4*i] = (sample & 0xff) as i8;
			raw_buffer[4*i+1] = ((sample >> 8) & 0xff) as i8;

			let sample = (0.75 * 32768.0 * buffer[i * (self.channels as usize) + 1]) as i32;
			raw_buffer[4*i+3] = ((sample >> 8) & 0xff) as i8;
			raw_buffer[4*i+2] = (sample & 0xff) as i8;
		}

		self.tx.send(raw_buffer);
	}
}
