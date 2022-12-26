//! Reads a complete project and processes it

use std::collections::HashMap;
use crate::parse_nodes;
use crate::parse;
use crate::module::process;
use crate::module;
use crate::nodes;

/// Parse a complete project and all its modules
///
/// Its result can then be sent to the processing/DSP stage of the synth.
fn parse_project(path: &str) -> Result<HashMap<String, module::Module>, String> {
	// TODO merayen move this into ModuleNode
	let mut modules: HashMap<String, module::Module> = HashMap::new();
	match std::fs::read_dir(path) {
		Ok(paths) => {
			for x in paths {
				let path = x.unwrap().path();
				let filename = path.to_str().unwrap();

				if filename.ends_with(".aim") {
					let stuff = std::fs::read_to_string(filename).unwrap();
					let (module, text_consumer) = parse_nodes::parse_module_text(stuff.as_str());

					modules.insert(filename.to_string(), module);
				}
			}

			Ok(modules)
		}
		Err(error) => {
			Err(format!("Could not open directory \"{}\"", path))
		}
	}
}


/// Start processing loop for a single module
///
/// If you have multiple modules, each of them should be executed into separate
/// threads.
fn start_process_loop( // TODO merayen change signature to match caller
	env: &nodes::common::ProcessNodeEnvironment,
	module: &mut module::Module,
	mut frames_to_process: i32,
) {
	// TODO merayen move to process-module
	// TODO should probably only run the loop when reacting on commands

	// Calculate in what order the nodes should be executed
	module::execution_order::plan_execution_order(module);

	loop { // CTRL-C this
		process::process_frame(&env, module);

		if frames_to_process > 0 {
			frames_to_process -= 1;
			if frames_to_process == 0 {
				break;
			}
		}
	}
}


/// Parse project, execute any commands, then run it
///
/// * `frame_count` - How many frames to process. Below 0 means "infinite"
pub fn run(path: &str) -> bool {
	let mut modules = parse_project(path).expect("Could not parse project");

	// Print errors that came up
	let mut has_errors = false;
	for (filename, module) in &modules {
		for error in &module.errors {
			println!("{}: {}", filename, error);
			has_errors = true;
		}
	}

	if has_errors {
		return false;
	}

	assert_eq!(modules.len(), 1, "Only support 1 module for now"); // TODO merayen multimodule: support multiple modules

	let module: &mut module::Module = modules.values_mut().next().unwrap();

	let env = nodes::common::ProcessNodeEnvironment { // TODO merayen get these parameters somewhere
		buffer_size: 8,
		sample_rate: 44100,
	};

	start_process_loop(&env, module, -1);

	return true;
}


/// Run a single text block of text. For debugging.
pub fn run_single_module(text: &str, env: &nodes::common::ProcessNodeEnvironment, frames_to_process: i32) -> (module::Module, String) {
	let (mut module, result_text) = parse_nodes::parse_module_text(text);

	let env = nodes::common::ProcessNodeEnvironment { // TODO merayen get these parameters somewhere
		buffer_size: 8,
		sample_rate: 44100,
	};

	start_process_loop(&env, &mut module, frames_to_process);

	(module, result_text)
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing_example_project() {
		let modules = parse_project("example_project/").unwrap();

		assert_eq!(modules.len(), 1);
		assert!(modules.contains_key("example_project/main.aim"));
		assert_eq!(modules["example_project/main.aim"].nodes.len(), 4);
	}
}
