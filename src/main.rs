//mod audio_output;
mod nodes;
mod parse;
mod parse_nodes;
mod process;
mod project;
mod execution_order;

fn main() {
	project::run("./");
}
