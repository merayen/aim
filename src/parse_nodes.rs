//! Parsing of nodes from text files that looks like:
//!
//! ```
//! sine
//!   frequency 440
//! ```

use std::collections::HashMap;
use crate::parse;
use crate::nodes;
use crate::module;


/// A node that parses its .aim-file
trait ParseNode {
	fn parse(text_consumer: parse::IndentBlock);
}


/// Parsed port type
pub enum PortParameter {
	/// The parameter is connected to an inlet
	Inlet { name: String, node_id: String, outlet: String },

	/// An output port
	Outlet { name: String, node_id: String, inlet: String },

	/// The parameter is using a constant
	Constant { name: String, value: String},
}


pub struct Command { // TODO merayen should we use < or [? Should commands and parameters have different characters?
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
fn parse_node_header(text: &String) -> (String, String) {
	let mut splitter = text.splitn(2, " ");

	// Spool past the name of the node, e.g "sine", "out", as the node is already identified
	let name = splitter.next().unwrap();

	let id = splitter.next().expect("The node should have an id set by now").trim().to_owned();

	(name.to_string(), id.to_string())
}


fn parse_node(module: &mut module::Module, indent_block: &mut parse::IndentBlock) {
	let (name, id) = parse_node_header(&indent_block.text);

	let mut ports = nodes::common::Ports::new();

	let node = match name.as_str() {
		"sine" => { Some(nodes::sine::new(indent_block, &mut ports)) }
		"out" => { Some(nodes::out::new(indent_block, &mut ports)) }
		_ => {
			indent_block.text.push_str("  # ERROR: Unknown node");
			None
		}
	};

	assert!(!module.nodes.contains_key(&id), "ID {} has already been used", id);
	module.nodes.insert(id.clone(), node);
	module.ports.insert(id, ports);
}


/// Parse a parameter line that can be connected to an outlet of another node
pub fn parse_node_parameter(text: &str) -> Result<PortParameter, String> {
	let mut splitter = text.trim().split(" ");
	let name = splitter.next().unwrap().to_string();

	match splitter.next() {
		Some("<-") => {
			match splitter.next() {
				Some(source) => {
					match source.split_once(":") {
						Some((node_id, outlet)) => {
							Ok(PortParameter::Inlet {name: name, node_id: node_id.to_string(), outlet: outlet.to_string()})
						}
						None => {
							Err("  # ERROR: Invalid node:port reference".to_string())
						}
					}
				}
				_ => {
					Err("  # ERROR: Missing id of the connecting node".to_string())
				}
			}
		}
		Some("->") => {
			match splitter.next() {
				Some(node_id) => {
					match splitter.next() {
						Some(inlet) => {
							Ok(PortParameter::Outlet {name: name, node_id: node_id.to_string(), inlet: inlet.trim().to_string()})
						}
						_ => {
							Err("  # ERROR: Missing name of port of the connecting node".to_string())
						}
					}
				}
				_ => {
					Err("  # ERROR: Missing id of the connecting node".to_string())
				}
			}
		}
		Some(v) => {
			Ok(
				PortParameter::Constant {
					name: name,
					value: (
						v.to_string() +
						" " +
						splitter.filter(|x|x.len()>1).map(|x|x.to_string() + " ").collect::<String>().as_str()).trim().to_string()
					}
				)
		}
		None => {
			Err("  # ERROR: Parameter is missing value".to_string())
		}
	}
}


/// Parse a module, e.g main.aim, verify, autocomplete and return changed text.
///
/// Will return error messages back into the file.
pub fn parse_module_text(raw_text: &str) -> (module::Module, String) {
	let text = initialize_nodes(raw_text);
	let owned_text = text.to_owned();
	let mut top_indent_block = parse::IndentBlock::parse_text(&owned_text);

	let mut module = module::Module::new();

	// Iterate each top level, which represents node headers (name + id)
	for indent_block in &mut top_indent_block.children {
		parse_node(&mut module, indent_block);
	}

	(module, top_indent_block.to_string())
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
		assert_eq!(highest, 12);
	}

	#[test]
	fn getting_node_ids() {
		let ids = get_all_node_ids("
a id1
	b id2
c
d id3
		".trim());

		assert_eq!(ids, vec!["id1".to_string(), "id3".to_string()])
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

		assert_eq!(result, "
a id10
	b
c id5
	d id1337
e id9
		".trim());
	}

	#[test]
	fn parsing_text() {
		let (result, text) = parse_module_text("
sine id0
	frequency 440
doesnotexist id1
	amplitude 1
		".trim());

		assert_eq!(result.nodes.len(), 2);
		assert_eq!(text, "
sine id0
	frequency 440
doesnotexist id1  # ERROR: Unknown node
	amplitude 1
		".trim());
	}

	#[test]
	fn unknown_node() {
		let (result, text) = parse_module_text("
lolwat id0
	lolparameter 1337
".trim());

		assert_eq!(result.nodes.len(), 1);
		assert_eq!(text, "
lolwat id0  # ERROR: Unknown node
	lolparameter 1337
		".trim());
	}

	#[test]
	fn sine_unknown_parameter() {
		let (result, text) = parse_module_text("
sine id0
	lolparameter 1337
		".trim());

		assert_eq!(result.nodes.len(), 1);
		assert_eq!(text, "
sine id0
	lolparameter 1337  # ERROR: Unknown parameter
		".trim());
	}

	#[test]
	fn parsing_input_parameter() {
		let result = parse_node_parameter("frequency 440 test 123");

		match result {
			Ok(PortParameter::Constant {name, value}) => {
				assert_eq!(name, "frequency");
				assert_eq!("440 test 123", value);
			}
			Err(message) => {
				panic!("{}", message);
			}
			_ => {
				panic!();
			}
		}

		let result = parse_node_parameter("frequency <- id1:out");
		match result {
			Ok(PortParameter::Inlet {name, node_id, outlet}) => {
				assert_eq!(name, "frequency");
				assert_eq!(node_id, "id1");
				assert_eq!(outlet, "out");
			}
			Err(message) => {
				panic!("{}", message);
			}
			_ => {
				panic!();
			}
		}

		let result = parse_node_parameter("frequency -> id1 in");
		match result {
			Ok(PortParameter::Outlet {name, node_id, inlet}) => {
				assert_eq!(name, "frequency");
				assert_eq!(node_id, "id1");
				assert_eq!(inlet, "in");
			}
			Err(message) => {
				panic!("{}", message);
			}
			_ => {
				panic!();
			}
		}
	}
}
