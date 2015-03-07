use libc::{c_char, c_ulong, c_int, c_uint, O_RDONLY, read, strerror};
use std::borrow::{Cow, IntoCow};
use std::ffi::{CStr, CString};
use std::{mem, os, str};
use Joystick;

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
		let c_error = CStr::from_ptr(strerror(num) as *const i8);
		str::from_utf8(c_error.to_bytes()).unwrap()
	}
}

/// Scan for joysticks
pub fn scan() -> Vec<NativeJoystick> {
	use std::fs;
	let mut joysticks = Vec::with_capacity(4);
	for entry in fs::walk_dir("/dev/input/").unwrap() {
		if let Ok(entry) = entry {
			let path = entry.path();
			if let Some(name) = path.file_name() {
				let name = name.to_str().unwrap();
				if name.starts_with("js") {
					let index = name[2..].parse().unwrap();
					if let Ok(js) = Joystick::new(index) {
						joysticks.push(js)
					}
				}
			}
		}
	}
	joysticks
}

/// Represents a system joystick
pub struct NativeJoystick {
	index: u8,
	fd: c_int,
	connected: bool
}

impl ::Joystick for NativeJoystick {
	type WithState = StatefulNativeJoystick;
	type NativeEvent = LinuxEvent;
	/// This tries to open the interface `/dev/input/js...` and will return the
	/// OS-level error if it fails to open this
	fn new(index: u8) -> Result<NativeJoystick, &'static str> {
		let path = format!("/dev/input/js{}", index);
		unsafe {
			let c_path = CString::new(path.as_bytes()).unwrap();
			let fd =  open(c_path.as_ptr(), O_RDONLY | 0x800);
			if fd == -1 {
				Err(os_error())
			} else {
				Ok(NativeJoystick {
					index: index,
					fd: fd,
					connected: true
				})
			}
		}
	}
	/// This reads from the interface in non-blocking mode and converts the native
	/// event into a Reminisce event
	fn poll_native(&mut self) -> Option<LinuxEvent> {
		unsafe {
			let mut event:LinuxEvent = mem::uninitialized();
			loop {
				let event_size = mem::size_of::<LinuxEvent>() as c_ulong;
				if read(self.fd, mem::transmute(&mut event), event_size) == -1 {
					match os::errno() {
						19 => self.connected = false,
						11 => (),
						code =>
							panic!("Error while polling joystick {} - {} - {}", self.index, code, os_error())
					}
					return None
				} else if event._type & 0x80 == 0 {
					return Some(event)
				}
			}
		}
	}
	fn is_connected(&self) -> bool {
		self.connected
	}
	fn get_num_axes(&self) -> u8 {
		unsafe {
			let mut num_axes: c_char = mem::uninitialized();
			ioctl(self.fd as u32, JSIOCGAXES, &mut num_axes as *mut i8);
			num_axes as u8
		}
	}
	fn get_num_buttons(&self) -> u8 {
		unsafe {
			let mut num_buttons: c_char = mem::uninitialized();
			ioctl(self.fd as u32, JSIOCGBUTTONS, &mut num_buttons as *mut i8);
			num_buttons as u8
		}
	}
	fn get_id(&self) -> Cow<str> {
		unsafe {
			use std::ffi::CStr;

			let mut bytes = Vec::with_capacity(JSIOCGID_LEN);
			ioctl(self.fd as u32, JSIOCGID, bytes.as_ptr() as *mut i8);
			let length = CStr::from_ptr(bytes.as_ptr() as *const i8).to_bytes().len();
			String::from_raw_parts(bytes.as_mut_ptr(), length, 64).into_cow()
		}
	}
	fn get_index(&self) -> u8 {
		self.index
	}
	fn with_state(self) -> StatefulNativeJoystick {
		StatefulNativeJoystick::wrap(self)
	}
}

impl Drop for NativeJoystick {
	/// Close the joystick's file descriptor
	fn drop(&mut self) {
		unsafe {
			if close(self.fd) == -1 {
				panic!("Failed to close joystick {} due to {}", self.index, os_error())
			}
		}
	}
}

/// The default joystick that tracks its state
pub struct StatefulNativeJoystick {
	js: NativeJoystick,
	axes: Vec<i16>,
	buttons: Vec<bool>
}
impl StatefulNativeJoystick {
	/// Wrap a joystick
	pub fn wrap(js: NativeJoystick) -> StatefulNativeJoystick {
		StatefulNativeJoystick {
			axes: vec![0; js.get_num_axes() as usize],
			buttons: vec![false; js.get_num_buttons() as usize],
			js: js
		}
	}
}
impl ::Joystick for StatefulNativeJoystick {
	type WithState = StatefulNativeJoystick;
	type NativeEvent = LinuxEvent;
	fn new(index: u8) -> Result<StatefulNativeJoystick, &'static str> {
		let js = try!(::Joystick::new(index));
		Ok(StatefulNativeJoystick::wrap(js))
	}
	fn is_connected(&self) -> bool {
		self.js.is_connected()
	}
	fn get_id(&self) -> Cow<str> {
		self.js.get_id()
	}
	fn get_index(&self) -> u8 {
		self.js.get_index()
	}
	fn get_num_axes(&self) -> u8 {
		self.js.get_num_axes()
	}
	fn get_num_buttons(&self) -> u8 {
		self.js.get_num_buttons()
	}
	fn poll_native(&mut self) -> Option<LinuxEvent> {
		self.js.poll_native()
	}
	fn poll(&mut self) -> Option<::Event> {
		let event = self.js.poll();
		match event {
			Some(::Event::JoystickMoved(i, v)) => self.axes[i as usize] = v,
			Some(::Event::ButtonPressed(i)) => self.buttons[i as usize] = true,
			Some(::Event::ButtonReleased(i)) => self.buttons[i as usize] = false,
			_ => ()
		}
		event
	}
	fn with_state(self) -> StatefulNativeJoystick {
		self
	}
}
impl ::StatefulJoystick for StatefulNativeJoystick {
	fn get_axis(&self, index: ::Axis) -> Option<i16> {
		self.axes.get(index as usize).cloned()
	}
	fn get_button(&self, index: ::Button) -> Option<bool> {
		self.buttons.get(index as usize).cloned()
	}
	fn update(&mut self) {
		while let Some(_) = self.poll() {}
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
		use std::mem::transmute as cast;
		match (self._type, self.value) {
			(1, 0) => ::Event::ButtonReleased(unsafe { cast(self.number) }),
			(1, 1) => ::Event::ButtonPressed(unsafe { cast(self.number) }),
			(2, _) => ::Event::JoystickMoved(unsafe { cast(self.number) }, self.value),
			_ => panic!("Bad type and value {} {} for joystick", self._type, self.value)
		}
	}
}
