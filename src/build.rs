#![feature(path, path_ext)]
#[cfg(windows)]
fn main() {
	use std::env;
	use std::fs::{self, PathExt};
	use std::path::{Path, PathBuf};
	let lib64 = Path::new("C:\\Windows\\SysWOW64\\XInput9_1_0.dll");
	let lib =  if lib64.exists() {
		lib64
	} else {
		Path::new("C:\\Windows\\System32\\XInput9_1_0.dll")
	};
	let out_dir = PathBuf::new(&env::var("OUT_DIR").unwrap());
	let mut out = out_dir.clone();
	out.push(&lib.file_name().unwrap()); 
	fs::copy(lib, &out).ok().expect("Failed to copy XInput");
	println!("cargo:rustc-flags=-L native={:?}", out_dir);
}
#[cfg(not(windows))]
fn main() {
}