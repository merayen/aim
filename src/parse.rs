struct ParseNode<'a> {
	id: Option<&'a str>,
	title: &'a str,
	properties: std::collections::HashMap<String, &'a str>,
}

pub fn parse_module(path: &str) {
	let mut parse_nodes: Vec<ParseNode> = Vec::new();

	let text = std::fs::read_to_string(path).expect(format!("Could not read {path}").as_str());
	for line in text.split("\n") {
		if line.starts_with("# ") {
			let mut splitter = line.splitn(3, " ");
			splitter.next();
			
			parse_nodes.push(ParseNode {
				title: splitter.next().unwrap(),
				id: splitter.next().or(None),
				properties: std::collections::HashMap::new(),
			});
		} else if line.trim().len() > 0 {
			let mut splitter = line.splitn(2, " ");
			let p = parse_nodes.last_mut().unwrap();
			let key = splitter.next().unwrap();

			p.properties.insert(
				key.to_string(),
				splitter.next().or(Some("")).unwrap(),
			);
			println!("{}({}): {}={}", p.title, p.id.or(Some("")).unwrap(), key, p.properties.get(key).unwrap());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
