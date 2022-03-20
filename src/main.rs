//mod nodes;
//mod audio_output;
mod parse;

//use crate::nodes::sine::{SineNode};
//use crate::nodes::{Node, NodeData};

//use crate::audio_output;


fn main() {
	parse::parse_module("example_project/main.txt");
}
