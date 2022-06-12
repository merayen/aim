pub mod sine;
pub mod module;
pub mod common;

/// All nodes should implement this trait
pub trait Node {
	// TODO merayen delete?
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
