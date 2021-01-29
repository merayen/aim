mod ui;
mod dsp;
mod aim;
mod interface;

fn main() {
	let mut aim = aim::Aim::new();
	aim.join();
}
