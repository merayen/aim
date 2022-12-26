use std::collections::HashMap;
use crate::nodes::common::{ProcessNode, ProcessNodeEnvironment, Ports};
use crate::module;

/// Process a single frame
///
/// Execute all the nodes in a module.
pub fn process_frame(env: &ProcessNodeEnvironment, module: &mut module::Module) {
	// TODO merayen need to process each group nodes too

	if module.nodes.len() != module.execution_order.len() {
		// Forgotten to call `plan_execution_order`?
		panic!("Length of execution order list and nodes in module is different");
	}

	// Do the processing work
	for node_id in module.execution_order.iter() {

		// Get the node, unwrap() as we are really sure it should be there
		// (otherwise there is something very wrong)
		let node = module.nodes.get_mut(node_id).unwrap();
		match node {
			Some(process_node) => {
				process_node.on_process(
					env,
					&mut module.ports,
				);
			}
			None => {
				// Node wasn't possible to parse or is unknown. Ignore it
			}
		}
	}

	// Now query all the nodes for any voices that we can delete
	let mut holds_voice = false;
	for (id, node) in module.nodes.iter() {
		match node {
			Some(process_node) => {
				// TODO merayen
			}
			None => {
			}
		}
	}
}

// TODO merayen move crate::process into this module
