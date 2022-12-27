use std::cell::RefCell;
use std::collections::HashMap;
use crate::module::process;
use crate::module;
use crate::nodes;

pub trait ProcessNode {
	/// Set up the node here
	///
	/// Called only once. Store any references to the buffers the node will be reading to keep the
	/// overhead as low as possible when processing each frame.
	fn on_init(&mut self, env: &ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>);

	/// Process a frame
	fn on_process(&mut self, node_id: String, env: &ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>);
}

pub struct ProcessNodeEnvironment {
	pub buffer_size: usize,
	pub sample_rate: i32,
}

/// Outlet from a node
///
/// Contains multiple voices that are lazily allocated.
pub struct Outlet {
	/// Audio data. Format: [voice_index][channel_index][sample_index]
	pub audio: Option<Vec<Vec<f32>>>,

	/// Signal data. Format: [voice_index][sample_index]
	pub signal: Option<Vec<Vec<f32>>>,

	/// Midi data. Format: [voice_index][midi_index]. This probably needs to be redesigned.
	pub midi: Option<Vec<Vec<u8>>>,
}

pub struct Inlet {
	/// Index to the node that this Inlet is connected to
	pub node_id: String,

	/// Name of the outlet of the node connected to this Inlet
	pub outlet: String,
}

impl Outlet {
	fn signal() -> Self {
		Outlet {
			signal: Some(Vec::new()),
			audio: None,
			midi: None,
		}
	}

	fn audio() -> Self {
		Outlet {
			signal: None,
			audio: Some(Vec::new()),
			midi: None,
		}
	}

	fn midi() -> Self {
		Outlet {
			signal: None,
			audio: None,
			midi: Some(Vec::new()),
		}
	}
}


/// Ports for a node
pub struct Ports {
	/// HashMap containing name of input port and Some(Inlet) if connected(?)
	pub inlets: HashMap<String, Option<Inlet>>,  // TODO merayen do we need Option<> here? Just don't have the key instead?
	pub outlets: HashMap<String, RefCell<Outlet>>,
}

impl Ports {
	pub fn new() -> Ports {
		Ports {
			inlets: HashMap::new(),
			outlets: HashMap::new(),
		}
	}

	/// Create a new outlet configured to send signals
	pub fn signal(&mut self, name: &str, env: &ProcessNodeEnvironment) {
		self.outlets.insert(name.to_string(), RefCell::new(Outlet::signal()));
	}

	/// Register an inlet
	pub fn inlet(&mut self, name: &str) {
		self.inlets.insert(name.to_string(), None);
	}
}


#[cfg(test)]
mod tests {
	use super::*;

}
