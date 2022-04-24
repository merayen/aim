// TODO merayen remove this file

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
