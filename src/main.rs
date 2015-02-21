extern crate reminisce;
use reminisce::*;
fn main() {
	let mut joysticks:Vec<Joystick> = (0..4).filter_map(|i| Joystick::new(i).ok()).collect();
	for js in &joysticks {
		println!("Joystick #{}: {}", js.get_id(), js.get_pretty_id());
		println!("\tAxes: {}", js.get_num_axes());
		println!("\tButtons: {}", js.get_num_buttons());
	}
	if joysticks.len() == 0 {
		panic!("No joysticks plugged in")
	}
	loop {
		for js in &mut joysticks {
			if js.is_plugged() {
				if let Some(event) = js.poll() {
					println!("{}: {:?}", js.get_id(), event)
				}
			}
		}
	}
}