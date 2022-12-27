//! Sends data out of current scope
//!
//! If in the topmost scope, and audio-like, send it to the speakers
use std::collections::HashMap;
use crate::nodes;
use crate::parse;
use crate::module::process;
use crate::module;
use crate::parse_nodes;
use crate::audio_output;

struct OutNode {
	audio_output: Option<audio_output::AudioOutput>,
}

pub fn new(indent_block: &mut parse::IndentBlock, ports: &mut nodes::common::Ports) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	for indent_block in &mut indent_block.children {
		match parse_nodes::parse_node_parameter(&indent_block.text) {
			Ok(parse_nodes::PortParameter::Inlet {name, node_id, outlet}) => {
				match name.as_str() {
					"in" => {
						&ports.inlets.insert(name, Some(nodes::common::Inlet {node_id, outlet}) );
					}
					_ => {
						indent_block.text.push_str("  # ERROR: Unknown inlet")
					}
				}
			}
			Err(message) => {
				indent_block.text.push_str(&("  # ERROR: ".to_string() + &message));
			}
			_ => {
				indent_block.text.push_str("  # ERROR: Invalid parameter");
			}
		}
	}

	Box::new(
		OutNode {
			audio_output: None,
		}
	)
}

impl nodes::common::ProcessNode for OutNode {
	// TODO merayen mixing: instead of sending directly to OS-mixer, implement a mixer inside synth
	fn on_init(&mut self, node_id: String, env: &nodes::common::ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>) {
			// TODO merayen multichannel: we shouldn't connect directly to the audio output, rather go via an internal mixer
			self.audio_output = Some(audio_output::AudioOutput::new(1, env.sample_rate));
	}

	fn on_process(&mut self, node_id: String, env: &nodes::common::ProcessNodeEnvironment, ports: &HashMap<String, nodes::common::Ports>) {
		let inlets = &ports[&node_id].inlets;
		let outlets = &ports[&node_id].outlets;

		if inlets.contains_key("in") {
			let out_inlet = inlets["in"].as_ref().unwrap();
			let out_node_id: &String = &out_inlet.node_id;
			let out_node_outlet: &String = &out_inlet.outlet;

			let input_buffer = &ports[out_node_id.as_str()].outlets[out_node_outlet.as_str()].borrow();

			// Mix all the voices into a single buffer (multichannel)
			let mut output_audio: Vec<f32> = vec![0f32; env.buffer_size]; // TODO merayen multichannel: fixed to single channel for now

			if let Some(input) = &input_buffer.audio {
				unimplemented!("Not implemented"); // TODO merayen implement outputting audio data
			} else if let Some(input) = &input_buffer.signal {
				for voice in input {
					for (i,sample) in voice.iter().enumerate() {
						output_audio[i] += sample;
					}
				}

				// Send the data to the speakers
				self.audio_output.as_ref().unwrap().output(&output_audio);
			}
		} else {
			println!("Unsupported type"); // TODO merayen remove
		}
	}
}
