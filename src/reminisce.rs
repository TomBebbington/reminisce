#![feature(core, std_misc, libc, os)]
extern crate libc;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

#[derive(Copy, Debug)]
/// An event from a joystick
pub enum Event {
	ButtonPressed(u8),
	ButtonReleased(u8),
	JoystickMoved(u8, i16)
}
/// Convert a raw event into a Reminisice event
pub trait IntoEvent {
	fn into_event(self) -> Event;
}

#[derive(Debug)]
pub struct SmartJoystick {
	js: Joystick,
	pub axes: Vec<f32>,
	pub buttons: Vec<bool>
}
impl SmartJoystick {
	pub fn new(index: u8) -> Result<SmartJoystick, &'static str> {
		let js = try!(Joystick::new(index));
		Ok(SmartJoystick {
			axes: vec![0.0f32; js.get_num_axes() as usize],
			buttons: vec![false; js.get_num_buttons() as usize],
			js: js
		})
	}
	/// Poll this joystick for events
	pub fn poll(&mut self) -> Option<Event> {
		let event = self.js.poll();
		match event {
			Some(Event::JoystickMoved(i, v)) => self.axes[i as usize] = v as f32 / MAX_JOYSTICK_VALUE as f32,
			Some(Event::ButtonPressed(i)) => self.buttons[i as usize] = true,
			Some(Event::ButtonReleased(i)) => self.buttons[i as usize] = false,
			_ => ()
		}
		event
	}
	pub fn update(&mut self) {
		while let Some(_) = self.poll() {}
	}
}
impl ::std::ops::Deref for SmartJoystick {
	type Target = Joystick;
	fn deref(&self) -> &Joystick {
		&self.js
	}
}
