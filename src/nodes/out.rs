struct Out {}

impl ProcessNode for Out {
	fn new(env: &ProcessNodeEnvironment) -> (Self, Ports<'static>) {
		(Out {}, Ports::new())
	}

	fn on_process(&mut self, index: usize, node_ports: &mut Vec<Ports>, env: &ProcessNodeEnvironment) {
		//match (&ports.inlets).get("in") {
		//	Some(out) => {
		//		for i in 0..env.buffer_size {
		//		}
		//	}
		//	None => {
		//		// TODO merayen send empty data to mixer
		//	}
		//}
	}
}

