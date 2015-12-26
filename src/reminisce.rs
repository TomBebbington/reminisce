//! Reminisce is a lightweight library intended to be used for detecting and
//! reading from joysticks at a low level across many different operating
//! systems.
//!
//! Scanning for joysticks
//! ----------------------
//! To scan for joysticks, a `Backend` must be created.
//!
//! ``` rust
//! use reminisce::{Backend, Native};
//! let backend = Native::new();
//! println!("{} joysticks connected", backend.num_joysticks());
//! ```
//!
//! Scanning for events
//! -------------------
//! To scan events, a `Backend` must be created and polled.
//!
//! ``` rust
//! use reminisce::{Backend, Native};
//! let mut backend = Native::new();
//! for event in &mut backend {
//!     println!("{:?}", event);
//! }
//! ```
extern crate libc;
#[cfg(target_os = "windows")]
#[macro_use] extern crate rustc_bitflags;

#[cfg(feature = "sdl")]
extern crate sdl2;

extern crate glob;

#[cfg(all(target_os = "linux", not(feature = "sdl")))]
pub mod linux;

#[cfg(all(target_os = "linux", not(feature = "sdl")))]
pub use linux as native;

#[cfg(all(target_os = "windows", not(feature = "sdl")))]
pub mod windows;

#[cfg(all(target_os = "windows", not(feature = "sdl")))]
pub use windows as native;

#[cfg(feature = "sdl")]
pub mod sdl;

#[cfg(feature = "sdl")]
pub use sdl as native;

/// The native joystick backend exposed as a `Backend`.
pub use native::Native;


/// The maximum axis value
pub static MAX_AXIS_VALUE:i16 = 32767;
/// The minimum axis value
pub static MIN_AXIS_VALUE:i16 = -32767;

use std::borrow::Cow;
use std::fmt::Debug;

/// A direction on a joystick.
pub type Axis = u8;

/// A button on a joystick.
pub type Button = u8;

/// A joystick index.

pub type JoystickIndex = u8;
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// An event emitted by a joystick
pub enum Event {
    /// Fired when a joystick is connected with its index.
    Connected(JoystickIndex),
    /// Fired when a joystick is disconnected with its index.
    Disconnected(JoystickIndex),
    /// Fired when a button is pressed with the joystick index and the
    /// button's index.
    ButtonPressed(JoystickIndex, Button),
    /// Fired when a button is released with the joystick index and the
    /// button's index.
    ButtonReleased(JoystickIndex, Button),
    /// Fired when a axis is moved with the joystick index, axis index
    /// and its value, which is between `MIN_JOYSTICK_VALUE` and 
    /// `MAX_JOYSTICK_VALUE`
    AxisMoved(JoystickIndex, Axis, i16)
}

/// A lightweight Backend that tracks and polls all the available joysticks.
///
/// Each `Backend` has its own event queue tied to the events of the joysticks.
///
/// ``` rust
/// use reminisce::*;
/// let mut backend = Native::new();
/// let mut dir = 0;
/// let mut left = 30;
/// for event in &mut backend {
///     left -= 1;
///     if(left <= 0) { break }
///     match event {
///         Event::AxisMoved(_, 0, nx) =>
///             dir = -1,
///         Event::AxisMoved(_, 1, ny) =>
///             dir = 1,
///         _ => ()
///     }
/// }
/// ```
pub trait Backend: Sized + Send {
    /// The kind of joystick this Backend tracks.
    type Joystick : Joystick;

    /// Create a new Backend and scan for joysticks.
    fn new() -> Self;

    /// Returns the number of joysticks connected.
    fn num_joysticks(&self) -> usize;

    /// Return a reference to the joysticks connected.
    fn joysticks(&self) -> &[Self::Joystick];

    /// Poll this Backend non-blockingly for events from any joysticks.
    fn poll(&mut self) -> Option<Event>;

    /// Iterate through the events that haven't been processed yet
    fn iter(&mut self) -> Poller<Self> {
        Poller { backend: self }
    }
}

