pub struct Pane {
	pub width: usize,
	pub height: usize,
	pub text: Vec<char>,
	pub fg: Vec<u8>,
	pub bg: Vec<u8>,
	pub fg_current: u8,
	pub bg_current: u8,
}

// A pane is a buffered area that can be drawn at a certain place on the screen
impl Pane {
	pub fn new<'a>(width: usize, height: usize) -> Pane {
		println!("\x1B[?25l"); // Removes cursor
		// TODO go into RAW MODE
		Pane {
			width: width,
			height: height,
			text: vec![' '; width * height],
			fg: vec![0; width * height],
			bg: vec![0; width * height],
			fg_current: 0,
			bg_current: 0,
		}
	}

	pub fn draw(&mut self, x: usize, y: usize) { // TODO create a buffer, then print it in 1 go instead?
		let mut output: Vec<char> = Vec::with_capacity(self.width * self.height);

		fn term_clear_fg(output: &mut Vec<char>) {
			output.push('\x1B');
			output.push('[');
			output.push('3');
			output.push('8');
			output.push(';');
			output.push('5');
			output.push(';');
			output.push('1');
			output.push('5');
			output.push('m');
		};

		fn term_clear_bg(output: &mut Vec<char>) {
			output.push('\x1B');
			output.push('[');
			output.push('4');
			output.push('8');
			output.push(';');
			output.push('5');
			output.push(';');
			output.push('0');
			output.push('m');
		};


		for i in 0..self.width*self.height {
			let character = self.text[i];
			let fg = self.fg[i];
			let bg = self.bg[i];

			if i > 0 && i % self.width == 0 {
				term_clear_bg(&mut output); // So that lines on right side does not span over
				output.push('\n');
			}

			if fg > 0 { // TODO Check if same color repeats
				output.push('\x1B');
				output.push('[');
				output.push('3');
				output.push('8');
				output.push(';');
				output.push('5');
				output.push(';');
				output.push(std::char::from_digit((fg % 10) as u32, 10).unwrap());
				if fg >= 10 {
					output.push(std::char::from_digit(((fg/10) % 10) as u32, 10).unwrap());
				}
				output.push('m');
			} else {
				term_clear_fg(&mut output);
			}

			if bg > 0 {
				output.push('\x1B');
				output.push('[');
				output.push('4');
				output.push('8');
				output.push(';');
				output.push('5');
				output.push(';');
				output.push(std::char::from_digit((bg % 10) as u32, 10).unwrap());
				if bg >= 10 {
					output.push(std::char::from_digit(((bg/10) % 10) as u32, 10).unwrap());
				}
				output.push('m');
			} else {
				term_clear_bg(&mut output);
			}

			output.push(character);
		}

		let string: String = output.iter().collect();

		print!("{}", string);
		print!("\x1B[{}A\x1B[K\n", self.height);


		self.fg_current = 15;
		self.bg_current = 0;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_text() {
		let mut pane = Pane::new(100, 30);
		pane.draw();
		draw::rect(&mut pane, 0, 0, 10, 10);
	}
}
