//! Parsing of nodes from text files that looks like:
//!
//! ```
//! sine
//!   frequency 440
//! ```

use crate::parse;
mod sine;

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

pub struct TextConsumer {
	lines: Vec<parse::TextLine>,
	index: usize,
	indent_level: u16,
	protection: u32,
}

impl TextConsumer {
	fn new(lines: Vec<parse::TextLine>) -> TextConsumer {
		TextConsumer {
			lines: lines,
			indent_level: 0,
			index: 0,
			protection: 0,
		}
	}

	/// Get the current line
	fn current(&mut self) -> Option<&parse::TextLine> {
		if self.protection > 1000 {
			panic!("Stuck? current() called many times without consumption");
		}

		self.protection += 1;

		if self.index >= self.lines.len() {
			return None;
		}

		Some(&self.lines[self.index])
	}

	/// Consume current line
	fn consume(&mut self, result: &mut ParseResults) {
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
	fn consume_with_error(&mut self, result: &mut ParseResults, error_message: &str) {
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

fn parse_node(result: &mut ParseResults, text_consumer: &mut TextConsumer) {
	let title_line = text_consumer.current().unwrap();

	// Figure out which node we should pass the data to
	match title_line.text.splitn(2, " ").next().unwrap() {
		"sine" => { sine::parse(result, text_consumer); }
		_ => {
			text_consumer.consume_with_error(result, "Unknown node");
		}
	}
}

/// Parse a module, e.g main.txt, verify, autocomplete and return changed text.
/// Will return error messages back into the file.
pub fn parse_module_text(text: &str) -> ParseResults {
	let parsed: Vec<parse::TextLine> = parse::parse_module(text);
	let mut text_consumer = TextConsumer::new(parsed);

	let mut result = ParseResults::new();

	loop {
		match text_consumer.current() {
			Some(line) => {
				if line.indent_level > 0 {
					panic!("Node did not consume all its data? At index: {}, text: {}", text_consumer.index, text_consumer.lines[text_consumer.index].text);
				}

				parse_node(&mut result, &mut text_consumer);
				println!("{}", text_consumer.index);
			}
			None => {
				break;
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
		for x in &result.lines {
			println!("x.text={:?}, x.indent_level={:?}", x.text, x.indent_level);
		}
		assert!(result.lines.len() == 4);
		assert!(result.lines[0].indent_level == 0);
		assert!(result.lines[1].indent_level == 1);
		assert!(result.lines[2].indent_level == 0);
		assert!(result.lines[3].indent_level == 1);
		assert!(result.lines[0].text == "sine");
		println!("result.lines[1].text={:?}", result.lines[0].text);
		assert!(result.lines[1].text == "frequency 440");
		println!("result.lines[2].text={:?}", result.lines[2].text);
		assert!(result.lines[2].text == "out  # ERROR: Unknown node");
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
		assert!(result.lines[0].text == "a  # ERROR: Not working");
		assert!(result.lines[1].text == "b");
		assert!(result.lines[2].text == "c");
		assert!(result.lines[3].text == "d");
	}

	#[test]
	fn unknown_node() {
		let result = parse_module_text("
lolwat
	lolproperty 1337
".trim());

		assert!(result.lines.len() == 2);
		assert!(result.lines[0].indent_level == 0);
		assert!(result.lines[1].indent_level == 1);
		assert!(result.lines[0].text == "lolwat  # ERROR: Unknown node");
		assert!(result.lines[1].text == "lolproperty 1337");
	}

	#[test]
	fn sine_unknown_property() {
		let result = parse_module_text("
sine
	lolproperty 1337
		".trim());

		assert!(result.lines.len() == 2);
		assert!(result.lines[0].text == "sine");
		println!("{}", result.lines[1].text);
		assert!(result.lines[1].text == "lolproperty 1337  # ERROR: Unknown property");
	}
}
