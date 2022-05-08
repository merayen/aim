//! Parsing of nodes from text files that looks like:
//!
//! ```
//! sine
//!   frequency 440
//! ```

use std::collections::HashMap;
use crate::parse;
use crate::nodes;


/// A node that parses its .txt-file
trait ParseNode {
	fn parse(text_consumer: parse::TextConsumer);
}


pub struct ParseResults {
	/// Errors that are shown in the stdout of the synth
	pub errors: Vec<String>,

	/// ProcessNodes configured from the module
	pub nodes: HashMap<String, Option<Box<dyn nodes::common::ProcessNode>>>,
}

pub struct Command { // TODO merayen should we use < or [? Should commands and properties have different characters?
	/// The command text excluding the `<` and `>` sign
	pub text: String,

	/// Line number in the text file being parsed
	pub line_number: usize,

	/// x-offset
	pub offset: usize,
}

pub fn parse_commands(text: &str) -> Vec<Command> {
	let mut result: Vec<Command> = Vec::new();

	result
}


/// Parse the header of a node and return the ID
///
/// Example:
/// ```
/// sine IDabc
/// ```
/// ...would return "IDabc"
fn parse_node_header(result: &mut ParseResults, text_consumer: &mut parse::TextConsumer) -> (String, String) {
	assert!(text_consumer.current_indentation() == 0);

	let header = text_consumer.current_line();
	let mut splitter = header.splitn(2, " ");

	// Spool past the name of the node, e.g "sine", "out", as the node is already identified
	let name = splitter.next().unwrap();

	let id = splitter.next().expect("The node should have an id set by now").trim().to_owned();

	(name.to_string(), id.to_string())
}


fn parse_node(result: &mut ParseResults, text_consumer: &mut parse::TextConsumer) {
	let title_line = text_consumer.current_line();

	let (name, id) = parse_node_header(result, text_consumer);

	// Figure out which node we should pass the data to
	let node = match name.as_str() {
		"sine" => { Some(nodes::sine::parse(result, text_consumer)) }
		_ => {
			let line = text_consumer.current_line() + "  # ERROR: Unknown node";
			let lol = Box::new(line.to_owned());
			// TODO merayen clean up code
			text_consumer.replace_line(lol.to_string());
			text_consumer.next_sibling();
			// TODO merayen

			None
		}
	};

	text_consumer.leave();

	assert!(!result.nodes.contains_key(&id), "ID {} has already been used", id);
	result.nodes.insert(id, node);
}


/// Parse a module, e.g main.txt, verify, autocomplete and return changed text.
///
/// Will return error messages back into the file.
pub fn parse_module_text(raw_text: &str) -> (ParseResults, String) {
	let text = initialize_nodes(raw_text);
	let owned_text = text.to_owned();
	let mut text_consumer = parse::TextConsumer::new(&owned_text);

	let mut result = ParseResults {
		errors: Vec::new(),
		nodes: HashMap::new(),
	};

	while text_consumer.has_next() {
		parse_node(&mut result, &mut text_consumer);
		assert!(text_consumer.current_indentation() == 0, "Node did not read all its data, left too early");
	}

	(result, text_consumer.to_string())
}


/// Get all node IDs in a module
fn get_all_node_ids(text: &str) -> Vec<String> {
	let mut ids: Vec<String> = Vec::new();
	for line in text.lines() {
		if line.trim().len() > 0 {
			if line.chars().next().unwrap() != '\t' { // TODO maybe we should support other type of indentations
				let mut header = line.split(" ");
				header.next(); // Skip name of node
				match header.next() {
					Some(id) => {
						ids.push(id.to_string());
					}
					None => {}
				}
			}
		}
	}
	ids
}


/// Get the highest id in a list consisting of ids.
///
/// Example: `get_highest_id(vec!["id1", "id12", "custom_id"]) -> 12`
fn get_highest_id(ids: Vec<String>) -> u32 {
	let mut max = 0u32;

	for id in ids {
		let mut split = id.splitn(2, "id");
		split.next(); // Remove the "id" part

		match split.next() {
			Some(id_number) => {
				match id_number.trim().parse::<u32>() {
					Ok(number) => {
						if number > max {
							max = number;
						}
					}
					Err(error) => {} // Ignore other types of ids
				}
			}
			None => {} // No id set on this node yet
		}
	}

	max
}


/// Do the first pass of the module text
///
/// This assigns a new ID to all the nodes that does not have that already.
fn initialize_nodes(text: &str) -> String {
	let mut next_id = get_highest_id(get_all_node_ids(text)) + 1;

	let mut result = String::with_capacity((text.len() as f32 * 1.1f32) as usize);
	for line in text.split("\n") {
		if line.trim().len() > 0 {
			if line.chars().next().unwrap() != '\t' {
				// Top-level, also a node
				let mut header = line.splitn(2, " ");
				let node_name = header.next().unwrap();
				match header.next() {
					Some(v) => { // Node has id already
						result.push_str(line);
					}
					None => { // No id, we assign it
						result.push_str((line.to_string() + " id" + next_id.to_string().as_str()).as_str());

						next_id += 1;
					}
				}
			} else {
				result.push_str(line);
			}
			result.push_str("\n");
		}
	}

	result.trim().to_string()
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn getting_highest_id_value() {
		let highest = get_highest_id(vec!["id12".to_string(), "id5".to_string(), "id1".to_string()]);
		assert!(highest == 12);
	}

	#[test]
	fn getting_node_ids() {
		let ids = get_all_node_ids("
a id1
	b id2
c
d id3
		".trim());

		assert!(ids == vec!["id1".to_string(), "id3".to_string()])
	}

	//#[test]
	fn initializing_nodes() {
		let result = initialize_nodes("
a
	b
c id5
	d id1337
e id9
		".trim());

		assert!(result == "
a id10
	b
c id5
	d id1337
e id9
		".trim());
	}

	//#[test]
	fn parsing_text() {
		let (result, text) = parse_module_text("
sine id0
	frequency 440
out id1
	amplitude 1
		".trim());

		assert!(result.nodes.len() == 2);
		assert!(text == "
sine id0
	frequency 440
out id1  # ERROR: Unknown node
	amplitude 1
		".trim());
	}

	//#[test]
	fn consuming_errors() {
		let (parsed, text) = parse_module_text("
a
	b
		c
	d
e
		".trim());
		// TODO merayen
	}

	#[test]
	fn unknown_node() {
		let (result, text) = parse_module_text("
lolwat id0
	lolproperty 1337
".trim());

		assert!(result.nodes.len() == 1);
		assert!(text == "
lolwat id0  # ERROR: Unknown node
	lolproperty 1337
		".trim());
	}

	#[test]
	fn sine_unknown_property() {
		let (result, text) = parse_module_text("
sine id0
	lolproperty 1337
		".trim());

		assert!(result.nodes.len() == 1);
		assert!(text == "
sine id0
	lolproperty 1337  # ERROR: Unknown property
		".trim());
	}
}
