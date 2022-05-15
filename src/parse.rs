pub struct IndentBlock {
	pub text: String,
	pub children: Vec<IndentBlock>,

	/// Internal variable. The total amount of children, deeply.
	///
	/// It will go out of sync when adding or removing elements. Not to be used outside IndentBlock.
	deep_count: usize,

	line_number: usize,
}

impl IndentBlock {
	pub fn parse_text(text: &str) -> IndentBlock {
		assert!(text.trim().len() > 0);

		// TODO merayen remove empty lines
		let lines: Vec<String> = text.split("\n").map(str::to_string).collect();

		let children = IndentBlock::parse_children(&lines[..], 0, 1);

		IndentBlock {
			text: "".to_string(),
			children: children,
			deep_count: lines.len(),
			line_number: 0,
		}
	}

	fn parse_children(lines: &[String], indent_level: u16, line_number: usize) -> Vec<IndentBlock> {
		if lines.len() == 0 {
			return vec![];
		}

		let actual_indent_level = get_indent_level(lines[0].as_str());
		if actual_indent_level > indent_level {
			panic!("Element does not have a direct parent at line number {}", line_number);
		} else if actual_indent_level < indent_level {
			return vec![]; // No elements at this indentation level
		}

		let mut result: Vec<IndentBlock> = Vec::new();

		let mut i = 0;
		while i < lines.len() {
			if lines[i].trim().len() == 0 {
				// Empty line, ignore it
				continue;
			}

			let new_indent_level = get_indent_level(lines[i].as_str());

			if new_indent_level < indent_level {
				break; // Leaving our indentation level, we are done processing
			}

			let line = lines[i].trim().to_string();

			if new_indent_level > indent_level {
				panic!("Element missing parent detected at line number {}: {}", line_number + i, line);
			}

			let children = IndentBlock::parse_children(&lines[i+1..], indent_level + 1, line_number + i + 1);

			let mut deep_count: usize = children.len();
			for c in &children {
				deep_count += c.deep_count;
			}

			result.push(
				IndentBlock {
					text: line,
					children: children,
					deep_count: deep_count,
					line_number: line_number + i,
				}
			);

			// Nested elements has been parsed by recursion. Move our pointer forward.
			i += deep_count + 1;
		}

		result
	}

	fn children_to_string(&self, result: &mut String, indent_level: usize) {
		for x in &self.children {
			result.push_str(&"\t".repeat(indent_level));
			result.push_str(x.text.as_str());
			result.push_str("\n");

			IndentBlock::children_to_string(&x, result, indent_level + 1);
		}
	}

	pub fn to_string(&self) -> String {
		let mut result = String::with_capacity(self.deep_count * 20);  // 20 is perhaps average line size, no idea

		IndentBlock::children_to_string(self, &mut result, 0);

		result.trim().to_string()
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
	fn parse_simple_text() {
		let text = "
a
	b
	c
		d
	e
		f
g
		".trim();

		let result = IndentBlock::parse_text(text);
		
		// Top level
		assert!(result.deep_count == 7);
		assert!(result.text == "");
		assert!(result.children.len() == 2);
		assert!(result.line_number == 0);

		// "a"
		assert!(result.children[0].deep_count == 5);
		assert!(result.children[0].text == "a");
		assert!(result.children[0].children.len() == 3);
		assert!(result.children[0].line_number == 1);

		// "b"
		assert!(result.children[0].children[0].deep_count == 0);
		assert!(result.children[0].children[0].text == "b");
		assert!(result.children[0].children[0].children.len() == 0);
		assert!(result.children[0].children[0].line_number == 2);

		// "c"
		assert!(result.children[0].children[1].deep_count == 1);
		assert!(result.children[0].children[1].text == "c");
		assert!(result.children[0].children[1].children.len() == 1);
		assert!(result.children[0].children[1].line_number == 3);

		// "d"
		assert!(result.children[0].children[1].children[0].deep_count == 0);
		assert!(result.children[0].children[1].children[0].text == "d");
		assert!(result.children[0].children[1].children[0].children.len() == 0);
		assert!(result.children[0].children[1].children[0].line_number == 4);

		// "e"
		assert!(result.children[0].children[2].deep_count == 1);
		assert!(result.children[0].children[2].text == "e");
		assert!(result.children[0].children[2].children.len() == 1);
		assert!(result.children[0].children[2].line_number == 5);

		// "f"
		assert!(result.children[0].children[2].children[0].deep_count == 0);
		assert!(result.children[0].children[2].children[0].text == "f");
		assert!(result.children[0].children[2].children[0].children.len() == 0);
		assert!(result.children[0].children[2].children[0].line_number == 6);

		// "g"
		assert!(result.children[1].deep_count == 0);
		assert!(result.children[1].text == "g");
		assert!(result.children[1].children.len() == 0);
		assert!(result.children[1].line_number == 7);

		assert!(result.to_string() == text);
	}
}
