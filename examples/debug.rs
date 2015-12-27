extern crate reminisce;
use reminisce::*;
fn main() {
	let mut backend = Native::new();
	for js in backend.joysticks() {
		println!("Joystick #{}: {}", js.index(), js.id());
		println!("\tAxes: {}", js.num_axes());
		println!("\tButtons: {}", js.num_buttons());
	}
	loop {
		for event in backend.poll() {
			println!("{:?}", event);
		}
	}
}
