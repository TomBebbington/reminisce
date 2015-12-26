use libc::{c_char, c_ulong, c_int, c_uint, O_RDONLY, O_NONBLOCK, read};
use glob::glob;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::io::Error;
use std::mem;
use {Backend, Event, Joystick};

static JSIOCGAXES: c_uint = 2147576337;
static JSIOCGBUTTONS: c_uint = 2147576338;
static JSIOCGID: c_uint = 2151705107;
static JSIOCGID_LEN: usize = 64;

extern {
	fn open(path: *const c_char, oflag: c_int) -> c_int;
	fn close(fd: c_int) -> c_int;
	fn ioctl(fd: c_uint, op: c_uint, result: *mut c_char);
}

pub struct Native {
	joysticks: Vec<NativeJoystick>,
	pending: Vec<Event>
}
impl Backend for Native {
	type Joystick = NativeJoystick;
	fn new() -> Native {
		let mut joysticks = Vec::with_capacity(4);
		for entry in glob("/dev/input/js*").unwrap() {
			if let Ok(path) = entry {
				if let Some(name) = path.file_name() {
					if let Some(name) = name.to_str() {
						if name.starts_with("js") {
							if let Ok(index) = name[2..].parse() {
								if let Ok(js) = Joystick::open(index) {
									joysticks.push(js)
								}
							}
						}
					}
				}
			}
		}
		let pending = joysticks.iter().by_ref().map(|js:&NativeJoystick| Event::Connected(js.index)).collect();
		Native {
			joysticks: joysticks,
			pending: pending
		}
	}
	fn num_joysticks(&self) -> usize {
		return self.joysticks.len();
	}
	fn joysticks(&self) -> &[NativeJoystick] {
		return &self.joysticks;
	}
	fn poll(&mut self) -> Option<Event> {
		match self.pending.pop() {
			Some(Event::Disconnected(i)) => {
				self.joysticks.remove(i as usize);
				Some(Event::Disconnected(i))
			}
			Some(v) => Some(v),
			None => self.joysticks.iter_mut().filter(|js| js.connected).flat_map(|js| js.poll()).next()
		}
	}
}
/// Represents a system joystick
pub struct NativeJoystick {
	index: u8,
	fd: c_int,
	connected: bool
}

impl NativeJoystick {
	fn poll(&mut self) -> Option<Event> {
		unsafe {
			let mut event:LinuxEvent = mem::uninitialized();
			loop {
				let event_size = mem::size_of::<LinuxEvent>() as c_ulong;
				if read(self.fd, mem::transmute(&mut event), event_size as usize) == -1 {
					let err = Error::last_os_error();
					match Error::last_os_error().raw_os_error().expect("Bad OS Error") {
						19 => {
							self.connected = false;
							return Some(Event::Disconnected(self.index))
						},
						11 => (),
						_ => panic!("{}", err)
					}
				} else if event._type & 0x80 == 0 {
					return Some(match (event._type, event.value) {
						(1, 0) => Event::ButtonReleased(self.index, event.number),
						(1, 1) => Event::ButtonPressed(self.index, event.number),
						(2, _) => Event::AxisMoved(self.index, event.number, event.value),
						_ => panic!("Bad type and value {} {} for joystick", event._type, event.value)
					})
				}
			}
		}
	}
}

impl ::Joystick for NativeJoystick {
	type OpenError = Error;
	/// This tries to open the interface `/dev/input/js...` and will return the
	/// OS-level error if it fails to open this
	fn open(index: u8) -> Result<NativeJoystick, Error> {
		let path = format!("/dev/input/js{}", index);
		unsafe {
			let c_path = CString::new(path.as_bytes()).unwrap();
			let fd = open(c_path.as_ptr(), O_RDONLY | O_NONBLOCK);
			if fd == -1 {
				Err(Error::last_os_error())
			} else {
				Ok(NativeJoystick {
					index: index,
					fd: fd,
					connected: true
				})
			}
		}
	}
	fn connected(&self) -> bool {
		self.connected
	}
	fn num_axes(&self) -> u8 {
		unsafe {
			let mut num_axes: c_char = mem::uninitialized();
			ioctl(self.fd as u32, JSIOCGAXES, &mut num_axes as *mut i8);
			num_axes as u8
		}
	}
	fn num_buttons(&self) -> u8 {
		unsafe {
			let mut num_buttons: c_char = mem::uninitialized();
			ioctl(self.fd as u32, JSIOCGBUTTONS, &mut num_buttons as *mut i8);
			num_buttons as u8
		}
	}
	fn id(&self) -> Cow<str> {
		unsafe {
			let text = String::with_capacity(JSIOCGID_LEN);
			ioctl(self.fd as u32, JSIOCGID, text.as_ptr() as *mut i8);
			let mut new_text = String::from_raw_parts(text.as_ptr() as *mut u8, JSIOCGID_LEN, JSIOCGID_LEN);
			let length = CStr::from_ptr(text.as_ptr() as *const i8).to_bytes().len();
			mem::forget(text);
			new_text.truncate(length);
			new_text.shrink_to_fit();
			new_text.into()
		}
	}
	fn index(&self) -> u8 {
		self.index
	}
	/// This is not supported on Linux so None is returned every time
	fn battery(&self) -> Option<f32> {
		None
	}
}

impl Drop for NativeJoystick {
	/// Close the joystick's file descriptor
	fn drop(&mut self) {
		unsafe {
			if close(self.fd) == -1 {
				let error = Error::last_os_error();
				panic!("Failed to close joystick {} due to {}", self.index, error)
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