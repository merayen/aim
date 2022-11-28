//! Handles one single module (e.g main.txt)
pub mod execution_order;
pub mod process;

use crate::nodes;
use std::collections::HashMap;

pub struct Module {
	/// ProcessNodes configured from the module
	pub nodes: HashMap<String, Option<Box<dyn nodes::common::ProcessNode>>>,

	/// In which order the nodes should be executed
	///
	/// A list of node ids
	pub execution_order: Vec<String>,

	pub inlets: HashMap<String, nodes::common::Inlet>,
	pub outlets: HashMap<String, nodes::common::Outlet>,

	/// Errors that are shown in the stdout of the synth
	pub errors: Vec<String>,
}

impl Module {
	pub fn new() -> Self {
		Module {
			nodes: HashMap::new(),
			execution_order: Vec::new(),
			inlets: HashMap::new(),
			outlets: HashMap::new(),
			errors: Vec::new(),
		}
	}
}
