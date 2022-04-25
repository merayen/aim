//! Reads a complete project and processes it

use std::collections::HashMap;
use crate::parse_nodes::{parse_module_text, ParseResults};

/// Parse a complete project and all its modules
///
/// Its result can then be sent to the processing/DSP stage of the synth.
fn parse_project() -> Result<HashMap<String, ParseResults>, String> {
	let mut modules: HashMap<String, ParseResults> = HashMap::new();
	match std::fs::read_dir("./") {
		Ok(paths) => {
			for x in paths {
				let path = x.unwrap().path();
				let filename = path.to_str().unwrap();
				assert!(filename.starts_with("./"));

				if filename.ends_with(".txt") {
					println!("Parsing module {}", filename);
					let stuff = std::fs::read_to_string(filename).unwrap();
					let module: ParseResults = parse_module_text(stuff.as_str());

					modules.insert(filename[2..filename.len()].to_string(), module);
				}
			}

			Ok(modules)
		}
		Err(error) => {
			Err("Could not open directory".to_string())
		}
	}
}

/// Parse project and execute any commands inside the project
pub fn begin() {
	let modules = parse_project().expect("Could not parse project");

	// Print errors that came up
	for (filename, module) in modules {
		for error in &module.errors {
			println!("{}", error);
		}
	}
	// TODO merayen send the nodes to something that process them?
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing_example_project() {
		std::env::set_current_dir("example_project").expect("Could not cd into example_project/");
		let modules = parse_project().unwrap();

		assert!(modules.len() == 1);
		assert!(modules.contains_key("main.txt"));

		for x in &modules["main.txt"].lines {
			if x.indent_level == 0 {
				println!("{}", x.text);
			}
		}

		println!("{}", modules["main.txt"].nodes.len());
		assert!(modules["main.txt"].nodes.len() == 4);

		std::env::set_current_dir("..").expect("Could not cd back with ../");
	}
}
