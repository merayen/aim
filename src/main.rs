//mod audio_output;
mod nodes;
mod parse;
mod parse_nodes;
mod module;
mod project;

fn main() {
	project::run("./");
}
