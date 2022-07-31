use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::nodes;
use crate::module::process;

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
		SineNode {
			frequency: frequency,
			position: 0f64,
		}
	)
}

pub struct SineNode {
	frequency: f32,
	position: f64,
}

impl nodes::common::ProcessNode for SineNode {
	fn on_init(&mut self, env: &nodes::common::ProcessNodeEnvironment) -> nodes::common::Ports {
		let mut ports = nodes::common::Ports::new();
		ports.signal("out", env);
		ports.inlet("frequency");

		ports
	}
	
	fn on_process(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &mut nodes::common::Ports, session: &process::session::Session) {
		panic!("Yay, it works"); // TODO merayen remove
		let mut out = ports.outlets.get_mut("out");
		let mut out_data = out.as_mut().unwrap();
		let mut signal = out_data.signal.as_mut().unwrap();

		let sample_rate = env.sample_rate as f64;
		let frequency = self.frequency as f64;

		for (voice, is_used) in signal.iter_mut().zip(session.active_voices.iter()) {}

		//for voice in signal {
		//	match voice {
		//		Some(voice_signal) => {
		//			for i in 0..env.buffer_size {
		//				voice_signal[i] = 1337f32;
		//				self.position += frequency / sample_rate * std::f64::consts::PI;
		//			}
		//		}
		//		None => {}
		//	}
		//}
	}

	fn on_create_voice(&mut self, index: usize) {
	}

	fn on_destroy_voice(&mut self, index: usize) {
	}

	fn holds_voice(&self, index: usize) -> bool {
		false
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
		);

		assert!(text == "
sine id1
	frequency 100
		".trim());

		assert!(parse_results.nodes.len() == 1);
		assert!(parse_results.nodes.contains_key("id1"));
	}
}
