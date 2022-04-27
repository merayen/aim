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
	fn parse(lines: Vec<parse::TextLine>);
}


pub struct ParseResults {
	/// Lines that should replace the content of the module that was parsed.
	/// These lines will contain errors too, as comments.
	pub lines: Vec<parse::TextLine>,

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


impl ParseResults {
	pub fn new() -> ParseResults {
		ParseResults {
			lines: Vec::new(),
			errors: Vec::new(),
			nodes: HashMap::new(),
		}
	}

	fn to_string(&self) -> String {
		let mut result = String::new();
		for line in &self.lines {
			result += ("\t".repeat(line.indent_level as usize) + line.text.as_str() + "\n").as_str()
		}

		result
	}
}


pub struct TextConsumer {
	lines: Vec<parse::TextLine>,
	index: usize,
	indent_level: u16,
	protection: u32,
}


impl TextConsumer {
	pub fn new(lines: Vec<parse::TextLine>) -> TextConsumer {
		TextConsumer {
			lines: lines,
			indent_level: 0,
			index: 0,
			protection: 0,
		}
	}

	/// Get the current line
	pub fn current(&mut self) -> Option<&parse::TextLine> {
		if self.protection > 1000 {
			panic!("Stuck? current() called many times without consumption");
		}

		self.protection += 1;

		if self.index >= self.lines.len() {
			return None;
		}

		Some(&self.lines[self.index])
	}

	pub fn skip(&mut self) {
		self.protection = 0;
		self.index += 1;
	}

	/// Consume current line
	pub fn consume(&mut self, result: &mut ParseResults) {
		if self.index >= self.lines.len() {
			panic!("TextConsumer is already consumed");
		}

		self.protection = 0;

		let line = &self.lines[self.index];

		result.lines.push(
			parse::TextLine {
				text: String::from(&line.text),
				indent_level: line.indent_level,
				line_number: result.lines.len() + 1,
			}
		);

		// Store new indent level of previous item
		self.indent_level = line.indent_level;

		// Go to next element, if any
		self.index += 1;
	}

