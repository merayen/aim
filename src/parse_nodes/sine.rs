use crate::parse_nodes::{ParseResults, Peeker, PeekerResult};

pub fn parse(result: &mut ParseResults, peeker: &mut Peeker) {
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
