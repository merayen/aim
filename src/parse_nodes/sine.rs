use crate::parse_nodes::{ParseResults, TextConsumer};

// TODO merayen move these two methods out
fn parse_common_node_properties(result: &mut ParseResults, text_consumer: &mut TextConsumer) {
}

/// Parse the header of a node and return the ID
///
/// Example:
/// ```
/// sine IDabc
/// ```
/// ...would return "IDabc"
fn parse_node_header(result: &mut ParseResults, text_consumer: &mut TextConsumer) -> Option<String> {
	let mut header = text_consumer.current().unwrap().text.split(" ");

	// Spool past the name of the node, e.g "sine", "out", as the node is already identified
	header.next();

	let id = match header.next() {
		Some(v) => {
			Some(v.trim().to_string())
		}
		None => {
			None  // TODO merayen No ID yet on this node. It will be added later...? How should we do that? Write back after reading all the nodes?
		}
	};

	text_consumer.consume(result);

	id
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
						text_consumer.consume_with_error(result, "Unknown property");
					}
				}
			}
			None => {
				break;  // Nothing more for us to process
			}
		}
	}
}
