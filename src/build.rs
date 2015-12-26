#[cfg(all(target_os = "windows", not(feature = "sdl")))]
static SEARCH_PATHS: [&'static str; 2] = [
	"C:\\Windows\\SysWOW64",
	"C:\\Windows\\System32"
];

#[cfg(all(target_os = "windows", not(feature = "sdl")))]
static LIB_NAME: &'static str = "XInput9_1_0.dll";

#[cfg(all(target_os = "windows", not(feature = "sdl")))]
fn main() {
	use std::{env, fs};
	use std::path::{Path, PathBuf};
	let mut lib = None;
	for path in SEARCH_PATHS.iter() {
		let mut path = Path::new(path).to_owned();
		path.push(LIB_NAME);
		if path.exists() {
			lib = Some(path);
		}
	}
	let lib = lib.expect("Could not locate XInput");
	let out_dir = PathBuf::from(&env::var("OUT_DIR").expect("Must be ran from cargo"));
	if !out_dir.exists() {
		panic!("Invalid value for OUT_DIR");
	}
	let mut out = out_dir.clone();
	out.push(&lib.file_name().expect("No file name found"));
	fs::copy(lib, &out).ok().expect("Failed to copy XInput");
	println!("cargo:rustc-flags=-L native={:?}", out_dir);
}
#[cfg(not(windows))]
fn main() {
}
