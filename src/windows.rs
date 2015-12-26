use std::borrow::Cow;
use std::collections::VecDeque;
use std::error::Error;
use std::mem;
// I tried
// It's a bit hard to write this without a Windows pc on hand...
// But this should work on Windows Vista, 7, and 8
#[link(name = "XInput9_1_0")]
extern "stdcall" {
	fn XInputGetCapabilities(index: u32, flags: u32, capabilities: *mut Capabilities) -> i32;
	fn XInputGetState(index: u32, state: *mut State) -> i32;
	fn XInputGetBatteryInformation(index: u32, ty: u8, information: *mut Battery) -> i32;

}

#[repr(u8)]
enum BatteryLevel {
	Empty,
	Low,
	Medium,
	High
}

#[repr(u8)]
enum BatteryType {
	Disconnected,
	Wired,
	Alkaline,
	Nimh,
	Unknown = 0xFF
}

#[repr(C)]
struct Battery {
	_type: BatteryType,
	level: BatteryLevel
}

#[repr(C)]
struct State {
	packet: i32,
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

/// Scan for joysticks
pub fn scan() -> Vec<NativeJoystick> {
	(0..4).filter_map(|i| ::Joystick::open(i).ok()).collect()
}

pub struct NativeJoystick {
	index: u8,
	last: Gamepad,
	last_packet: i32,
	events: VecDeque<::Event>
}
impl ::Joystick for NativeJoystick {
	type WithState = NativeJoystick;
	type NativeEvent = ::Event;
	type Error = Error;
	fn open(index: u8) -> Result<NativeJoystick, &'static str> {
		unsafe {
			let mut caps: Capabilities = mem::uninitialized();
			let code = XInputGetCapabilities(index as u32, 0, &mut caps);
			if code == 0 {
				Ok(NativeJoystick {
					index: index,
					last: caps.gamepad,
					last_packet: 0,
					events: VecDeque::with_capacity(10)
				})
			} else {
				Err(Error::from_os_error(code))
			}
		}
	}
	fn poll_native(&mut self) -> Option<::Event> {
		use ::StatefulJoystick;
		self.update();
		self.events.pop_back()
	}
	fn connected(&self) -> bool {
		true
	}
	fn battery(&self) -> Option<f32> {
		unsafe {
			let mut battery = mem::uninitialized();
			XInputGetBatteryInformation(self.index as u32, 0, &mut battery);
			match battery._type {
				BatteryType::Wired | BatteryType::Disconnected =>
					None,
				_ =>
					Some(match battery.level {
						BatteryLevel::Empty => 0.0,
						BatteryLevel::Low => 0.25,
						BatteryLevel::Medium => 0.5,
						BatteryLevel::High => 1.0
					})
			}
		}
	}
	fn id(&self) -> Cow<str> {
		"XInput Device".into()
	}
	fn index(&self) -> u8 {
		self.index
	}
	fn num_buttons(&self) -> u8 {
		4
	}
	fn num_axes(&self) -> u8 {
		4
	}
	fn with_state(self) -> NativeJoystick {
		self
	}
}
macro_rules! event{
	(button $this:expr, $now:expr, $last:expr, $btn:expr, $id:expr) => (
		{
			let now_btn = $now.buttons.contains($btn);
			let last_btn = $last.buttons.contains($btn);
			if now_btn && !last_btn {
				$this.events.push_back(::Event::ButtonPressed($id))
			} else if !now_btn && last_btn {
				$this.events.push_back(::Event::ButtonReleased($id))
			}
		}
	);
	(axis $this:expr, $now:expr, $last:expr, $field:ident, $id:expr) => (
		if $now.$field != $last.$field {
			$this.events.push_back(::Event::AxisMoved($id, $now.$field))
		}
	);
	(axes $this:expr, $now:expr, $last:expr, $($field:ident => $id:expr),+) => ({
		$(event!(axis $this, $now, $last, $field, $id);)+
	});
	(buttons $this:expr, $now:expr, $last:expr, $($btn:ident => $id:expr),+) => ({
		$(event!(button $this, $now, $last, $btn, $id);)+
	});
}
impl ::StatefulJoystick for NativeJoystick {
	fn axis(&self, index: ::Axis) -> Option<i16> {
		match index {
			::Axis::LeftX => Some(self.last.thumb_lx),
			::Axis::LeftY => Some(self.last.thumb_ly),
			::Axis::RightX => Some(self.last.thumb_rx),
			::Axis::RightY => Some(self.last.thumb_ry),
			_ => None
		}
	}
	fn button(&self, index: ::Button) -> Option<bool> {
		let bits = match index {
			::Button::A => Some(A),
			::Button::B => Some(B),
			::Button::X => Some(X),
			::Button::Y => Some(Y),
			::Button::LeftShoulder => Some(LEFT_SHOULDER),
			::Button::RightShoulder => Some(RIGHT_SHOULDER),
			::Button::Select => Some(BACK),
			::Button::Start => Some(START),
			::Button::LeftStick => Some(LEFT_THUMB),
			::Button::RightStick => Some(RIGHT_THUMB),
			_ => None
		};
		bits.map(|v| self.last.buttons.contains(v))
	}
	fn update(&mut self) {
		let state = unsafe {
			let mut state: State = mem::uninitialized();
			XInputGetState(self.index as u32, &mut state);
			state
		};
		let now = state.gamepad;
		if state.packet != self.last_packet {
			let last = self.last;
			event!{axes self, now, last,
				thumb_lx => ::Axis::LeftX,
				thumb_ly => ::Axis::LeftY,
				thumb_rx => ::Axis::RightX,
				thumb_ry => ::Axis::RightY
			}
			event!{buttons self, now, last,
				A => ::Button::A,
				B => ::Button::B,
				X => ::Button::X,
				Y => ::Button::Y,
				LEFT_SHOULDER => ::Button::LeftShoulder,
				RIGHT_SHOULDER => ::Button::RightShoulder,
				BACK => ::Button::Select,
				START => ::Button::Start,
				LEFT_THUMB => ::Button::LeftStick,
				RIGHT_THUMB => ::Button::RightStick
			}
		}
		self.last_packet = state.packet;
		self.last = now;
	}
}
