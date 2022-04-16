//mod audio_output;
mod nodes;
mod parse;
mod process;

//use crate::nodes::sine::{SineNode};
//use crate::nodes::{Node, NodeData};

//use crate::audio_output;


fn main() {
	parse::parse_module("example_project/main.txt");
	//process::test_process();
}
