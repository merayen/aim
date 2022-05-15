//! Reads a complete project and processes it

use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::process;
use crate::nodes;


/// Parse a complete project and all its modules
///
/// Its result can then be sent to the processing/DSP stage of the synth.
fn parse_project(path: &str) -> Result<HashMap<String, parse_nodes::ParseResults>, String> {
	let mut modules: HashMap<String, parse_nodes::ParseResults> = HashMap::new();
	match std::fs::read_dir(path) {
		Ok(paths) => {
			for x in paths {
				let path = x.unwrap().path();
				let filename = path.to_str().unwrap();

				if filename.ends_with(".txt") {
					let stuff = std::fs::read_to_string(filename).unwrap();
					let (results, text_consumer) = parse_nodes::parse_module_text(stuff.as_str());

					modules.insert(filename.to_string(), results);
				}
			}

			Ok(modules)
		}
		Err(error) => {
			Err("Could not open directory".to_string())
		}
	}
}


/// Parse project and execute any commands and then run it
///
/// * `frame_count` - How many frames to process. Below 0 means "infinite"
pub fn run(path: &str) {
	let mut modules = parse_project(path).expect("Could not parse project");

	// Print errors that came up
	for (filename, module) in &modules {
		for error in &module.errors {
			println!("{}", error);
		}
	}

	assert!(modules.len() == 1, "Only support 1 module for now");

	// TODO merayen support multiple modules
	let parse_results: &mut parse_nodes::ParseResults = modules.values_mut().next().unwrap();
	let nodes = &mut parse_results.nodes;

	let env = nodes::common::ProcessNodeEnvironment { // TODO merayen move out
		buffer_size: 8,
		sample_rate: 44100,
	};

	let mut ports: HashMap<String, nodes::common::Ports> = process::init_nodes(&env, nodes);

	start_process_loop(&env, nodes, &mut ports);

}


/// Run a single text block of text. For debugging.
pub fn run_single_module<'a>(text: &'a str, env: &nodes::common::ProcessNodeEnvironment) -> (parse_nodes::ParseResults, String) {
	let (parse_results, result_text) = parse_nodes::parse_module_text(text);

	(parse_results, result_text)
}


fn start_process_loop(
	env: &nodes::common::ProcessNodeEnvironment,
	nodes: &mut HashMap<String, Option<Box<dyn nodes::common::ProcessNode>>>,
	ports: &mut HashMap<String, nodes::common::Ports>,
) {
	// TODO should probably only run the loop when reacting on commands
	let mut frames_to_process = -1; // TODO merayen get the frames to process count from a command, like "<play 100"
	loop { // CTRL-C this
		if frames_to_process == 0 {
			break;
		}
		if frames_to_process > 0 {
			frames_to_process -= 1;
		}

		process::process_frame(&env, nodes, ports);
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing_example_project() {
		// TODO merayen hangs forever
		let modules = parse_project("example_project/").unwrap();

		assert!(modules.len() == 1);
		assert!(modules.contains_key("example_project/main.txt"));
		assert!(modules["example_project/main.txt"].nodes.len() == 4);
	}
}
