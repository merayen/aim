//! Reads a complete project and processes it

use crate::parse_nodes::parse_module_text;

pub fn begin() -> Result<String, String> {
	match std::fs::read_dir("./") {
		Ok(paths) => {
			for x in paths {
				let path = x.unwrap().path();
				let filename = path.to_str().unwrap();

				if filename.ends_with(".txt") {
					println!("{}", filename);
					let stuff = std::fs::read_to_string(filename).unwrap();
					parse_module_text(stuff.as_str());
				}
			}

			Ok("Got it".to_string())
		}
		Err(error) => {
			Err("Could not open directory".to_string())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn scan_directory() {
		std::env::set_current_dir("example_project");
		begin();
		// TODO merayen
	}
}
