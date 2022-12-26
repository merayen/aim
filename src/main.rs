//mod audio_output;
mod nodes;
mod parse;
mod parse_nodes;
mod module;
mod project;
mod process;
mod audio_output;

fn main() {
	// TODO merayen parse command line arguments

	let path = std::env::current_dir().unwrap();
	project::run(path.to_str().unwrap());
}