impl<'a> IntoIterator for &'a mut Native {
    type Item = Event;
    type IntoIter = Poller<'a, Native>;
    fn into_iter(self) -> Poller<'a, Native> {
        self.iter()
    }
}

/// A joystick or gamepad.
pub trait Joystick : Sized {
    /// The error that could be thrown while trying to open the joystick
    type OpenError: Debug;

    /// Attempts to open a joystick from its index
    ///
    /// If an error occurs, this will return the textual representation of that error.
    ///
    /// ``` rust
    /// use reminisce::{Backend, Joystick, Native};
    /// if let Ok(joystick) = <Native as Backend>::Joystick::open(0) {
    ///     println!("{}", joystick.id())
    /// } else {
    ///     println!("No joystick plugged in")
    /// }
    /// ```
    fn open(index: JoystickIndex) -> Result<Self, Self::OpenError>;

    /// Check if the joystick is still connected.
    fn connected(&self) -> bool;

    /// Get the identifier of this joystick.
    ///
    /// This is copy-on-write because it can be a borrowed or owned String,
    /// depending on the implementation.
    fn id(&self) -> Cow<str>;

    /// Get the index of this joystick.
    fn index(&self) -> JoystickIndex;

    /// Get the number of axes this joystick has
    ///
    /// This is capped at 6 axes for now.
    fn num_axes(&self) -> Axis;

    /// Get the number of buttons this joystick has
    ///
    /// This is capped at 16 buttons currently.
    fn num_buttons(&self) -> Button;

    /// Get the battery level of this joystick
    ///
    /// Returns none if the joystick is wired or this operation is not supported
    /// by the backend
    fn battery(&self) -> Option<f32>;
}

/// A joystick that tracks its state.
///
/// You must call `process` before you query anything.
pub struct StatefulJoystick<J> where J: Joystick {
    joystick: J,
    buttons: i32,
    axes: Vec<i16>
}
impl<J> StatefulJoystick<J> where J: Joystick {
    /// Get the position of the axis.
    pub fn axis(&self, axis: Axis) -> Option<i16> {
        self.axes.get(axis as usize).cloned()
    }
    /// Check if the button given is being pressed.
    pub fn button(&self, button: Button) -> Option<bool> {
        if button < self.joystick.num_buttons() {
            Some(self.buttons & (1 << button) > 0)
        } else {
            None
        }
    }
    /// Update this joystick's state with the event given.
    pub fn process(&mut self, event: Event) {
        let index = self.joystick.index();
        match event {
            Event::ButtonPressed(i, b) if i == index => {
                self.buttons |= (i as i32) << b;
            },
            Event::ButtonReleased(i, b) if i == index => {
                self.buttons &= !((i as i32) << b);
            },
            Event::AxisMoved(i, a, v) if i == index => {
                self.axes[a as usize] = v;
            },
            _ => ()
        }
    }
}
impl<J> Joystick for StatefulJoystick<J> where J: Joystick {
    type OpenError = J::OpenError;
    fn open(index: JoystickIndex) -> Result<Self, Self::OpenError> {
        let joystick = try!(J::open(index));
        let axes = (0..joystick.num_axes()).map(|_| 0).collect();
        Ok(StatefulJoystick {
            joystick: joystick,
            buttons: 0,
            axes: axes
        })
    }
    fn connected(&self) -> bool {
        self.joystick.connected()
    }
    fn id(&self) -> Cow<str> {
        self.joystick.id()
    }
    fn index(&self) -> JoystickIndex {
        self.joystick.index()
    }
    fn num_axes(&self) -> Axis {
        self.joystick.num_axes()
    }
    fn num_buttons(&self) -> Button {
        self.joystick.num_buttons()
    }
    fn battery(&self) -> Option<f32> {
        self.joystick.battery()
    }
}

/// An iterator over a Backend's event queue.
pub struct Poller<'a, J> where J:Backend+'a {
    backend: &'a mut J
}

impl<'a, C> Iterator for Poller<'a, C> where C:Backend {
    type Item = Event;
    /// This calls the `Backend.poll()` method to poll for the next event
    fn next(&mut self) -> Option<Event> {
        self.backend.poll()
    }
}
