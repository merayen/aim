use crate::parse_nodes::{ParseResults, TextConsumer};

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
