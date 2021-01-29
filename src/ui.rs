pub mod input;
pub mod draw;
pub mod pane;

// UI
pub struct UI {
	pane: pane::Pane,
	input: input::Input,
}

impl UI {
	pub fn new() -> UI {
		UI {
			pane: pane::Pane::new(100, 30),
			input: input::Input::new(),
		}
	}

	pub fn start(&mut self) {
		for x in 0..10 {
			draw::text(&mut self.pane, 0, 0, "Hei");
			self.pane.draw(0, 0);
		}
	}
}
