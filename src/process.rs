
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

fn test_process() {
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
