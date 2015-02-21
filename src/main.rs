extern crate reminisce;
use reminisce::*;
fn main() {
	let mut js = Joystick::new(0).unwrap();
	println!("Axes: {}", js.get_num_axes());
	println!("Buttons: {}", js.get_num_buttons());
	println!("Id: {}", js.get_pretty_id());
	loop {
		if let Some(event) = js.poll() {
			println!("{:?}", event)
		}
	}
}