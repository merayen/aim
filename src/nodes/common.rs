use std::collections::HashMap;

pub trait ProcessNode {
	fn new(env: &ProcessNodeEnvironment) -> (Self, Ports<'static>) where Self: Sized;
	fn process(&mut self, index: usize, node_ports: &mut Vec<Ports>, env: &ProcessNodeEnvironment);
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
	pub fn signal(buffer_size: usize) -> Self {
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

pub struct Ports<'a> {
	pub inlets: HashMap<&'a str, Inlet>,
	pub outlets: HashMap<&'a str, Outlet>,
}

impl Ports<'_> {
	pub fn new() -> Ports<'static> {
		Ports {
			inlets: HashMap::new(),
			outlets: HashMap::new(),
		}
	}
}
