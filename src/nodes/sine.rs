use std::collections::HashMap;
use crate::parse_nodes::{ParseResults, TextConsumer};
use crate::nodes::common::{Ports, ProcessNode, ProcessNodeEnvironment, Outlet};

// TODO merayen move these two methods out
/// Parse common property or write an error about them if unknown
///
/// These are usually inlets and outlets. Run this if your node doesn't match any.
fn parse_common_node_property(result: &mut ParseResults, text_consumer: &mut TextConsumer) {
	text_consumer.consume_with_error(result, "Unknown property");
}

/// Parse the header of a node and return the ID
///
/// Example:
/// ```
/// sine IDabc
/// ```
/// ...would return "IDabc"
fn parse_node_header(result: &mut ParseResults, text_consumer: &mut TextConsumer) -> String {
	let mut header = text_consumer.current().unwrap().text.split(" ");

	// Spool past the name of the node, e.g "sine", "out", as the node is already identified
	header.next();

	let id = header.next().expect("The node should have an id set by now").trim().to_owned();

	text_consumer.consume(result);

	id.to_string()
}

pub fn parse(result: &mut ParseResults, text_consumer: &mut TextConsumer) {
	let indent_level = text_consumer.current().unwrap().indent_level + 1;

	let id = parse_node_header(result, text_consumer);

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
				break;  // Nothing more for us to process
			}
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
