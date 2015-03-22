//! Reminisce is a lightweight library intended to be used for detecting and
//! reading from joysticks.
#![cfg_attr(feature = "mappings", feature(alloc))]
#![cfg_attr(target_os = "linux", feature(core, libc, fs_walk))]
extern crate libc;
#[cfg(target_os = "windows")]
#[macro_use] extern crate rustc_bitflags;

#[cfg(feature = "sdl")]
extern crate sdl2;

/// Someday, somehow
#[cfg(all(feature = "emscripten", not(feature = "sdl")))]
pub mod emscripten;

#[cfg(all(feature = "emscripten", not(feature = "sdl")))]
pub use emscripten as native;

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

#[cfg(feature = "mappings")]
pub mod mapper;

#[cfg(feature = "mappings")]
pub use mapper::JoystickMapper;
pub use native::{NativeJoystick, scan};


/// The maximum axis value
pub static MAX_AXIS_VALUE:i16 = 32767;
/// The minimum axis value
pub static MIN_AXIS_VALUE:i16 = -32767;

use std::borrow::Cow;
use std::mem::transmute as cast;

#[cfg(feature = "mappings")]
macro_rules! text_enum(
    ($name:ident, $($enumer:ident => $text:expr),+) => (
        impl ::std::fmt::Display for $name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(fmt, "{}", match *self {
                    $($name::$enumer => $text),+
                })
            }
        }
        impl ::std::str::FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<$name, String> {
                match s {
                    $($text => Ok($name::$enumer),)+
                    _ => Err(format!("Could not make {} from {}", stringify!($name), s))
                }
            }
        }
    )
);
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A direction on a joystick
///
/// This uses the order that Linux drivers and the HTML5 Gamepad API uses so it should ring a bell
pub enum Axis {
	/// The x direction of the left stick
    ///
    /// This is usually used for horizontal movement in a game
    LeftX,
	/// The y direction of the left stick
    ///
    /// This is usually used for vertical movement in a game
    LeftY,
	/// The x direction of the right stick
    //
    /// This is usually used for looking around in a game
    RightX,
	/// The y direction of the right stick
    //
    /// This is usually used for looking around in a game
    RightY,
	/// How far down the left trigger is pressed
    ///
    /// This is only used as a button on some platforms so don't rely on just this
    TriggerLeft,
	/// How far down the right trigger is pressed
    ///
    /// This is only used as a button on some platforms so don't rely on just this
    TriggerRight
}
#[cfg(feature = "mappings")]
text_enum!(Axis,
    LeftX => "leftx",
    LeftY => "lefty",
    RightX => "rightx",
    RightY => "righty",
    TriggerLeft => "triggerleft",
    TriggerRight => "triggerright"
);

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A button on a joystick
///
/// This uses the order that Linux drivers and the HTML5 Gamepad API uses so it should be familiar
pub enum Button {
	/// The A button
    ///
    /// This is typically used for jumping
	A,
	/// The B button
    ///
    /// This is typically used for hitting
	B,
	/// The X button
	X,
	/// The Y button
	Y,
	/// The left top shoulder button
    ///
    /// This is usually used in FPS games for throwing a certain kind of grenade
	LeftShoulder,
	/// The right top shoulder button
	RightShoulder,
	/// The left bottom shoulder / trigger button
    ///
    /// This is typically used in FPS games for aiming down the sights in a weapon
	LeftTrigger,
	/// The right bottom shoulder / trigger button
    ///
    /// This is typically used in FPS games for shooting with a weapon
	RightTrigger,
	/// The back / select button
    ///
    /// This is usually used as an alternate pause menu or to open a menu in-game.
	Select,
	/// The start / forward button
    ///
    /// This is usually used to start and pause a game
	Start,
	/// The left stick button
	LeftStick,
	/// The right stick button
	RightStick,
	/// The up button on the directional pad
	DPadUp,
	/// The down button on the directional pad
	DPadDown,
	/// The left button on the directional pad
	DPadLeft,
	/// The right button on the directional pad
	DPadRight
}
#[cfg(feature = "mappings")]
text_enum!(Button,
    A => "a",
    B => "b",
    X => "x",
    Y => "y",
    LeftShoulder => "leftshoulder",
    RightShoulder => "rightshoulder",
    LeftTrigger => "lefttrigger",
    RightTrigger => "righttrigger",
    Select => "select",
    Start => "start",
    LeftStick => "leftstick",
    RightStick => "rightstick",
    DPadUp => "dpadup",
    DPadDown => "dpaddown",
    DPadLeft => "dpadleft",
    DPadRight => "dpadright"
);

#[derive(Copy, Debug, Eq, PartialEq)]
/// An event from a joystick
pub enum Event {
	/// Fired when a button is pressed with the button's index
	ButtonPressed(Button),
	/// Fired when a button is released with the button's index
	ButtonReleased(Button),
	/// Fired when a axis is moved with the axis index and its value,
	/// which is between `MIN_JOYSTICK_VALUE` and `MAX_JOYSTICK_VALUE`
	AxisMoved(Axis, i16)
}
/// Convert a raw event into a Reminisce event
pub trait IntoEvent {
	/// Convert this into a Reminisce event
	fn into_event(self) -> Event;
}

