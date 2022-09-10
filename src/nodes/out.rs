//! Sends data out of current scope
//!
//! If in the topmost scope, and audio-like, send it to the speakers
use crate::nodes;
use crate::parse;
use crate::module::process;
use crate::parse_nodes;

struct OutNode {}

pub fn parse(result: &mut parse_nodes::ParseResults, indent_block: &mut parse::IndentBlock) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	let mut frequency = 440f32;

	for parameter_indent_block in &mut indent_block.children {
		match nodes::common::parse_input_parameter(&parameter_indent_block.text) {
			Ok(nodes::common::PortParameter::Constant {name, value}) => {
				match name.as_str() {
					"frequency" => {
						frequency = value.parse::<f32>().unwrap();
					}
					_ => {
						parameter_indent_block.text.push_str("  # ERROR: Unknown parameter");
					}
				}
			}
			Ok(nodes::common::PortParameter::Inlet {name, node_id, outlet}) => {
			}
			Err(message) => {
				parameter_indent_block.text.push_str(&("  # ERROR: ".to_string() + &message));
			}
			_ => {
				parameter_indent_block.text.push_str("  # ERROR: Invalid parameter");
			}
		}
	}

	Box::new(
		OutNode { }
	)
}

impl nodes::common::ProcessNode for OutNode {
	fn on_init(&mut self, env: &nodes::common::ProcessNodeEnvironment) -> nodes::common::Ports {
		let mut ports = nodes::common::Ports::new();
		ports.inlet("in");

		ports
	}

	fn on_process(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &mut nodes::common::Ports) {
		panic!("Yay, it works"); // TODO merayen remove
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
