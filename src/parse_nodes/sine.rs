use crate::parse_nodes::{ParseResults, Peeker};
use crate::parse::TextLine;

pub fn parse(result: &mut ParseResults, peeker: &mut Peeker) {
	let indent_level = peeker.current().unwrap().indent_level + 1;

	peeker.consume(result);  // Consume the node header, "sine IDabc"

	let mut frequency = 440f32;

	// Parse properties
	loop {
		match peeker.current() {
			Some(line) => {
				if line.indent_level < indent_level {
					break;  // We are done on our level
				}

				match line.text.splitn(2, " ").next().unwrap() {
					"frequency" => {
						//frequency = line.text.splitn(2, " ").next().next().unwrap().parse::<f32>().unwrap();
						peeker.consume(result);
					}
					_ => {
						peeker.consume_with_error(result, "Unknown property");
					}
				}
			}
			None => {
				break;  // Nothing more for us to process
			}
		}
	}
}