	/// Consume current line and its children. Adds an error message to the first line.
	pub fn consume_with_error(&mut self, result: &mut ParseResults, error_message: &str) {
		self.protection = 0;

		// Get the current indentation level
		let line = match TextConsumer::current(self) {
			Some(line) => {
				line
			}
			None => {
				panic!("Nothing to consume");
			}
		};

		let indent_level = line.indent_level;

		result.lines.push(
			parse::TextLine {
				text: line.text.to_owned() + "  # ERROR: " + error_message,
				indent_level: indent_level,
				line_number: result.lines.len() + 1,
			}
		);

		// We have manually added to result.lines, so we go to the next line
		self.index += 1;

		// Consume all the indentations below
		loop {
			match TextConsumer::current(self) {
				Some(v) => {
					if v.indent_level <= indent_level {
						break;
					}

					// Write the line to the output
					TextConsumer::consume(self, result);
				}
				None => {
					break;  // We hit the end
				}
			}
		}
	}
}


/// Parse the header of a node and return the ID
///
/// Example:
/// ```
/// sine IDabc
/// ```
/// ...would return "IDabc"
fn parse_node_header(result: &mut ParseResults, text_consumer: &mut TextConsumer) -> (String, String) {
	let mut header = text_consumer.current().unwrap().text.split(" ");

	// Spool past the name of the node, e.g "sine", "out", as the node is already identified
	let name = header.next().unwrap();

	let id = header.next().expect("The node should have an id set by now").trim().to_owned();

	(name.to_string(), id.to_string())
}


fn parse_node(result: &mut ParseResults, text_consumer: &mut TextConsumer) {
	let title_line = text_consumer.current().unwrap();

	let (name, id) = parse_node_header(result, text_consumer);

	// Figure out which node we should pass the data to
	let node = match name.as_str() {
		"sine" => { nodes::sine::parse(result, text_consumer) }
		_ => {
			text_consumer.consume_with_error(result, "Unknown node");

			None
		}
	};

	assert!(!result.nodes.contains_key(&id), "ID {} has already been used", id);
	result.nodes.insert(id, node);
}


/// Parse a module, e.g main.txt, verify, autocomplete and return changed text.
/// Will return error messages back into the file.
pub fn parse_module_text(raw_text: &str) -> ParseResults {
	let text = initialize_nodes(raw_text);
	let parsed: Vec<parse::TextLine> = parse::parse_module(&text);
	let mut text_consumer = TextConsumer::new(parsed);

	let mut result = ParseResults::new();

	loop {
		match text_consumer.current() {
			Some(line) => {
				if line.indent_level > 0 {
					panic!("Node did not consume all its data? At index: {}, text: {}", text_consumer.index, text_consumer.lines[text_consumer.index].text);
				}

				parse_node(&mut result, &mut text_consumer);
			}
			None => {
				break;
			}
		}
	}

	result
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

	let mut text_consumer = TextConsumer::new(parse::parse_module(text));
	let mut result = ParseResults::new();

	loop {
		match text_consumer.current() {
			Some(line) => {
				if line.indent_level == 0 {  // Node header
					let mut header = line.text.splitn(2, " ");
					let name = header.next().unwrap();
					match header.next() {
						Some(v) => {
							text_consumer.consume(&mut result);  // All good
						}
						None => {  // No id, we assign it
							
							// Push the line manually
							result.lines.push(
								parse::TextLine {
									text: name.to_string() + " id" + next_id.to_string().as_str(),
									indent_level: line.indent_level,
									line_number: result.lines.len() + 1,
								}
							);

							next_id += 1;

							text_consumer.skip();  // As we have written the line above manually
						}
					}
				} else {  // Node property
					text_consumer.consume(&mut result);
				}
			}
			None => {
				break;
			}
		}
	}

	result.to_string().trim().to_string()
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
	#[test]
	fn initializing_nodes() {
		let result = initialize_nodes("
a
	b
c id5
	d id1337
e id9
		".trim());

		println!("{}", result);
		assert!(result == "
a id10
	b
c id5
	d id1337
e id9
		".trim());
	}

	#[test]
	fn parsing_text() {
		let result = parse_module_text("
sine id0
	frequency 440
out id1
	amplitude 1
		".trim());

		assert!(result.nodes.len() == 2);
		assert!(result.lines.len() == 4);
		assert!(result.lines[0].indent_level == 0);
		assert!(result.lines[1].indent_level == 1);
		assert!(result.lines[2].indent_level == 0);
		assert!(result.lines[3].indent_level == 1);
		assert!(result.lines[0].text == "sine id0");
		assert!(result.lines[1].text == "frequency 440");
		assert!(result.lines[2].text == "out id1  # ERROR: Unknown node");
		assert!(result.lines[3].text == "amplitude 1");
	}

	#[test]
	fn consuming_errors() {
		let parsed: Vec<parse::TextLine> = parse::parse_module("
a
	b
		c
	d
e
		".trim());
		let mut text_consumer = TextConsumer::new(parsed);
		let mut result = ParseResults::new();
		text_consumer.consume_with_error(&mut result, "Not working");
		assert!(result.lines.len() == 4);
		assert!(result.lines[0].indent_level == 0);
		assert!(result.lines[1].indent_level == 1);
		assert!(result.lines[2].indent_level == 2);
		assert!(result.lines[3].indent_level == 1);
		assert!(result.lines[0].text == "a  # ERROR: Not working");
		assert!(result.lines[1].text == "b");
		assert!(result.lines[2].text == "c");
		assert!(result.lines[3].text == "d");
	}

	#[test]
	fn unknown_node() {
		let result = parse_module_text("
lolwat id0
	lolproperty 1337
".trim());

		assert!(result.nodes.len() == 1);
		assert!(result.lines.len() == 2);
		assert!(result.lines[0].indent_level == 0);
		assert!(result.lines[1].indent_level == 1);
		assert!(result.lines[0].text == "lolwat id0  # ERROR: Unknown node");
		assert!(result.lines[1].text == "lolproperty 1337");
	}

	#[test]
	fn sine_unknown_property() {
		let result = parse_module_text("
sine id0
	lolproperty 1337
		".trim());

		assert!(result.nodes.len() == 1);
		assert!(result.lines.len() == 2);
		assert!(result.lines[0].text == "sine id0");
		assert!(result.lines[1].text == "lolproperty 1337  # ERROR: Unknown property");
	}
}
