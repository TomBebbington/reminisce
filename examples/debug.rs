extern crate reminisce;
use reminisce::*;
use std::thread;
use std::time::Duration;
fn main() {
	let mut context = NativeContext::new();
	for js in context.get_joysticks() {
		println!("Joystick #{}: {}", js.get_index(), js.get_id());
		println!("\tAxes: {}", js.get_num_axes());
		println!("\tButtons: {}", js.get_num_buttons());
	}
	loop {
		for event in context.poll() {
			println!("{} - {:?}", context.get_joysticks().len(), event)
		}
	}
}
