//! Parses the specialized files that makes up the node projects


pub struct TextLine {
	text: String,
	indent: u16,
}

pub fn parse_module(text: &str) -> Vec<TextLine> {
	//let text = std::fs::read_to_string(path).expect(format!("Could not read {}", path).as_str()).to_string();

	let mut text_lines = Vec::new();
	let mut stack: Vec<i32> = Vec::new();
	let mut indent_level = 0;

	for line in (&text).split("\n") {
		if line.trim().len() == 0 {
			continue;
		}

		let new_indent_level = get_indent_level(line);

		if new_indent_level > indent_level {
			stack.push((text_lines.len() as i32) - 1);
		} else if new_indent_level < indent_level {
			stack.pop();
		}

		text_lines.push(TextLine {text: line.trim().to_string(), indent: new_indent_level});
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

		assert!(a[0].indent == 0);
		assert!(a[1].indent == 1);
		assert!(a[2].indent == 2);
		assert!(a[3].indent == 1);
		assert!(a[4].indent == 0);
		assert!(a[5].indent == 2);
	}
}
