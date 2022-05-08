use std::collections::HashSet;

pub struct TextConsumer {
	lines: Vec<String>,
	indentations: Vec<u16>,
	index: usize,
	current_indentation: u16,
}

impl TextConsumer {
	pub fn new(text: &str) -> TextConsumer {
		assert!(text.trim().len() > 0);

		let mut lines: Vec<String> = Vec::new();
		let mut indentations: Vec<u16> = Vec::new();

		for line in text.split("\n") {
			lines.push(line.trim().to_string());
			indentations.push(get_indent_level(line));
		}

		TextConsumer {
			lines: lines,
			indentations: indentations,
			index: 0,
			current_indentation: 0,
		}
	}

	/// Expect that next element will be at one higher indentation level
	#[must_use]
	pub fn enter(&mut self) -> bool {
		if self.index + 1 < self.lines.len() {
			let next_indentation_level = self.indentations[self.index + 1];

			if next_indentation_level == self.current_indentation + 1 {
				self.current_indentation += 1;
				self.index += 1;
				return true;
			}
		}

		return false;
	}

	/// Leave the current level, down one indentation
	///
	/// If next element is two or more indentations lower, stay at same position
	/// and decrease current indentation by 1.
	///
	/// Note that current_line() will return empty string in these cases. Continue
	/// to call leave() until indentation level hits the same indentation level as
	/// the next element.
	#[must_use]
	pub fn leave(&mut self) -> bool {
		if self.current_indentation == 0 {
			// We are already on the lowest indentation level, nothing to leave
			return false;
		}

		for i in self.index + 1 .. self.lines.len() {
			if self.indentations[i] == self.current_indentation - 1 {
				// We have found an element that is one level below.
				// Jump to it directly.
				self.index = i;
				self.current_indentation -= 1;
				return true;
			} else if self.indentations[i] < self.current_indentation - 1 {
				self.index = i - 1;
				self.current_indentation -= 1;
				return true;
			}
		}

		// We didn't find any lower elements.
		// Jump to the end.
		self.index = self.lines.len() - 1;
		self.current_indentation -= 1;

		return true;
	}

	/// Adds a line of text before the current line, at the current level
	#[must_use]
	pub fn add_line(&mut self, line: String) {
		assert!(line.find("\n").is_none(), "TextConsumer::add does not allow multiple lines");

		self.lines.insert(self.index, line.trim().to_string());
		self.indentations.insert(self.index, self.current_indentation);

		self.index += 1;
	}

	/// Replace the current line
	pub fn replace_line(&mut self, line: String) {
		assert!(line.find("\n").is_none(), "TextConsumer::add does not allow multiple lines");

		self.lines[self.index] = line.trim().to_string();
	}

	/// Skip whole element including its children
	///
	/// If reaches the end, sets the index to the last item.
	/// Does not change current_indentation. You need to call leave() for that.
	pub fn next_sibling(&mut self) {
		for position in self.index + 1 .. self.lines.len() {
			if self.indentations[position] <= self.current_indentation {
				self.index = position; // Found the sibling or a lower level
				return;
			}
		}

		// No sibling found or element with lower element. Move to the end.
		self.index = self.lines.len() - 1;
	}

	/// Return the current line
	///
	/// Return empty string if no element is at that position.
	pub fn current_line(&self) -> String {
		if self.current_indentation == self.indentations[self.index] {
			// Only return text if on the same indentation level
			self.lines[self.index].trim().to_string()
		} else {
			"".to_string()
		}
	}

	/// Move cursor back to the top
	pub fn reset_cursor(&mut self) {
		self.index = 0;
		self.current_indentation = 0;
	}

	pub fn to_string(&self) -> String {
		let result = String::new();

		assert!(self.lines.len() == self.indentations.len());

		for i in 0..self.lines.len() {
			let mut line = String::new();

			if i > 0 {
				line.push_str("\n");
			}

			line.push_str(("\t".repeat(self.indentations[i] as usize)).as_str());
			line.push_str(&self.lines[i]);
		}

		result
	}

