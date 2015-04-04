use libc::*;
use std::borrow::Cow;
use std::mem;
use std::mem::transmute as cast;
use std::io::Error;
use std::ffi::CStr;
use std::collections::VecDeque;
use Joystick;

extern {
    fn emscripten_get_num_gamepads() -> c_int;
    fn emscripten_get_gamepad_status(index: c_int, state: *mut NativeEvent) -> c_int;
}

#[repr(C)]
pub struct NativeEvent {
    timestamp: f64,
    num_axes: c_int,
    num_buttons: c_int,
    axis: [f64; 64],
    analog_button: [f64; 64],
    digital_button: [bool; 64],
    connected: bool,
    index: c_long,
    id: [c_char; 64],
    mapping: [c_char; 64]
}

pub struct NativeJoystick {
    last: NativeEvent,
	events: VecDeque<::Event>
}

fn os_error(code: c_int) -> &'static str {
    match code {
        1 => "Deferred",
        -1 => "Not supported",
        -2 => "Failed not deferred",
        -3 => "Invalid target",
        -4 => "Unknown target",
        -5 => "Invalid param",
        -6 => "Failed",
        -7 => "No data",
        _ => "No error"
    }
}

impl ::Joystick for NativeJoystick {
    type WithState = NativeJoystick;
    type NativeEvent = ::Event;
    type OpenError = Error;
    fn open(index: u8) -> Result<NativeJoystick, Error> {
        unsafe {
            let mut state = mem::uninitialized();
            let code = emscripten_get_gamepad_status(index as c_int, &mut state);
            if code != 0 {
                Err(Error::last_os_error())
            } else {
                Ok(NativeJoystick {
                    last: state,
                    events: VecDeque::with_capacity(16)
                })
            }
        }
    }
    fn is_connected(&self) -> bool {
        self.last.connected
    }
    fn get_id(&self) -> Cow<str> {
        unsafe {
            use std::str;
            let bytes = CStr::from_ptr(self.last.id.as_ptr()).to_bytes();
            str::from_utf8_unchecked(bytes).into()
        }
    }
    fn get_battery(&self) -> Option<f32> {
        None
    }
    fn get_index(&self) -> u8 {
        self.last.index as u8
    }
    fn get_num_axes(&self) -> u8 {
        self.last.num_axes as u8
    }
    fn get_num_buttons(&self) -> u8 {
        self.last.num_buttons as u8
    }
    fn with_state(self) -> NativeJoystick {
        self
    }
	fn poll_native(&mut self) -> Option<::Event> {
		use ::StatefulJoystick;
		self.update();
		self.events.pop_back()
	}
}
impl ::StatefulJoystick for NativeJoystick {
    fn get_axis(&self, index: ::Axis) -> Option<i16> {
        self.get_normalised_axis(index).map(|axis| (axis * ::MAX_AXIS_VALUE as f32) as i16)
    }
    fn get_normalised_axis(&self, index: ::Axis) -> Option<f32> {
        let index = unsafe { cast(index) };
        if index < self.last.num_axes as u8 && index < 64 {
            Some(self.last.axis[index as usize] as f32)
        } else {
            None
        }
    }
    fn get_button(&self, index: ::Button) -> Option<bool> {
        let index = unsafe { cast(index) };
        if index < self.last.num_buttons as u8 && index < 64 {
          Some(self.last.digital_button[index as usize])
        } else {
          None
        }
    }
    fn update(&mut self) {
        unsafe {
            let mut state = mem::uninitialized();
            emscripten_get_gamepad_status(self.last.index as i32, &mut state);
            {
                let now = &state;
                let last = &self.last;
                for button in (0..now.num_buttons) {
                    let now_btn = now.digital_button[button as usize];
                    let last_btn = last.digital_button[button as usize];
                    if now_btn && !last_btn {
                        let btn = cast(button as u8);
            			self.events.push_back(::Event::ButtonPressed(btn))
                    } else if !now_btn && last_btn {
                        let btn = cast(button as u8);
            			self.events.push_back(::Event::ButtonReleased(btn))
                    }
                }
                for axis in (0..now.num_axes) {
                    let now_axis = now.axis[axis as usize];
                    let last_axis = last.axis[axis as usize];
                    if now_axis != last_axis {
                        let axis = cast(axis as u8);
                		self.events.push_back(::Event::AxisMoved(axis, (now_axis as f32 * ::MAX_AXIS_VALUE as f32) as i16))
                    }
                }
            }
            self.last = state;
        }
    }
}

/// Scan for joysticks
pub fn scan() -> Vec<NativeJoystick> {
    let count = unsafe { emscripten_get_num_gamepads() };
    let mut joysticks = Vec::with_capacity(count as usize);
    for i in (0..count) {
        joysticks.push(Joystick::open(i as u8).unwrap())
    }
    joysticks
}
