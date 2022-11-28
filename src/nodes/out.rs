//! Sends data out of current scope
//!
//! If in the topmost scope, and audio-like, send it to the speakers
use std::collections::HashMap;
use crate::nodes;
use crate::parse;
use crate::module::process;
use crate::module;
use crate::parse_nodes;

struct OutNode {}

pub fn new(indent_block: &mut parse::IndentBlock, ports: &mut nodes::common::Ports) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	for indent_block in &mut indent_block.children {
		match nodes::common::parse_node_parameter(&indent_block.text) {
			Ok(nodes::common::PortParameter::Inlet {name, node_id, outlet}) => {
				match name.as_str() {
					"in" => {
						// TODO merayen connect this or...?
					}
					_ => {
						indent_block.text.push_str("  # ERROR: Unknown inlet")
					}
				}
			}
			Err(message) => {
				indent_block.text.push_str(&("  # ERROR: ".to_string() + &message));
			}
			_ => {
				indent_block.text.push_str("  # ERROR: Invalid parameter");
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

	fn on_process(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &mut HashMap<String, nodes::common::Ports>) {
		// TODO merayen send data to speakers
	}
}
