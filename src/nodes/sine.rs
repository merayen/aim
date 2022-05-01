use std::collections::HashMap;
use crate::parse_nodes;
use crate::nodes;

// TODO merayen move out
/// Parse common property or write an error about them if unknown
///
/// These are usually inlets and outlets. Run this if your node doesn't match any.
fn parse_common_node_property(result: &mut parse_nodes::ParseResults, text_consumer: &mut parse_nodes::TextConsumer) {
	text_consumer.consume_with_error(result, "Unknown property");
}

pub fn parse(result: &mut parse_nodes::ParseResults, text_consumer: &mut parse_nodes::TextConsumer) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	let indent_level = text_consumer.current().unwrap().indent_level + 1;

	// Consume the node header (e.g "sine id123")
	text_consumer.consume(result);

	let mut frequency = None;

	// Parse properties
	loop {
		match text_consumer.current() {
			Some(line) => {
				if line.indent_level < indent_level {
					break;  // We are done parsing data on our level
				}

				let mut key_value = line.text.splitn(2, " ");
				let key = key_value.next().unwrap();

				match key {
					"frequency" => {
						match key_value.next() {
							Some(value) => {
								match value.parse::<f32>() {
									Ok(float_value) => {
										frequency = Some(float_value);
										text_consumer.consume(result);
									}
									Err(error) => {
										text_consumer.consume_with_error(result, format!("Invalid float value: {}", error).as_str());
									}
								}
							}
							None => {
								text_consumer.consume_with_error(result, "Missing value");
							}
						}
					}
					_ => {
						parse_common_node_property(result, text_consumer);
					}
				}
			}
			None => {
				break;  // Nothing more to process
			}
		}
	}

	Box::new(
		SineNode {
			frequency: frequency.unwrap_or(440f32),
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
	
	fn process(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &mut nodes::common::Ports) {
		let mut out = ports.outlets.get_mut("out");
		let mut out_data = out.as_mut().unwrap();
		let mut signal = out_data.signal.as_mut().unwrap();

		let sample_rate = env.sample_rate as f64;
		let frequency = self.frequency as f64;

		for i in 0..env.buffer_size {
			signal[i] = 1337f32;
			self.position += frequency / sample_rate * std::f64::consts::PI;
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
		let parse_results: parse_nodes::ParseResults = project::run_single_module("
sine
	frequency 100
		",
			&env,
		);

		for x in &parse_results.lines {
			println!("{}", x.text);
		}

		assert!(parse_results.lines[0].text == "sine id1");
		assert!(parse_results.lines[1].text == "frequency 100");
		assert!(parse_results.lines[2].text == "inlets");
		assert!(parse_results.lines[3].text == "outlets");
	}
}
