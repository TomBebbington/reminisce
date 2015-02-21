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