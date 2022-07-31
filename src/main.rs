//mod audio_output;
mod nodes;
mod parse;
mod parse_nodes;
mod module;
mod project;
mod process;

fn main() {
	project::run("./");
}
