extern crate reminisce;
use reminisce::*;
fn main() {
	let mut context = NativeContext::new();
	for js in context.get_joysticks() {
		println!("Joystick #{}: {}", js.index(), js.id());
		println!("\tAxes: {}", js.num_axes());
		println!("\tButtons: {}", js.num_buttons());
	}
	loop {
		for event in context.poll() {
			println!("{} - {:?}", context.get_joysticks().len(), event)
		}
	}
}
