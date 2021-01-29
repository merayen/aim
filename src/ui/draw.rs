use crate::ui::pane::Pane;

pub fn rect(pane: &mut Pane, x: usize, y: usize, width: usize, height: usize) {
	for i in x..x+width { // Top and bottom line
		pane.bg[y * pane.width + i] = pane.bg_current;
		pane.bg[(y + height - 1) * pane.width + i] = pane.bg_current;
	}

	for i in y+1..std::cmp::min(y+height, pane.height) { // Left and right line
		pane.bg[i * pane.width + x] = pane.bg_current;
		pane.bg[i * pane.width + x + width - 1] = pane.bg_current;
	}
}

pub fn text(pane: &mut Pane, x: usize, y: usize, text: &str) {
	let mut x_pos: usize = x;
	let mut y_pos: usize = y;

	for c in text.chars() {
			if c == '\n' {
				y_pos += 1;
				x_pos = x;
				continue;
			}

			if x_pos < pane.width && y_pos < pane.height {
				let pos = x_pos + y_pos * pane.width;
				pane.text[pos] = c;
				pane.fg[pos] = pane.fg_current;
				pane.bg[pos] = pane.bg_current;
			}

			x_pos += 1;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	fn create_pane() -> Pane {
		Pane::new(100, 30)
	}

	#[test]
	fn test_rect() {
		let mut pane = create_pane();
		pane.bg_current = 10;
		rect(&mut pane, 0, 0, 10, 10);
		pane.draw();
		std::thread::sleep(std::time::Duration::from_secs(1));
	}
}
