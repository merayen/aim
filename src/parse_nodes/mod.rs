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
	result.lines.push(
		parse::TextLine {
			text: text.to_string(),
			line_number: result.lines.len() + 1,
			indent_level: line.indent_level,
		}
	);
}

/// Forward a line without any modifications
fn write_forward(result: &mut ParseResults, line: &parse::TextLine) {
	result.lines.push(
		parse::TextLine {
			text: line.text.to_owned(),
			line_number: result.lines.len() + 1,
			indent_level: line.indent_level,
		}
	);
}

/// Add a new line, as a child
fn write_child_line(result: &mut ParseResults, line: &parse::TextLine, text: &str) {
	result.lines.push(parse::TextLine {
		text: text.to_string(),
		line_number: result.lines.len() + 1,
		indent_level: line.indent_level + 1,
	});
}

fn read_property(line: &parse::TextLine) -> (String, String) {
	let mut iter = line.text.splitn(2, " ");

	(iter.next().unwrap().to_string(), iter.next().unwrap_or("").to_string())
}

#[derive(Debug)]
enum PeekerResult<'a> {
	/// Next indentation if deeper
	/// E.g:
	/// ```
	/// a
	/// 	b  # Our current position
	/// ```
	IndentDown(&'a parse::TextLine),

	/// Next indentation is higher.
	/// E.g:
	/// ```
	/// a
	/// 	b
	/// c  # Our current position
	/// ```
	IndentUp(&'a parse::TextLine),

	/// Next line is on the same indentation.
	/// E.g:
	/// ```
	/// a
	/// 	b
	/// 	c  # Our current position
	/// ```
	IndentSame(&'a parse::TextLine),

	/// There are no items left.
	Done,
}

struct Peeker {
	lines: Vec<parse::TextLine>,
	index: usize,
	indent_level: u16,
	protection: u32,
}

impl Peeker {
	fn new(lines: Vec<parse::TextLine>) -> Peeker {
		Peeker {
			lines: lines,
			indent_level: 0,
			index: 0,
			protection: 0,
		}
	}

	/// Get the current line
	fn current(&mut self) -> PeekerResult {
		if self.protection > 1000 {
			panic!("Stuck? current() called many times without consumption");
		}

		self.protection += 1;

		if self.index >= self.lines.len() {
			return PeekerResult::Done;
		}

		let line = &self.lines[self.index];

		if line.indent_level < self.indent_level {
			return PeekerResult::IndentUp(&line);
		}
		else if line.indent_level == self.indent_level {
			return PeekerResult::IndentSame(&line);
		}
		else if line.indent_level > self.indent_level {
			return PeekerResult::IndentDown(&line);
		}
		else {
			panic!("Should not happen");
		}
	}

	/// Consume current line
	fn consume(&mut self, result: &mut ParseResults) {
		if self.index >= self.lines.len() {
			panic!("Peeker is already consumed");
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
		let line = match Peeker::current(self) {
			PeekerResult::IndentDown(line) | PeekerResult::IndentUp(line) | PeekerResult::IndentSame(line) => {
				line
			}
			PeekerResult::Done => {
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
			match Peeker::current(self) {
				PeekerResult::IndentUp(v) | PeekerResult::IndentDown(v) | PeekerResult::IndentSame(v) => {
					if v.indent_level <= indent_level {
						println!("consume_with_error Indentation done");
						break;
					}

					// Write the line to the output
					println!("consume_with_error text={}", v.text);
					Peeker::consume(self, result);
				}
				PeekerResult::Done => {
					println!("consume_with_error Done");
					break;  // We hit the end
				}
			}
		}
	}
}

fn parse_node(result: &mut ParseResults, peeker: &mut Peeker) { // TODO probably move the nodes out somewhere
	match peeker.current() {
		PeekerResult::IndentSame(title_line) | PeekerResult::IndentUp(title_line) | PeekerResult::IndentSame(title_line) => {
			match title_line.text.splitn(2, " ").next().unwrap() {
				"sine" => {
					peeker.consume(result);  // Consume the node header, "sine IDabc"

					// Parse properties
					loop {
						match peeker.current() {
							PeekerResult::IndentDown(line) | PeekerResult::IndentSame(line) => {
								match line.text.splitn(2, " ").next().unwrap() {
									"frequency" => {
										peeker.consume(result);
									}
									_ => {
										peeker.consume_with_error(result, "Unknown property");
										println!("Did consume with error");
									}
								}
							}
							_ => {
								break;
							}  // Nothing more for us to process
						}
					}
				}
				_ => {
					println!("Node is unknown");
					peeker.consume_with_error(result, "Unknown node");
				}
			}
		}
		PeekerResult::IndentDown(line) => {
			panic!("Should not happen");
		}
		PeekerResult::Done => {
		}
	}
}

/// Parse a module, e.g main.txt, verify, autocomplete and return changed text.
/// Will return error messages back into the file.
pub fn parse_module_text(text: &str) -> ParseResults {
	let parsed: Vec<parse::TextLine> = parse::parse_module(text);
	let mut peeker = Peeker::new(parsed);

	let mut result = ParseResults::new();

	loop {
		match peeker.current() {
			PeekerResult::IndentSame(line) | PeekerResult::IndentDown(line) | PeekerResult::IndentUp(line) => {
				if line.indent_level > 0 {
					panic!("Node did not consume all its data? At index: {}, text: {}", peeker.index, peeker.lines[peeker.index].text);
				}

				parse_node(&mut result, &mut peeker);
			}
			PeekerResult::Done => {
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
		let mut peeker = Peeker::new(parsed);
		let mut result = ParseResults::new();
		peeker.consume_with_error(&mut result, "Not working");
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
