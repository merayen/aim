//! Calculating order to execute nodes
//!
//! Not fully implemented yet as we executes nodes randomly until we get
//! performance issues.
use crate::nodes::common::{ProcessNode, Ports};
use crate::project;
use crate::module;
use std::collections::{HashMap, HashSet};

/// Plans in which order nodes in a module should be executed
///
/// Returns the indexes in the input array that the nodes should be executed.
pub fn plan_execution_order(module: &mut module::Module) {
	assert_eq!(module.execution_order.len(), 0);

	// Find all the right-most nodes (e.g module outputs)
	// They will be executed last.
	let mut right_most = Vec::new();
	for (node_id, ports) in &module.ports {
		if ports.outlets.len() == 0 {
			right_most.push(node_id);
		}
	}

	// Add them to the ordered list.
	// The first ones are now the latest one to execute.
	let mut reversed_result: Vec<String> = Vec::new();
	for node_id in &right_most {
		reversed_result.push(node_id.to_string());
	}

	// TODO merayen check for circular dependencies, we don't support them

	// Then trace those right-most nodes to the left
	for node_id in &right_most {
		let inlets = &module.ports[node_id.clone()].inlets;
		for (left_node_id, inlet_option) in inlets {
			match inlet_option {
				Some(inlet) => {
					let index = reversed_result.iter().position(|x| &x == node_id).unwrap();
					println!("{}", index);
					//reversed_result.insert(index, left_node_id.clone());
				}
				None => {
				}
			}
		}
	}

	reversed_result.reverse();

	// Set the result onto the Module
	for node_id in &reversed_result {
		module.execution_order.push(node_id.clone());
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
	frequency 440
		".trim());

		assert!(!module.is_ok());

		plan_execution_order(&mut module);

		assert_eq!(module.execution_order, vec!["id1", "id2", "id3", "id4"]);
	}
}
