//! Calculating order to execute nodes
//!
//! Not fully implemented yet as we executes nodes randomly until we get
//! performance issues.
use crate::nodes::common::{ProcessNode, Ports};
use crate::project;
use std::collections::{HashMap, HashSet};

/// Plans in which order nodes in a module should be executed
///
/// Returns the indexes in the input array that the nodes should be executed.
pub fn plan_execution_order<'a>(nodes: &'a HashMap<String, Option<Box<dyn ProcessNode>>>, ports: &HashMap<String, Ports>) -> Vec<&'a String> {
	let mut result: Vec<&String> = Vec::with_capacity(nodes.len());

	let mut remaining_nodes: HashSet<String> = nodes.keys().cloned().collect();

	// Find nodes that are not connected
	let mut not_connected_nodes: Vec<&String> = Vec::new();
	let mut generators: Vec<&String> = Vec::new();
	for (id, node) in nodes {
		let ports = &ports[id];

		if ports.inlets.len() > 0 {
			// A node dependent on other ones.
			// We will handle those below.
			continue;
		}

		if ports.outlets.len() > 0 {
			// Generator node
			generators.push(id);
		} else {
			// Node not connected to anything
			not_connected_nodes.push(id);
		}
		remaining_nodes.remove(id);
	}

	// Follow the generator nodes
	for id in generators {
		println!("{}", id);
	}

	result.extend(not_connected_nodes.iter());
	result
}

mod tests {
	use super::*;
	use crate::nodes::sine::SineNode;
	use crate::parse_nodes;

	#[test]
	#[ignore]
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

		assert_eq!(result, vec!["id1", "id2", "id3", "id4"]);
	}
}
