//mod audio_output;
mod nodes;
mod parse;
mod parse_nodes;
mod module;
mod project;
mod process;
mod audio_output;

fn main() {
	project::run("./");
}
