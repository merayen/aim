pub struct ParseNode {
	id: Option<String>,
	title: String,
	properties: std::collections::HashMap<String, String>,
}

pub fn parse_module(path: &str) -> Vec<ParseNode> {
	let mut parse_nodes: Vec<ParseNode> = Vec::new();

	let text = std::fs::read_to_string(path).expect(format!("Could not read {}", path).as_str());
	for line in text.split("\n") {
		if line.starts_with("# ") {
			let mut splitter = line.splitn(3, " ");
			splitter.next();
			
			parse_nodes.push(ParseNode {
				title: splitter.next().unwrap().to_string(),
				id: splitter.next().map(|s| s.to_string()),
				properties: std::collections::HashMap::new(),
			});
		} else if line.trim().len() > 0 {
			let mut splitter = line.splitn(2, " ");
			let p = parse_nodes.last_mut().unwrap();
			let key = splitter.next().unwrap();

			p.properties.insert(
				key.to_string(),
				splitter.next().unwrap_or("").to_string(),
			);

			println!("{}({}): {}={}", p.title, p.id.as_ref().unwrap_or(&"".to_string()), key, p.properties.get(key).unwrap());
		}
	}

	parse_nodes
}

#[cfg(test)]
mod tests {
	use super::*;
}
