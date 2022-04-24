//mod audio_output;
mod nodes;
mod parse;
mod parse_nodes;

fn main() {
	parse::parse_module("example_project/main.txt");
	//process::test_process();
}
