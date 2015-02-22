//! Reminisce is a lightweight library intended to be used for detecting and
//! reading from joysticks.

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

use std::ops::Deref;

#[derive(Copy, Debug)]
/// An event from a joystick
pub enum Event {
	/// Fires when a button is pressed with the button's index
	ButtonPressed(u8),
	/// Fires when a button is released with the button's index
	ButtonReleased(u8),
	/// Fires when a joystick / axis is moved with the axis index and its value,
	/// which is between MIN_JOYSTICK_VALUE and MAX_JOYSTICK_VALUE
	JoystickMoved(u8, i16)
}
/// Convert a raw event into a Reminisce event
pub trait IntoEvent {
	/// Convert this into a Reminisce event
	fn into_event(self) -> Event;
}

#[derive(Debug)]
/// A joystick that tracks its state
pub struct SmartJoystick {
	js: Joystick,
	/// The axes on the joystick, normalised to floats between -1.0 and 1.0
	///
	/// Typically the first two of these are the primary analog stick's x and y
	/// co-ordinates
	pub axes: Vec<f32>,
	/// The buttons on the joystick as booleans indicating if they are pressed
	///
	/// The order of these usually indicates the button priority with the first
	/// two being accept and back buttons
	pub buttons: Vec<bool>
}
impl SmartJoystick {
	/// Create a new joystick from its index
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
	/// Update the joystick's state
	pub fn update(&mut self) {
		while let Some(_) = self.poll() {}
	}
}
impl Deref for SmartJoystick {
	type Target = Joystick;
	fn deref(&self) -> &Joystick {
		&self.js
	}
}
