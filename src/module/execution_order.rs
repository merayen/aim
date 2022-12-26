//! Calculate in which order to execute the nodes
use crate::nodes::common::{ProcessNode, Ports};
use crate::project;
use crate::module;
use std::collections::{HashMap, HashSet};

/// Plans in which order nodes in a module should be executed
///
/// Returns a list of node ids in which order they should be executed.
///
/// When node A needs the output of node B, node B must always execute before
/// node A is executed.
pub fn plan_execution_order(module: &mut module::Module) {
	let mut dependencies: HashMap<String, HashSet<String>> = HashMap::new();

	// Collect all nodes and put them into a simplified dependency list
	for (node_id, ports) in &module.ports { // TODO merayen what about unconnected nodes? They won't be registered here, or...?
		let mut node_dependencies = HashSet::new();
		for (port_name, inlet) in &ports.inlets {
			
			// Check if the inlet is connected
			if let Some(inlet) = inlet {

				// Add the left node as a dependency for this node
				node_dependencies.insert(inlet.node_id.to_string());
			}
		}
		dependencies.insert(node_id.to_string(), node_dependencies);
	}


	// Now iterate through the simplified dependency list and put those into the
	// module's execution order.

	module.execution_order.clear();

	while dependencies.len() > 0 {
		let last_length = dependencies.len();

		// Go through all the remaining nodes and see if anyone have their
		// dependencies satisfied.
		for (node_id, node_dependencies) in dependencies.clone() {

			if node_dependencies.iter().all(|x| module.execution_order.contains(x)) {
				// All dependencies for the node has been solved, or no dependencies
				dependencies.remove(&node_id);
				module.execution_order.push(node_id);
			}
		}

		if last_length == dependencies.len() {
			// We are stuck. Most likely due to nodes being connected in a loop,
			// which is not something we support.
			panic!("Loop detected");
		}
	}
}

mod tests {
	use super::*;
	use crate::nodes::sine::SineNode;
	use crate::parse_nodes;

	#[test]
	fn check_execution_order_of_nodes() {
		let (mut module, _) = parse_nodes::parse_module_text("
sine id1
	frequency 440
sine id2
	frequency <- id1:out
sine id3
	frequency <- id1:out
sine id4
	frequency <- id2:out
sine id5
	frequency 440
		".trim());

		assert!(!module.is_ok());

		plan_execution_order(&mut module);

		assert!(module.execution_order.len() == 5, "Nodes are missing from the execution order");

		assert!(
			vec!["id1","id2","id3","id4","id5"].iter().all(
				|x| module.execution_order.contains(&x.to_string())
			),
			"Nodes are missing in the execution order list"
		);

		assert!(
			module.execution_order.iter().position(|x| x == "id2")
			>
			module.execution_order.iter().position(|x| x == "id1")
		);

		assert!(
			module.execution_order.iter().position(|x| x == "id3")
			>
			module.execution_order.iter().position(|x| x == "id1")
		);

		assert!(
			module.execution_order.iter().position(|x| x == "id4")
			>
			module.execution_order.iter().position(|x| x == "id2")
		);
	}
}
