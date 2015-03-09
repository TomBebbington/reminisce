//! Reminisce is a lightweight library intended to be used for detecting and
//! reading from joysticks.
#![allow(unused_features)]
#![feature(core, std_misc, libc, fs, fs_walk, os, rustc_private, path)]
extern crate libc;
#[cfg(target_os = "windows")]
#[macro_use] extern crate rustc_bitflags;

#[cfg(feature = "sdl")]
extern crate sdl2;

/// Someday, somehow
#[cfg(target_os = "emscripten")]
mod emscripten;

#[cfg(target_os = "emscripten")]
pub use emscripten as native;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux as native;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows as native;

#[cfg(feature = "sdl")]
mod sdl;

#[cfg(feature = "sdl")]
pub use sdl as native;

pub use native::{NativeJoystick, scan};


/// The maximum axis value
pub static MAX_JOYSTICK_VALUE:i16 = 32767;
/// The minimum axis value
pub static MIN_JOYSTICK_VALUE:i16 = -32767;

use std::mem::transmute as cast;
use std::borrow::Cow;

#[repr(u8)]
#[derive(Copy, Debug, PartialEq, Eq)]
/// A direction on a joystick
pub enum Axis {
	/// The x direction of the left stick
    LeftX,
	/// The y direction of the left stick
    LeftY,
	/// The x direction of the right stick
    RightX,
	/// The y direction of the right stick
    RightY,
	/// How far down the left trigger is pressed
    TriggerLeft,
	/// How far down the right trigger is pressed
    TriggerRight,
}

#[repr(u8)]
#[derive(Copy, Debug, PartialEq, Eq)]
/// A button on a joystick
pub enum Button {
	/// The A button - typically used for jumping
	A,
	/// The B button - typically used for shooting
	B,
	/// The X button
	X,
	/// The Y button
	Y,
	/// The back / select button
	Back,
	/// The guide button
	Guide,
	/// The start button
	Start,
	/// The left stick button
	LeftStick,
	/// The right stick button
	RightStick,
	/// The left shoulder button
	LeftShoulder,
	/// The right shoulder button
	RightShoulder,
	/// The up button on the directional pad
	DPadUp,
	/// The down button on the directional pad
	DPadDown,
	/// The left button on the directional pad
	DPadLeft,
	/// The right button on the directional pad
	DPadRight
}

#[derive(Copy, Debug, Eq, PartialEq)]
/// An event from a joystick
pub enum Event {
	/// Fires when a button is pressed with the button's index
	ButtonPressed(Button),
	/// Fires when a button is released with the button's index
	ButtonReleased(Button),
	/// Fires when a joystick / axis is moved with the axis index and its value,
	/// which is between MIN_JOYSTICK_VALUE and MAX_JOYSTICK_VALUE
	JoystickMoved(Axis, i16)
}
/// Convert a raw event into a Reminisce event
pub trait IntoEvent {
	/// Convert this into a Reminisce event
	fn into_event(self) -> Event;
}

impl IntoEvent for Event {
    fn into_event(self) -> Event {
        self
    }
}

/// A single joystick
pub trait Joystick : Sized {
	/// A joystick that includes the state
	type WithState : StatefulJoystick;

    /// The event that this joystick processes
    type NativeEvent: IntoEvent;

	/// Create a new joystick from its index
	fn new(index: u8) -> Result<Self, &'static str>;

	/// Check if the joystick is still connected
	fn is_connected(&self) -> bool;

	/// Get the identifier of this joystick
    ///
    /// This is a cow because it can be a String
	fn get_id(&self) -> Cow<str>;

	/// Get the index of this joystick
	fn get_index(&self) -> u8;

	/// Get the number of axes this joystick has
	fn get_num_axes(&self) -> u8;

	/// Get the number of buttons this joystick has
	fn get_num_buttons(&self) -> u8;

    /// Get the battery level of this joystick
    ///
    /// Returns none if the joystick is wired or this operation is not supported
    fn get_battery(&self) -> Option<f32>;

    /// Poll the joystick for events in non-blocking mode
	fn poll_native(&mut self) -> Option<Self::NativeEvent>;

    /// Poll the joystick for events in non-blocking mode
    fn poll(&mut self) -> Option<Event> {
        self.poll_native().map(|e| e.into_event())
    }

    /// Iterate through the joystick's event queue in non-blocking mode
    fn iter(&mut self) -> Poller<Self> {
        Poller {
            joystick: self
        }
    }

	/// Get a version of this joystick which includes state
	fn with_state(self) -> Self::WithState;
}
impl Iterator for NativeJoystick {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        self.poll()
    }
}

/// A single joystick with its state saved
pub trait StatefulJoystick : Joystick + Sized {
	/// Get the value of a specific axis from its index
	///
	/// Typically the first two of these are the primary analog stick's x and y
	/// co-ordinates
	fn get_axis(&self, index: Axis) -> Option<i16>;

	/// Get the value of a specific axis normalised to between -1.0 and 1.0
	fn get_normalised_axis(&self, index: Axis) -> Option<f32> {
		self.get_axis(index).map(|v| v as f32 / MAX_JOYSTICK_VALUE as f32)
	}

	/// Iterate over the axes in this joystick
	fn axes(&self) -> Axes<Self> {
		use std::mem;
		Axes {
			joystick: self,
			axis: unsafe { mem::zeroed() }
		}
	}

	/// Iterate over the buttons in this joystick
	fn buttons(&self) -> Buttons<Self> {
		use std::mem;
		Buttons {
			joystick: self,
			button: unsafe { mem::zeroed() }
		}
	}

	/// Get the value (if it is pressed or not) of a specific button
	///
	/// The first two buttons are usually the accept and back buttons
	fn get_button(&self, index: Button) -> Option<bool>;

	/// Update the state of this joystick
	fn update(&mut self);
}

/// An iterator over a joystick's axes
pub struct Axes<'a, J> where J:StatefulJoystick+'a {
	joystick: &'a J,
	axis: u8
}
impl<'a, J> Iterator for Axes<'a, J> where J:StatefulJoystick {
	type Item = (Axis, i16);
	fn next(&mut self) -> Option<(Axis, i16)> {
		self.axis += 1;
		let axis = unsafe { cast(self.axis - 1) };
		self.joystick.get_axis(axis).map(|v| (axis, v))
	}
}

/// An iterator over a joystick's buttons
pub struct Buttons<'a, J> where J:StatefulJoystick+'a {
	joystick: &'a J,
	button: u8
}
impl<'a, J> Iterator for Buttons<'a, J> where J:StatefulJoystick {
	type Item = (Button, bool);
	fn next(&mut self) -> Option<(Button, bool)> {
		self.button += 1;
		let button = unsafe { cast(self.button - 1) };
		self.joystick.get_button(button).map(|v| (button, v))
	}
}

/// An iterator over a joystick's event queue
pub struct Poller<'a, J> where J:Joystick+'a {
    joystick: &'a mut J
}

impl<'a, J> Iterator for Poller<'a, J> where J:Joystick {
	type Item = Event;
	fn next(&mut self) -> Option<Event> {
		self.joystick.poll()
	}
}