	/// Checks if current indentation level has more elements after this one
	#[must_use]
	pub fn has_next(&self) -> bool {
		for position in self.index + 1 .. self.lines.len() {
			if self.indentations[position] == self.current_indentation {
				return true;
			} else if self.indentations[position] < self.current_indentation {
				return false;
			}
		}

		return false;
	}

	pub fn current_indentation(&self) -> u16 {
		self.current_indentation
	}
}


fn get_indent_level(line: &str) -> u16 {
	let mut c = 0;
	for x in (&line).chars() {
		if x != '\t' {
			return c;
		}
		c += 1;
	}

	c
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn simple_parsing() {
		let mut text_consumer = TextConsumer::new("
a
	b
	c
		d
	e
f
		".trim());

		assert!(text_consumer.lines.len() == 6);
		assert!(text_consumer.current_line() == "a");
		assert!(text_consumer.has_next());
		assert!(!text_consumer.leave(), "Shouldn't be able to leave 'a' here as it is on the top-level, and next element is child");
		assert!(text_consumer.enter());
		assert!(text_consumer.current_line() == "b");
		text_consumer.next_sibling();
		assert!(text_consumer.current_line() == "c");
		assert!(text_consumer.enter());
		assert!(text_consumer.current_line() == "d");
		assert!(!text_consumer.enter(), "Shouldn't be able to enter non-existing level below 'd'");
		assert!(text_consumer.leave());
		assert!(text_consumer.current_line() == "e");
		assert!(!text_consumer.enter());
		assert!(text_consumer.leave());
		assert!(text_consumer.current_line() == "f");
		assert!(!text_consumer.leave());
	}

	#[test]
	fn leave_multiple_levels() {
		let mut text_consumer = TextConsumer::new("
a
	b
		c
			d
	e
f
		".trim());


		assert!(text_consumer.current_line() == "a");
		assert!(text_consumer.enter());
		assert!(text_consumer.enter());
		assert!(text_consumer.enter());
		assert!(text_consumer.current_line() == "d");
		assert!(text_consumer.current_indentation() == 3);
		assert!(text_consumer.leave());
		assert!(text_consumer.current_line() == "");
	}

	#[test]
	fn only_leave_single_indentation() {
		let mut text_consumer = TextConsumer::new("
a
	b
		c
d
		".trim());

		assert!(text_consumer.enter());
		assert!(text_consumer.enter());
		assert!(text_consumer.current_line() == "c");
		assert!(text_consumer.current_indentation() == 2);
		assert!(text_consumer.leave());
		assert!(text_consumer.current_line() == "");
		assert!(text_consumer.leave());
		assert!(text_consumer.current_line() == "d");
		assert!(!text_consumer.has_next());
	}

	#[test]
	fn adding_lines() {
		let mut text_consumer = TextConsumer::new("
a
	b
c
		".trim());

		assert!(text_consumer.lines.len() == 3);
		assert!(text_consumer.current_line() == "a");
		assert!(text_consumer.index == 0);
		text_consumer.add_line("A".to_string());
		assert!(text_consumer.current_line() == "a");
		assert!(text_consumer.index == 1);
	}

	#[test]
	fn remove_lines() {
		let mut text_consumer = TextConsumer::new("
a
	b
c
		".trim());

		assert!(text_consumer.lines.len() == 3);
		assert!(text_consumer.current_line() == "a");
		text_consumer.replace_line("A".to_string());
		assert!(text_consumer.current_line() == "A");
		assert!(text_consumer.current_indentation == 0);
		assert!(text_consumer.enter());
		assert!(text_consumer.current_line() == "b");
		text_consumer.replace_line("B".to_string());
		assert!(text_consumer.current_line() == "B");
		assert!(text_consumer.leave());
	}

	#[test]
	fn skip_whole_element() {
		let mut text_consumer = TextConsumer::new("
a
	b
c
		".trim());

		text_consumer.next_sibling();
		assert!(text_consumer.current_line() == "c");
		assert!(text_consumer.current_indentation() == 0);

		let mut text_consumer = TextConsumer::new("
a
	b
		".trim());

		assert!(text_consumer.current_line() == "a");
		assert!(text_consumer.current_indentation() == 0);
		text_consumer.next_sibling();
		assert!(text_consumer.current_line() == "");
		assert!(text_consumer.current_indentation() == 0);
	}
}
