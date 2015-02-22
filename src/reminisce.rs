//! Reminisce is a lightweight library intended to be used for detecting and
//! reading from joysticks.

#![feature(core, std_misc, libc, fs, os, rustc_private, path)]
extern crate libc;
#[cfg(target_os = "windows")]
#[macro_use] extern crate rustc_bitflags;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{NativeJoystick, scan};

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::NativeJoystick;


/// The maximum axis value
pub static MAX_JOYSTICK_VALUE:i16 = 32767;
/// The minimum axis value
pub static MIN_JOYSTICK_VALUE:i16 = -32767;

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

/// A single joystick
pub trait Joystick {
	/// A joystick that includes the state
	type WithState : StatefulJoystick;

	/// Create a new joystick from its index
	fn new(index: u8) -> Result<Self, &'static str>;

	/// Check if the joystick is still connected
	fn is_connected(&self) -> bool;

	/// Get the identifier of this joystick
	fn get_id(&self) -> String;

	/// Get the index of this joystick
	fn get_index(&self) -> u8;

	/// Get the number of axes this joystick has
	fn get_num_axes(&self) -> u8;

	/// Get the number of buttons this joystick has
	fn get_num_buttons(&self) -> u8;

	/// Poll the joystick for events in non-blocking mode
	fn poll(&mut self) -> Option<Event>;

	/// Get a version of this joystick which includes state
	fn with_state(self) -> Self::WithState;
}

/// A single joystick with its state saved
pub trait StatefulJoystick : Joystick {
	/// Get the value of a specific axis from its index
	///
	/// Typically the first two of these are the primary analog stick's x and y
	/// co-ordinates
	fn get_axis(&self, index: u8) -> Option<i16>;

	/// Get the value of a specific axis normalised to between -1.0 and 1.0
	fn get_normalised_axis(&self, index: u8) -> Option<f32> {
		self.get_axis(index).map(|v| v as f32 / MAX_JOYSTICK_VALUE as f32)
	}

	/// Get the value (if it is pressed or not) of a specific button
	///
	/// The first two buttons are usually the accept and back buttons
	fn get_button(&self, index: u8) -> Option<bool>;

	/// Update the state of this joystick
	fn update(&mut self);
}
