use libc::{c_char, c_ulong, c_int, c_uint, O_RDONLY, read, strerror};
use std::ffi::{c_str_to_bytes, CString};
use std::{mem, os, str};

pub static MAX_JOYSTICK_VALUE:i16 = 32767;
pub static MIN_JOYSTICK_VALUE:i16 = -32767;

static JSIOCGAXES: c_uint = 2147576337;
static JSIOCGBUTTONS: c_uint = 2147576338;
static JSIOCGID: c_uint = 2151705107;
static JSIOCGID_LEN: usize = 64;
extern {
	fn open(path: *const c_char, oflag: c_int) -> c_int;
	fn close(fd: c_int) -> c_int;
	fn ioctl(fd: c_uint, op: c_uint, result: *mut c_char);
}
fn os_error() -> &'static str {
	unsafe {
		let num = os::errno() as c_int;
		let bytes = c_str_to_bytes(mem::transmute(&strerror(num)));
		str::from_utf8(bytes).unwrap()
	}
}
pub struct Joystick {
	id: u8,
	fd: c_int,
	plugged: bool
}
impl Joystick {
	pub fn new(id: u8) -> Result<Joystick, &'static str> {
		let path = format!("/dev/input/js{}", id);
		unsafe {
			let c_path = CString::from_slice(path.as_bytes());
			let fd =  open(c_path.as_ptr(), O_RDONLY | 0x800);
			if fd == -1 {
				Err(os_error())
			} else {
				Ok(Joystick {
					id: id,
					fd: fd,
					plugged: true
				})
			}
		}
	}
	/// Poll this joystick for events
	pub fn poll(&mut self) -> Option<::Event> {
		use ::IntoEvent;
		unsafe {
			let mut event:LinuxEvent = mem::uninitialized();
			loop {
				let event_size = mem::size_of::<LinuxEvent>() as c_ulong;
				if read(self.fd, mem::transmute(&mut event), event_size) == -1 {
					match os::errno() {
						19 => self.plugged = false,
						11 => (),
						code =>
							panic!("Error while polling joystick {} - {} - {}", self.id, code, os_error())
					}
					return None
				} else if event._type & 0x80 == 0 {
					return Some(event.into_event())
				}
			}
		}
	}
	/// Check if this joystick is still plugged in
	pub fn is_plugged(&self) -> bool {
		self.plugged
	}
	/// Get the number of axes supported by this joystick
	pub fn get_num_axes(&self) -> u8 {
		unsafe {
			let mut num_axes: c_char = mem::uninitialized();
			ioctl(self.fd as u32, JSIOCGAXES, &mut num_axes as *mut i8);
			num_axes as u8
		}
	}
	/// Get the number of buttons supported by this joystick
	pub fn get_num_buttons(&self) -> u8 {
		unsafe {
			let mut num_buttons: c_char = mem::uninitialized();
			ioctl(self.fd as u32, JSIOCGBUTTONS, &mut num_buttons as *mut i8);
			num_buttons as u8
		}
	}
	/// Get the pretty identifier of this joystick
	pub fn get_pretty_id(&self) -> String {
		unsafe {
			let text = String::with_capacity(JSIOCGID_LEN);
			ioctl(self.fd as u32, JSIOCGID, text.as_ptr() as *mut i8);
			let new_text = String::from_raw_parts(text.as_ptr() as *mut u8, JSIOCGID_LEN, JSIOCGID_LEN);
			mem::forget(text);
			new_text
		}
	}
	/// Get the numerical identifier of this joystick
	pub fn get_id(&self) -> u8 {
		self.id
	}
}
impl Drop for Joystick {
	fn drop(&mut self) {
		unsafe {
			if close(self.fd) == -1 {
				panic!("Failed to close joystick {} due to {}", self.id, os_error())
			}
		}
	}
}
#[repr(C)]
pub struct LinuxEvent {
	/// timestamp in milleseconds
	time: u32,
	/// value
	value: i16,
	/// event type
	_type: u8,
	/// axis / button number
	number: u8
}
impl ::IntoEvent for LinuxEvent {
	fn into_event(self) -> ::Event {
		match (self._type, self.value) {
			(1, 0) => ::Event::ButtonReleased(self.number),
			(1, 1) => ::Event::ButtonPressed(self.number),
			(2, _) => ::Event::JoystickMoved(self.number, self.value),
			_ => panic!("Bad type and value {} {}", self._type, self.value)
		}
	}
}
