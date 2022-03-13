use crate::nodes::{Node, NodeData, UINode};

pub struct SineNode {
	nodedata: NodeData,
	frequency: f32,
}

impl SineNode {
	pub fn new(nodedata: NodeData) -> SineNode {
		SineNode {
			nodedata: nodedata,
			frequency: 1000f32,
		}
	}
}

impl Node for SineNode {
	fn process() {}
	fn get_nodedata(&self) -> NodeData {
		self.nodedata
	}
}

impl UINode for SineNode {
	fn parse(text: String) {
		println!("Got text: {}", text);
	}
}