impl IntoEvent for Event {
    /// Convert the event into itself
    ///
    /// The only reason this exists is because windows doesn't have events so the backend has to
    /// make reminisce events directly and pretend that they are the native events.
    fn into_event(self) -> Event {
        self
    }
}

/// A joystick or gamepad
///
/// Each `Joystick` has its own event queue that can be polled repeatedly
/// using the `iter()` method or by calling the `poll()` method repeatedly.
///
/// ``` rust
/// use reminisce::{scan, Axis, Event, Joystick};
/// let mut joysticks = scan();
/// let (mut x, mut y) = (0, 0);
/// for joystick in &mut joysticks {
///     for event in joystick.iter() {
///         match event {
///             Event::AxisMoved(Axis::LeftX, nx) =>
///                 x += nx,
///             Event::AxisMoved(Axis::LeftY, ny) =>
///                 y += ny,
///             _ => ()
///         }
///     }
/// }
/// println!("{}, {}", x, y);
/// ```
pub trait Joystick : Sized {
	/// The version of this joystick that includes the state
    ///
    /// On Linux, this is a wrapper around the joystick, but on Windows,
    /// this is the same stucture because the joystick always includes
    /// state on it.
	type WithState : StatefulJoystick;

    /// The event that this joystick processes
    type NativeEvent: IntoEvent;

	/// Create a new joystick from its index
    ///
    /// If an error occurs, this will return the textual representation of that error.
    ///
    /// ``` rust
    /// use reminisce::{NativeJoystick, Joystick};
    /// let joystick:Result<NativeJoystick, &str> = Joystick::new(0);
    /// if let Ok(joystick) = joystick {
    ///     println!("{}", joystick.get_id())
    /// } else {
    ///     println!("No joystick plugged in")
    /// }
    /// ```
	fn new(index: u8) -> Result<Self, &'static str>;

	/// Check if the joystick is still connected
	fn is_connected(&self) -> bool;

	/// Get the identifier of this joystick
    ///
    /// This is a cow because it can be a static or owned String depending on the
    /// implementation.
	fn get_id(&self) -> Cow<str>;

	/// Get the index of this joystick
	fn get_index(&self) -> u8;

	/// Get the number of axes this joystick has
    ///
    /// This is capped at 6 axes for now.
	fn get_num_axes(&self) -> u8;

	/// Get the number of buttons this joystick has
    ///
    /// This is capped at 16 buttons currently.
	fn get_num_buttons(&self) -> u8;

    /// Get the battery level of this joystick
    ///
    /// Returns none if the joystick is wired or this operation is not supported
    /// by the backend
    fn get_battery(&self) -> Option<f32>;

    /// Poll the joystick for events in non-blocking mode and return the native event
    /// as returned from the backend
	fn poll_native(&mut self) -> Option<Self::NativeEvent>;

    /// Poll the joystick for events in non-blocking mode
    ///
    /// This runs `self.poll_native()` then converts it into an event using
    /// the `into_event` method.
    fn poll(&mut self) -> Option<Event> {
        self.poll_native().map(|e| e.into_event())
    }

    /// Map the axes and buttons of this joystick by wrapping it in a `JoystickMapper`
    #[cfg(feature = "mappings")]
    fn into_mapper(self) -> JoystickMapper<Self> {
        JoystickMapper::new(self)
    }

    /// Iterate through the events that haven't been processed yet
    fn iter(&mut self) -> Poller<Self> {
        Poller {
            joystick: self
        }
    }

	/// Get the version of this joystick which includes state
    ///
    /// On the Linux backend, a wrapper is made, but on the Windows
    /// backend no wrapper is needed so this just returns the
    ///n `NativeJoystick` structure
	fn with_state(self) -> Self::WithState;
}

/// A joystick that keeps a record of its state
/// This makes it really easy to write the input for games, etc because its just a matter
/// of calling a single method to query where the axes are
///
/// ``` rust
/// use reminisce::{scan, Axis, NativeJoystick, Joystick, StatefulJoystick};
/// let mut joysticks = scan().into_iter().map(|js| js.with_state()).collect::<Vec<_>>();
/// for joystick in &joysticks {
///     let x = joystick.get_axis(Axis::LeftX).unwrap_or(0);
///     let y = joystick.get_axis(Axis::LeftY).unwrap_or(0);
///     println!("{}, {}", x, y)
/// }
/// ```
pub trait StatefulJoystick : Joystick + Sized {
	/// Get the value of a specific axis as an integer between `MIN_AXIS_VALUE` and `MAX_AXIS_VALUE`
	fn get_axis(&self, index: Axis) -> Option<i16>;

	/// Get the value of a specific axis normalised to between -1.0 and 1.0
	fn get_normalised_axis(&self, index: Axis) -> Option<f32> {
		self.get_axis(index).map(|v| v as f32 / MAX_AXIS_VALUE as f32)
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

	/// Update the state of this joystick by polling the native backend
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
    /// This calls the `joystick.poll()` method to poll for the next event
	fn next(&mut self) -> Option<Event> {
		self.joystick.poll()
	}
}
