use crate::dsp::DSP;
use crate::ui;

// The main application


pub struct Aim {
	dsp: DSP,
	ui_thread: std::thread::JoinHandle<()>,
}

impl Aim {
	pub fn new() -> Aim {
		let ui_thread = std::thread::spawn(|| {
			let mut ui = ui::UI::new();
			ui.start();
		});

		Aim {
			dsp: DSP::new(),
			ui_thread: ui_thread,
		}
	}

	pub fn join(self) {
		self.ui_thread.join();
	}
}
