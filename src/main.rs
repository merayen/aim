//mod audio_output;
mod nodes;
mod parse;
mod parse_nodes;
mod process;
mod project;

fn main() {
	project::run("./");
}
