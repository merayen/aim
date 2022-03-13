pub mod sine;

use std::collections::HashMap;

/// All nodes should implement this trait
pub trait Node {
	fn process();

	fn get_nodedata(&self) -> NodeData;
}

/// Common data for all nodes
#[derive(Copy, Clone)]
pub struct NodeData {
	pub id: i64,
	//pub inlets: HashMap<String, String>,
}

// All nodes must implement for parsing and dumping UI file
pub trait UINode {
	fn parse(text: String);
}
