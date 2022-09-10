use std::collections::HashMap;
use crate::module::process;

pub trait ProcessNode {
	/// Initialize your ports here
	fn on_init(&mut self, env: &ProcessNodeEnvironment) -> Ports;

	/// Process a frame
	fn on_process(&mut self, env: &ProcessNodeEnvironment, ports: &mut Ports);
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
	node_index: usize, 

	/// Name of the outlet of the node connected to this Inlet
	outlet_name: String,
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

pub struct Ports {
	pub inlets: HashMap<String, Option<Inlet>>,
	pub outlets: HashMap<String, Outlet>,
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
		self.outlets.insert(name.to_string(), Outlet::signal());
	}

	/// Register an inlet
	pub fn inlet(&mut self, name: &str) {
		self.inlets.insert(name.to_string(), None);
	}
}

pub enum PortParameter {
	/// The parameter is connected to an inlet
	Inlet { name: String, node_id: String, outlet: String },

	/// An output port
	Outlet { name: String, node_id: String, inlet: String },

	/// The parameter is using a constant
	Constant { name: String, value: String},
}

/// Parse a parameter line that can be connected to an outlet of another node
pub fn parse_input_parameter(text: &str) -> Result<PortParameter, String> {
	// TODO merayen why do we parse in this module?
	let mut splitter = text.trim().split(" ");
	let name = splitter.next().unwrap().to_string();

	match splitter.next() {
		Some("<-") => {
			match splitter.next() {
				Some(node_id) => {
					match splitter.next() {
						Some(outlet) => {
							Ok(PortParameter::Inlet {name: name, node_id: node_id.to_string(), outlet: outlet.to_string()})
						}
						_ => {
							Err("Missing name of port of the connecting node".to_string())
						}
					}
				}
				_ => {
					Err("Missing id of the connecting node".to_string())
				}
			}
		}
		Some("->") => {
			match splitter.next() {
				Some(node_id) => {
					match splitter.next() {
						Some(inlet) => {
							Ok(PortParameter::Outlet {name: name, node_id: node_id.to_string(), inlet: inlet.trim().to_string()})
						}
						_ => {
							Err("Missing name of port of the connecting node".to_string())
						}
					}
				}
				_ => {
					Err("Missing id of the connecting node".to_string())
				}
			}
		}
		Some(v) => {
			Ok(
				PortParameter::Constant {
					name: name,
					value: (
						v.to_string() +
						" " +
						splitter.filter(|x|x.len()>1).map(|x|x.to_string() + " ").collect::<String>().as_str()).trim().to_string()
					}
				)
		}
		None => {
			Err("Parameter is missing value".to_string())
		}
	}
}

mod tests {
	use super::*;

	#[test]
	fn parsing_input_parameter() {
		let result = parse_input_parameter("frequency 440 test 123");

		match result {
			Ok(PortParameter::Constant {name, value}) => {
				assert_eq!(name, "frequency");
				assert_eq!("440 test 123", value);
			}
			Err(message) => {
				panic!("{}", message);
			}
			_ => {
				panic!();
			}
		}

		let result = parse_input_parameter("frequency <- id1 out");
		match result {
			Ok(PortParameter::Inlet {name, node_id, outlet}) => {
				assert_eq!(name, "frequency");
				assert_eq!(node_id, "id1");
				assert_eq!(outlet, "out");
			}
			Err(message) => {
				panic!("{}", message);
			}
			_ => {
				panic!();
			}
		}

		let result = parse_input_parameter("frequency -> id1 in");
		match result {
			Ok(PortParameter::Outlet {name, node_id, inlet}) => {
				assert_eq!(name, "frequency");
				assert_eq!(node_id, "id1");
				assert_eq!(inlet, "in");
			}
			Err(message) => {
				panic!("{}", message);
			}
			_ => {
				panic!();
			}
		}
	}
}
