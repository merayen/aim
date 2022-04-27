use std::collections::HashMap;

pub trait ProcessNode {
	/// Initialize your ports here
	fn on_init(&mut self, env: &ProcessNodeEnvironment) -> Ports;

	/// Process a frame
	fn process(&mut self, env: &ProcessNodeEnvironment, ports: &mut Ports);
}

pub struct ProcessNodeEnvironment {
	pub buffer_size: usize,
	pub sample_rate: i32,
}

pub struct Outlet {
	audio: Option<Vec<Vec<f32>>>,
	signal: Option<Vec<f32>>,
	midi: Option<Vec<u8>>,
}

pub struct Inlet {
	/// Index to the node that this Inlet is connected to
	node_index: usize, 

	/// Name of the outlet of the node connected to this Inlet
	outlet_name: String,
}

impl Outlet {
	fn signal(buffer_size: usize) -> Self {
		let mut signal = Vec::with_capacity(buffer_size);
		for i in 0..buffer_size {
			signal.push(0f32);
		}
		Outlet {
			audio: None,
			signal: Some(signal),
			midi: None,
		}
	}

	// TODO merayen add audio and midi too
}

pub struct Ports {
	pub inlets: HashMap<String, Option<Inlet>>,
	pub outlets: HashMap<String, Outlet>,
}

impl Ports {
	pub fn new() -> Ports {
		Ports {
			inlets: HashMap::new(),
			outlets: HashMap::new(),
		}
	}

	/// Create a new outlet configured to send signals
	pub fn signal(&mut self, name: &str, env: &ProcessNodeEnvironment) {
		self.outlets.insert(name.to_string(), Outlet::signal(env.buffer_size));
	}

	/// Register an inlet
	pub fn inlet(&mut self, name: &str) {
		self.inlets.insert(name.to_string(), None);
	}
}
