//! Handles one single module (e.g main.txt)
pub mod execution_order;
pub mod process;

use crate::nodes;
use std::collections::HashMap;

pub struct Module {
	/// ProcessNodes configured from the module
	pub nodes: HashMap<String, Option<Box<dyn nodes::common::ProcessNode>>>,

	/// In which order the nodes should be executed in this module
	///
	/// A list of node ids
	pub execution_order: Vec<String>,

	pub ports: HashMap<String, nodes::common::Ports>,

	/// Errors that are shown in the stdout of the synth
	pub errors: Vec<String>,
}

impl Module {
	pub fn new() -> Self {
		Module {
			nodes: HashMap::new(),
			execution_order: Vec::new(),
			ports: HashMap::new(),
			errors: Vec::new(),
		}
	}

	pub fn is_ok(&self) -> bool {
		self.errors.len() == 1
	}
}
