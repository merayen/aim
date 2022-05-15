use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::nodes;

pub fn parse(result: &mut parse_nodes::ParseResults, text_consumer: &mut parse::IndentBlock) -> Box<(dyn nodes::common::ProcessNode + 'static)> {
	unimplemented!("Not implemented"); // TODO merayen
	//assert!(text_consumer.current_indentation() == 0);
	//let mut frequency = 440f32;

	//if text_consumer.enter() { // Dive into the properties indentation level
	//	loop {
	//		let line = text_consumer.current_line();
	//		let mut splitter = line.splitn(2, " ");
	//		let property_name = splitter.next().unwrap();
	//		let property_value = splitter.next();
	//		match property_name {
	//			"frequency" => {
	//				frequency = property_value.unwrap().parse::<f32>().unwrap();
	//			}
	//			_ => {
	//				unimplemented!("Add 'unknown property' error"); // TODO merayen
	//			}
	//		}
	//		if !text_consumer.next_sibling() {break;}
	//	}
	//	text_consumer.leave();
	//}

	//Box::new(
	//	SineNode {
	//		frequency: 440f32,
	//		position: 0f64,
	//	}
	//)
}

pub struct SineNode {
	frequency: f32,
	position: f64,
}

impl nodes::common::ProcessNode for SineNode {
	fn on_init(&mut self, env: &nodes::common::ProcessNodeEnvironment) -> nodes::common::Ports {
		let mut ports = nodes::common::Ports::new();
		ports.signal("out", env);
		ports.inlet("frequency");

		ports
	}
	
	fn process(&mut self, env: &nodes::common::ProcessNodeEnvironment, ports: &mut nodes::common::Ports) {
		let mut out = ports.outlets.get_mut("out");
		let mut out_data = out.as_mut().unwrap();
		let mut signal = out_data.signal.as_mut().unwrap();

		let sample_rate = env.sample_rate as f64;
		let frequency = self.frequency as f64;

		for i in 0..env.buffer_size {
			signal[i] = 1337f32;
			self.position += frequency / sample_rate * std::f64::consts::PI;
		}
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

		println!("NEI{}JO", text);
		assert!(text == "
sine id1
	frequency 100
		".trim());
	}
}
