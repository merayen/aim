//! Module node is the top-most node in a project
//!
//! It can be imported by other nodes.
use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::nodes;
use crate::module::process;

pub fn parse(result: &mut parse_nodes::ParseResults, indent_block: &mut parse::IndentBlock) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	// TODO merayen this should parse a project directory, not module.rs
	Box::new(
		ModuleNode {
		}
	)
}

pub struct ModuleNode {
}

impl nodes::common::ProcessNode for ModuleNode {
	fn on_init(&mut self, env: &nodes::common::ProcessNodeEnvironment) -> nodes::common::Ports {
		nodes::common::Ports::new()
	}
	
	fn on_process(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &mut nodes::common::Ports, session: &process::session::Session) {
		// TODO merayen create initial voice if not existing
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::project;

	#[test]
	fn create_node_and_process() {
		let env = nodes::common::ProcessNodeEnvironment {
			sample_rate: 44100,
			buffer_size: 8,
		};
		let (parse_results, text) = project::run_single_module("
sine
	frequency 100
		",
			&env,
		);

		assert!(text == "
sine id1
	frequency 100
		".trim());

		assert!(parse_results.nodes.len() == 1);
		assert!(parse_results.nodes.contains_key("id1"));
	}
}
