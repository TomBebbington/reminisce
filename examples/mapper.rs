extern crate reminisce;
use reminisce::*;
fn main() {
	let mut joysticks = scan();
	let mut joystick = joysticks.pop().expect("Failed to open joystick").into_mapper();
	println!("Please press the button you would like to map, then press Start to debug");
	let mut button = None;
	let mut mappings = Vec::new();
	'a: loop {
		for event in joystick.iter() {
			match event {
				Event::ButtonPressed(Button::Start) => break 'a,
				Event::ButtonPressed(btn) =>
					match button {
						Some(from) => {
							mappings.push((from, btn));
							println!("Mapping from {:?} to {:?}", from, btn);
							button = None;
						},
						None => {
							button = Some(btn);
							println!("Now press the button you want to map {:?} to", btn);
						}
					},
				_ => ()
			}
		}
	}
	joystick.map_buttons(mappings.into_iter());
	loop {
		for event in joystick.iter() {
			println!("{:?}", event)
		}
	}
}
