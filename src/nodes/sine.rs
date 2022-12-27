use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::nodes;
use crate::module::process;
use crate::module;

pub fn new(indent_block: &mut parse::IndentBlock, ports: &mut nodes::common::Ports) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	let mut frequency = 440f32;

	for parameter_indent_block in &mut indent_block.children {
		match parse_nodes::parse_node_parameter(&parameter_indent_block.text) {
			Ok(parse_nodes::PortParameter::Constant {name, value}) => {
				match name.as_str() {
					"frequency" => {
						frequency = value.parse::<f32>().unwrap();
					}
					_ => {
						parameter_indent_block.text.push_str("  # ERROR: Unknown parameter");
					}
				}
			}
			Ok(parse_nodes::PortParameter::Inlet {name, node_id, outlet}) => {
				match name.as_str() {
					"frequency" => {
						ports.inlets.insert(name, Some(nodes::common::Inlet { node_id, outlet }));
					}
					_ => {
						parameter_indent_block.text.push_str("  # ERROR: Unknown parameter");
					}
				}
			}
			Err(message) => {
				parameter_indent_block.text.push_str(&message);
			}
			_ => {
				parameter_indent_block.text.push_str("  # ERROR: Invalid parameter");
			}
		}
	}

	// Create our only output port
	ports.signal("out");

	Box::new(
		SineNode {
			frequency,
			position: 0f32,
		}
	)
}

pub struct SineNode {
	/// Used for unconnected frequency inlet
	frequency: f32,

	/// Current position of the oscillator
	position: f32,
}

impl nodes::common::ProcessNode for SineNode {
	fn on_init(&mut self, node_id: String, env: &nodes::common::ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>) {
	}
	
	fn on_process(&mut self, node_id: String, env: &nodes::common::ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>) {
		let inlets = &ports[&node_id].inlets;
		let outlets = &ports[&node_id].outlets;

		if inlets.contains_key("frequency") {
			// We receive frequency information on inlet, so we use that to modulate the oscillator
			unimplemented!("Support modulation of frequency"); // TODO merayen implement frequency modulation
		} else {
			// We generate wave on a fixed frequency
			let mut a = outlets["out"].borrow_mut();
			let b = a.signal.as_mut().unwrap();

			// Check if we have a voice on our outlet
			if b.len() == 0 {
				// Create a new voice
				let new_voice = vec![0f32; env.buffer_size];
				b.push(new_voice);
			}

			let mut position = self.position;
			for out_voice in b {
				position = self.position;
				for (i,sample) in out_voice.iter_mut().enumerate() {
					*sample = position.sin();
					position += (self.frequency as f32 / env.sample_rate as f32 * 2f32) as f32;
				}
			}

			self.position = position % (std::f32::consts::PI * 2f32);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::project;

	#[test]
	fn create_node_and_process() {
		let env = nodes::common::ProcessNodeEnvironment {
			sample_rate: 44100,
			buffer_size: 8,
		};
		let (parse_results, text) = project::run_single_module("
sine
	frequency 100
		",
			&env,
			1, // Process only single frame
		);

		assert!(text == "
sine id1
	frequency 100
		".trim());

		assert!(parse_results.nodes.len() == 1);
		assert!(parse_results.nodes.contains_key("id1"));
	}
}
