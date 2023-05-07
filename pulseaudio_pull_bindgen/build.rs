fn main() {
	println!("cargo:rustc-link-lib=dylib=pulse");
	println!("cargo:rerun-if-changed=wrapper.h");

	let bindings = bindgen::Builder::default()
		.header("wrapper.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.generate()
		.expect("Unable to create bindings");

		let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
		bindings
			.write_to_file(out_path.join("bindings.rs"))
			.expect("Could not write to bindings.rs");
}
