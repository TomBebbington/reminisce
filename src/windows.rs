use libc::*;
use std::mem;
// I tried
// It's a bit hard to write this without a Windows pc on hand...
extern "stdcall" {
	/// How many joysticks could be plugged in
	fn joyGetNumDevs() -> c_uint;
	fn joyGetDevCaps(id: c_uint, caps: *mut JoyCaps, caps_size: c_uint) -> MmResult;
	/// Get the state of the joystick
	fn joyGetPos(id: c_uint, info: *mut JoyInfo) -> MmResult;
}

#[repr(C)]
pub struct JoyInfo {
	x: c_uint,
	y: c_uint,
	z: c_uint,
	buttons: c_uint
}

static MAXPNAMELEN: usize = 32;
#[repr(C)]
pub struct JoyCaps {
	man_id: c_uint,
	product_id: c_uint,
	product_name: [c_char; 32],
	x_min: c_uint,
	x_max: c_uint,
	y_min: c_uint,
	y_max: c_uint,
	z_min: c_uint,
	z_max: c_uint,
	num_buttons: c_uint,
	period_min: c_uint,
	period_man: c_uint,
	rudder_min: c_uint,
	rudder_max: c_uint,
	u_min: c_uint,
	u_max: c_uint,
	v_min: c_uint,
	v_max: c_uint,
	caps: c_uint,
	max_axes: c_uint,
	num_axes: c_uint,
	max_buttons: c_uint
}

pub struct Joystick {
	info: JoyInfo,
	caps: JoyCaps
}
impl Joystick {
	pub fn new(id: u8) -> Result<Joystick, &'static str> {
		unsafe {
			let mut info = mem::uninitialized();
			let mut caps = mem::uninitialized();
			match joyGetDevCaps(id as c_uint, &mut caps as *mut JoyCaps, mem::size_of::<JoyCaps>() as c_uint) {
				MmResult::NoError => (),
				MmResult::Error => return Err("Error"),
				MmResult::BadDeviceId => return Err("Bad Device Id"),
				_ => return Err("Unknown Error")
			}
			match joyGetPos(id as c_uint, &mut info as *mut JoyInfo) {
				MmResult::NoError => (),
				MmResult::Error => return Err("Error"),
				MmResult::BadDeviceId => return Err("Bad Device Id"),
				_ => return Err("Unknown Error")
			}
			Ok(Joystick {
				info: info,
				caps: caps
			})
		}
	}
	pub fn get_num_axes(&self) -> u8 {
		self.caps.num_axes as u8
	}
	pub fn get_num_buttons(&self) -> u8 {
		self.caps.num_buttons as u8
	}
	pub fn get_pretty_id(&self) -> String {
		unsafe {
			let name = &self.caps.product_name[..];
			String::from_utf8_lossy(mem::transmute(name)).into_owned()
		}
	}
	pub fn poll(&mut self) -> Option<String> {
		None
	}
}

#[repr(C)]
pub enum MmResult {
	NoError,
	Error,
	BadDeviceId,
	NotEnabled,
	Allocated,
	InvalHandle,
	NoDriver,
	NoMem,
	NotSupported,
	BadErrNum,
	InvalFlag,
	InvalParam,
	HandleBusy,
	InvalidAlias,
	BadDb,
	KeyNotFound,
	ReadError,
	WriteError,
	DeleteError,
	ValNotFound,
	NoDiverCb
}
