//! Parses the specialized files that makes up the node projects


pub struct TextLine {
	pub text: String,
	pub indent_level: u16,
	pub line_number: usize,
}

pub fn parse_module(text: &str) -> Vec<TextLine> {
	//let text = std::fs::read_to_string(path).expect(format!("Could not read {}", path).as_str()).to_string();

	let mut text_lines = Vec::new();
	let mut stack: Vec<i32> = Vec::new();
	let mut indent_level = 0;

	for (line_number, raw_line) in (&text).split("\n").enumerate() {
		let line = raw_line.splitn(2, "#").next().unwrap();
		if line.trim().len() == 0 {
			continue;
		}

		let new_indent_level = get_indent_level(line);

		if new_indent_level > indent_level {
			stack.push((text_lines.len() as i32) - 1);
		} else if new_indent_level < indent_level {
			stack.pop();
		}

		text_lines.push(TextLine {text: line.trim().to_string(), indent_level: new_indent_level, line_number: line_number + 1});
		indent_level = new_indent_level;
	}

	text_lines
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

/// Get the lines for one "indent group"
/// ```
/// a
/// 	b
/// 	c
/// 		d
/// 	e
/// b
/// ```
/// ...will yield `a, b, c, d, e`.
pub fn get_indent_lines(lines: &[TextLine]) -> &[TextLine] {
	&lines[..get_indent_line_count(lines)]
}

/// Find out how many lines this indent level and its children is. E.g, here we would return 4:
/// ```
/// sine
///     property 1
///     property 2
///         subproperty
/// out
/// ```
pub fn get_indent_line_count(lines: &[TextLine]) -> usize {
	lines
		.iter()
		.position(|x| {
			x.indent_level == lines[0].indent_level && !std::ptr::eq(&lines[0], x)
		})
		.unwrap_or(
			lines.len()
		)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn indent_level() {
		assert!(get_indent_level("	hei du") == 1);
	}

	#[test]
	fn parse_a_module() {
		let a = parse_module("
Top 1
	Child 1
		Child Child 1
	Child 2
Top 2
		Child 2
");
		println!("{}", a.len());
		assert!(a.len() == 6);

		assert!(a[0].text == "Top 1");
		assert!(a[1].text == "Child 1");
		assert!(a[2].text == "Child Child 1");
		assert!(a[3].text == "Child 2");
		assert!(a[4].text == "Top 2");
		assert!(a[5].text == "Child 2");

		assert!(a[0].indent_level == 0);
		assert!(a[1].indent_level == 1);
		assert!(a[2].indent_level == 2);
		assert!(a[3].indent_level == 1);
		assert!(a[4].indent_level == 0);
		assert!(a[5].indent_level == 2);
	}

	#[test]
	fn indent_line_count() {
		let lines = parse_module("
a
	b
	c
		d
	e
f
		");
		assert!(get_indent_line_count(&lines) == 5);

		{
			let result = get_indent_lines(&lines);
			assert!(result.len() == 5);
		}
	}

	#[test]
	fn indent_line_count_end_of_file() {
		let lines = parse_module("
a
		");
		assert!(get_indent_line_count(&lines) == 1)
	}

	#[test]
	fn indent_line_count_no_properties() {
		let lines = parse_module("
a
b
		");
		assert!(get_indent_line_count(&lines) == 1)
	}

	#[test]
	fn ignore_comments() {
		let lines = parse_module("
a # This is ignore, # and this
		".trim());

		assert!(lines.len() == 1);
		println!("lines[0].text={:?}", lines[0].text);
		assert!(lines[0].text == "a");
	}
}
