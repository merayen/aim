//! Project node is the top-most node in a project
//!
//! It can be imported into other nodes via linking.
use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::nodes;

pub fn parse(result: &mut parse_nodes::ParseResults, indent_block: &mut parse::IndentBlock) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
}

pub struct ProjectNode {
}

impl nodes::common::ProcessNode for ProjectNode {
	fn on_init(&mut self, env: &nodes::common::ProcessNodeEnvironment) -> nodes::common::Ports {
		let mut ports = nodes::common::Ports::new();

		ports
	}
	
	fn process(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &mut nodes::common::Ports) {
		todo create initial 
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
