//! Parsing of nodes from text files that looks like:
//!
//! ```
//! sine
//!   frequency 440
//! ```

use crate::parse;

/// A node that parses its .txt-file
trait ParseNode {
	fn parse(lines: Vec<parse::TextLine>);
}

pub struct ParseResults {
	/// Lines that should replace the content of the module that was parsed.
	/// These lines will contain errors too, as comments.
	lines: Vec<parse::TextLine>,

	/// Errors that are shown in the stdout of the synth
	errors: Vec<String>,
}

impl ParseResults {
	fn new() -> ParseResults {
		ParseResults {
			lines: Vec::new(),
			errors: Vec::new(),
		}
	}
}

/// Add a new line, based on previous line
fn write_line(result: &mut ParseResults, line: &parse::TextLine, text: &str) {
}

/// Add a new line, as a child
fn write_child_line(result: &mut ParseResults, line: &parse::TextLine, text: &str) {
	result.lines.push(parse::TextLine {
		text: text.to_string(),
		line_number: result.lines.len() + 1,
		indent_level: line.indent_level + 1,
	});
}

/// Write an existing line appending " # ERROR: ..."
fn write_error(result: &mut ParseResults, line: &parse::TextLine, description: &str) {
	result.lines.push(
		parse::TextLine {
			text: line.text.to_owned() + " # ERROR: " + description,
			line_number: result.lines.len() + 1,
			indent_level: line.indent_level,
		}
	);
}

fn node_sine(result: &mut ParseResults, lines: &mut std::iter::Peekable<std::slice::Iter<parse::TextLine>>) {
	loop {
		match &lines.next() {
			Some(line) => {
				match line.text.as_str() {
					"frequency" => {
						println!("{}", "Got frequency");
					}
					_ => {
						write_error(result, line, "Unknown property");
					}
				}
			}
			None => {
				break; // Done
			}
		}
	}
}

/// Parse a module, e.g main.txt, verify, autocomplete and return changed text.
/// Will return error messages back into the file.
pub fn parse_module_text(text: &str) -> ParseResults {
	let parsed =  parse::parse_module(text);
	let mut lines = parsed.iter().peekable();

	let mut result = ParseResults::new();

	loop {
		match lines.next() {
			Some(line) => {
				if line.indent_level > 0 {
					write_error(&mut result, line, "ERROR: Expected no indentation here");
					continue;
				}

				// Processing a new node
				match line.text.as_str() {
					"sine" => {
						node_sine(&mut result, &mut lines);
					}
					_ => {
						write_error(&mut result, line, format!("Unknown node at line {}", line.line_number).as_str());
					}
				}
			}
			None => {
				break; // Done
			}
		}
	}

	result
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing_text() {
		let result = parse_module_text("
sine
	frequency 440
out
	amplitude 1
		".trim());

		assert!(result.lines.len() == 4);

		// TODO merayen finish test
	}

	#[test]
	fn unknown_node() {
		let result = parse_module_text("
lolwat
	lolproperty 1337
".trim());
		assert!(result.lines.len() == 2);

		// TODO merayen finish test
	}

	#[test]
	fn sine_unknown_property() {
		let result = parse_module_text("
sine
	lolproperty 1337
		".trim());

		assert!(result.lines.len() == 2);

		// TODO merayen finish test
	}
}
