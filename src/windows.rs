use libc::*;
use std::mem;
// I tried
// It's a bit hard to write this without a Windows pc on hand...
extern "stdcall" {
	fn XInputGetCapabilities(index: u32, flags: u32, capabilities: *mut Capabilities) -> i32;
	fn XInputGetState(index: u32, state: *mut State) -> i32;
}

#[repr(C)]
struct State {
	packet_number: i32,
	gamepad: Gamepad
}

#[repr(C)]
struct Capabilities {
	_type: u8,
	sub_type: u8,
	flags: u32,
	gamepad: Gamepad,
	vibration: Vibration
}

bitflags!(
	flags Buttons: u32 {
		const DPAD_UP = 0x0001,
		const DPAD_DOWN = 0x0002,
		const DPAD_LEFT = 0x0004,
		const DPAD_RIGHT = 0x0008,
		const START = 0x0010,
		const BACK = 0x0020,
		const LEFT_THUMB = 0x0040,
		const RIGHT_THUMB = 0x0080,
		const LEFT_SHOULDER = 0x0100,
		const RIGHT_SHOULDER = 0x0200,
		const A = 0x1000,
		const B = 0x2000,
		const X = 0x4000,
		const Y = 0x8000,
	}
);
#[repr(C)]
#[derive(Copy)]
struct Gamepad {
	buttons: Buttons,
	left_trigger: u8,
	right_trigger: u8,
	thumb_lx: i16,
	thumb_ly: i16,
	thumb_rx: i16,
	thumb_ry: i16
}
#[repr(C)]
struct Vibration {
	left_motor_speed: i32,
	right_motor_speed: i32
}

pub struct NativeJoystick {
	index: u8,
	last: Gamepad
}
impl ::Joystick for NativeJoystick {
	fn new(index: u8) -> Result<NativeJoystick, &'static str> {
		unsafe {
			let mut caps:Capabilities = mem::uninitialized();
			XInputGetCapabilities(index as u32, 0, &mut caps);
			Ok(NativeJoystick {
				index: index,
				last: caps.gamepad
			})
		}
	}
	fn get_index(&self) -> u8 {
		self.index
	}
}
impl NativeJoystick {
	pub fn get_axis(&self, index: usize) -> Option<i16> {
		match index {
			0 => Some(self.last.thumb_lx),
			1 => Some(self.last.thumb_ly),
			2 => Some(self.last.thumb_rx),
			3 => Some(self.last.thumb_ry),
			_ => None
		}
	}
	/// Get the value of a specific axis normalised to between -1.0 and 1.0
	pub fn get_normalised_axis(&self, index: usize) -> Option<f32> {
		self.get_axis(index).map(|v| v as f32 / ::MAX_JOYSTICK_VALUE as f32)
	}
	pub fn get_button(&self, index: usize) -> Option<bool> {
		let bits = match index {
			0 => Some(A),
			1 => Some(B),
			2 => Some(X),
			3 => Some(Y),
			_ => None
		};
		bits.map(|v| self.last.buttons.contains(v))
	}
}
