use std::collections::HashMap;

trait ProcessNode {
	fn new(env: &ProcessNodeEnvironment) -> (Self, Ports<'static>) where Self: Sized;
	fn process(&mut self, index: usize, node_ports: &mut Vec<Ports>, env: &ProcessNodeEnvironment);
}

struct ProcessNodeEnvironment {
	buffer_size: usize,
	sample_rate: i32,
}

struct Outlet {
	audio: Option<Vec<Vec<f32>>>,
	signal: Option<Vec<f32>>,
	midi: Option<Vec<u8>>,
}

struct Inlet {
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

struct Ports<'a> {
	inlets: HashMap<&'a str, Inlet>,
	outlets: HashMap<&'a str, Outlet>,
}

impl Ports<'_> {
	fn new() -> Ports<'static> {
		Ports {
			inlets: HashMap::new(),
			outlets: HashMap::new(),
		}
	}
}

struct Sine {
	frequency: f32,
	position: f64,
}

impl ProcessNode for Sine {
	fn new(env: &ProcessNodeEnvironment) -> (Self, Ports<'static>) {
		let mut ports = Ports::new();
		ports.outlets.insert("out", Outlet::signal(env.buffer_size));
		//let mut out_lol = out_data.signal.unwrap();
		(Sine { frequency: 440f32, position: 0f64 }, ports)
	}
	
	fn process(&mut self, index: usize, node_ports: &mut Vec<Ports>, env: &ProcessNodeEnvironment) {
		//let mut out = ports.outlets.get_mut("out");
		//let mut out_data = out.as_mut().unwrap();
		//let mut signal = out_data.signal.as_mut().unwrap();

		//let sample_rate = env.sample_rate as f64;
		//let frequency = self.frequency as f64;

		//for i in 0..env.buffer_size {
		//	signal[i] = 1337f32;
		//	self.position += frequency / sample_rate * std::f64::consts::PI;
		//}
	}
}

struct Out {}

impl ProcessNode for Out {
	fn new(env: &ProcessNodeEnvironment) -> (Self, Ports<'static>) {
		(Out {}, Ports::new())
	}

	fn process(&mut self, index: usize, node_ports: &mut Vec<Ports>, env: &ProcessNodeEnvironment) {
		//match (&ports.inlets).get("in") {
		//	Some(out) => {
		//		println!("{}", out.node_index);
		//		for i in 0..env.buffer_size {
		//		}
		//	}
		//	None => {
		//		// TODO merayen send empty data to mixer
		//	}
		//}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn process() {
		let env = ProcessNodeEnvironment { buffer_size: 8, sample_rate: 44100};

		let mut nodes: Vec<Box<dyn ProcessNode>> = Vec::new();
		let mut node_ports: Vec<Ports> = Vec::new();

		{
			let (mut node, mut ports) = Sine::new(&env);
			nodes.push(Box::new(node));
			node_ports.push(ports);
		}

		{
			let (mut node, mut ports) = Out::new(&env);
			nodes.push(Box::new(node));
			node_ports.push(ports);
		}

		// Connecting
		(&mut node_ports[1].inlets).insert("in", Inlet { node_index: 0, outlet_name: "out".to_string() });

		nodes[0].process(0, &mut node_ports, &env);
		nodes[1].process(1, &mut node_ports, &env);
	}
}
