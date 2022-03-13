//mod nodes;
mod audio_output;

//use crate::nodes::sine::{SineNode};
//use crate::nodes::{Node, NodeData};

//use crate::audio_output;

struct Inlet<T,U> {
	outlet: Outlet<T>,
	default: U,
}

struct Outlet<T> {
	data: T,
}

struct Sine<T,U> {
	frequency: Inlet<T,U>,
	output: Outlet<Vec<f32>>,
}

impl Sine<Vec<f32>,f32> {
	fn new(frequency: Inlet<Vec<f32>,f32>, output: Outlet<Vec<f32>>) -> Self {
		Sine {
			frequency: frequency,
			output: output,
		}
	}
}

fn d(a: &mut i8) {
	*a += 1;
}

fn main() {
	let rate = 8000;
	let channels = 2;

	let mut buffer = vec![0f32; (rate * channels) as usize];

	for i in 0..buffer.len() / 2 {
		buffer[i * 2 + 0] = (2f32 * std::f32::consts::PI * 220f32 * ((i as f32) / (rate as f32))).sin();
		buffer[i * 2 + 1] = (2f32 * std::f32::consts::PI * 440f32 * ((i as f32) / (rate as f32))).sin();
	}

	let ao = audio_output::AudioOutput::new(channels, rate);
	ao.output(&buffer);

	std::thread::sleep(std::time::Duration::new(5, 0));
	ao.output(&buffer);
	std::thread::sleep(std::time::Duration::new(5, 0));

	let mut n = 32i8;
	for _ in 0..32 {
		d(&mut n);
	}

	println!("{}", n);

	let mut _a = Sine::new(
		Inlet {
			outlet: Outlet {
				data: vec![440f32],
			},
			default: 100f32,
		},
		Outlet {
			data: vec![0f32],
		},
	);
}
