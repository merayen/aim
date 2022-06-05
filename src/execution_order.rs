use crate::nodes::common::{ProcessNode, Ports};
use crate::project;
use std::collections::{HashMap, LinkedList};

/// Plans in which order nodes in a module should be executed
///
/// Returns the indexes in the input array that the nodes should be executed.
pub fn plan_execution_order(nodes: &HashMap<String, Option<Box<dyn ProcessNode>>>, ports: &HashMap<String, Ports>) -> Vec<usize> {
	let mut result: Vec<usize> = Vec::new();

	let mut remaining_nodes: LinkedList<String> = nodes.keys().cloned().collect();

	// Find nodes that are not connected
	let mut not_connected_nodes: Vec<String> = Vec::new();
	for (id, node) in nodes {}

	// Find the left-most nodes
	let mut left_most_nodes: Vec<String> = Vec::new();
	for (id, node) in nodes {}

	// Follow from the left-most nodes
	let mut left_most_nodes: Vec<String> = Vec::new();
	for (id, node) in nodes {}

	result
}

mod tests {
	use super::*;
	use crate::nodes::sine::SineNode;
	use crate::parse_nodes;

	#[test]
	fn check_execution_order_of_nodes() {
		let (parse_results, something) = parse_nodes::parse_module_text("
sine id1
	frequency 440
sine id2
	frequency <- id1 out
sine id3
	frequency <- id1 out
sine id4
	frequency 440
		".trim());

		let mut nodes: HashMap<String, Option<Box<dyn ProcessNode>>> = parse_results.nodes;

		let (env, mut ports) = project::initialize_nodes(&mut nodes);

		for node in nodes.values() {}

		let result = plan_execution_order(&nodes, &ports);
	}
}
