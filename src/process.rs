//! Processes a network of nodes
//!
//! Processes one module. If you have many modules, this needs to be called for each of them.

use crate::nodes::common::{ProcessNode, ProcessNodeEnvironment, Ports};
use std::collections::HashMap;

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

// TODO merayen create a module called "execution_order" or something, that plans the order the nodes are to be executed

/// Process a single frame
///
/// Execute all the nodes in a module.
pub fn process_frame(env: &ProcessNodeEnvironment, nodes: &mut HashMap<String, Option<Box<dyn ProcessNode>>>, ports: &mut HashMap<String, Ports>) {
	for (id, node) in nodes.iter_mut() {
		// TODO merayen need correct execution order here
		match node {
			Some(process_node) => {
				process_node.process(env, &mut ports.get_mut(id).expect("ProcessNode does not have its Ports initialized. Forgotton init_nodes()?"));
			}
			None => {
				// Node wasn't possible to parse or is unknown. Ignore it
			}
		}
	}
}
