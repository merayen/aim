use std::collections::HashMap;
use crate::nodes::common::{ProcessNode, ProcessNodeEnvironment, Ports};

/// Initialize all nodes for processing
///
/// Allocating buffers for the ports.
pub fn init_nodes(env: &ProcessNodeEnvironment, nodes: &mut HashMap<String, Option<Box<dyn ProcessNode>>>) -> HashMap<String, Ports> {
	let mut module_ports: HashMap<String, Ports> = HashMap::new();
	for (id, node) in nodes.iter_mut() {
		match node {
			Some(process_node) => {
				module_ports.insert(id.to_owned(), process_node.on_init(env));
			}
			None => {
				// Node wasn't possible to parse or is unknown. Ignore it
			}
		}
	}

	module_ports
}

/// Process a single frame
///
/// Execute all the nodes in a module.
pub fn process_frame(env: &ProcessNodeEnvironment, nodes: &mut HashMap<String, Option<Box<dyn ProcessNode>>>, ports: &mut HashMap<String, Ports>) {
	// TODO merayen need to process each group nodes too

	// Do the processing work
	for (id, node) in nodes.iter_mut() {
		// TODO merayen need correct execution order here
		match node {
			Some(process_node) => {
				process_node.on_process(
					env,
					&mut ports.get_mut(id).expect("ProcessNode does not have its Ports initialized. Forgotton init_nodes()?"),
				);
			}
			None => {
				// Node wasn't possible to parse or is unknown. Ignore it
			}
		}
	}

	// Now query all the nodes for any voices that we can delete
	let mut holds_voice = false;
	for (id, node) in nodes.iter() {
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
