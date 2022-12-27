use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::nodes;
use crate::module::process;
use crate::module;

pub fn new(indent_block: &mut parse::IndentBlock, ports: &mut nodes::common::Ports) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	let mut frequency = 440f32;

	for parameter_indent_block in &mut indent_block.children {
		match nodes::common::parse_node_parameter(&parameter_indent_block.text) {
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
				parameter_indent_block.text.push_str(&("  # ERROR: ".to_string() + &message));
			}
			_ => {
				parameter_indent_block.text.push_str("  # ERROR: Invalid parameter");
			}
		}
	}

	Box::new(
		SineNode {
			frequency,
			position: 0f64,
		}
	)
}

pub struct SineNode {
	frequency: f32,
	position: f64,
}

impl nodes::common::ProcessNode for SineNode {
	fn on_init(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>) {
	}
	
	fn on_process(&mut self, node_id: String, env: &nodes::common::ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>) {
		let frequency = &ports[&node_id].inlets;
		//let mut out: Option<&mut nodes::common::Outlet> = outlets.get_mut("out");
		//let out_data: &mut nodes::common::Outlet = out.as_mut().unwrap();
		//let out_signal: &mut Vec<Vec<f32>> = out_data.signal.as_mut().unwrap();

		//let sample_rate = env.sample_rate as f64;
		//let frequency = self.frequency as f64;

		//// TODO merayen implement handling of input data

		//if out_signal.len() == 0 {
		//	// No output voice. Create one
		//	out_signal.push(vec![0f32; env.buffer_size]);
		//}

		//for voice in out_signal {
		//	for i in 0..env.buffer_size {
		//		voice[i] = 1337f32;
		//		self.position += frequency / sample_rate * std::f64::consts::PI;
		//	}
		//}
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
