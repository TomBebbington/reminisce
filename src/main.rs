extern crate reminisce;
use reminisce::*;
fn main() {
	let mut joysticks:Vec<SmartJoystick> = (0..4).filter_map(|i| SmartJoystick::new(i).ok()).collect();
	for js in &joysticks {
		println!("Joystick #{}: {}", js.get_index(), js.get_id());
		println!("\tAxes: {}", js.get_num_axes());
		println!("\tButtons: {}", js.get_num_buttons());
	}
	if joysticks.len() == 0 {
		panic!("No joysticks plugged in")
	}
	loop {
		for js in &mut joysticks {
			js.update();
			if js.is_connected() {
				println!("{:?}", js)
			}
		}
	}
}
