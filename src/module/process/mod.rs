use std::collections::HashMap;
use crate::nodes::common::{ProcessNode, ProcessNodeEnvironment, Ports};
use crate::module;

/// Process a single frame
///
/// Execute all the nodes in a module.
pub fn process_frame(env: &ProcessNodeEnvironment, module: &mut module::Module) {
	// TODO merayen need to process each group nodes too

	// Do the processing work
	for (id, node) in module.nodes.iter_mut() { // TODO merayen need to execute the nodes in correct order here
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
