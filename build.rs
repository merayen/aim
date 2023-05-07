extern crate bindgen;

fn main() {
	println!("cargo:rustc-link-lib=dylib=ao");
	let bindgen = bindgen::Builder::default().header("").generate();
}
