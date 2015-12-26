use sdl2::joystick::*;
use sdl2::{init, event, Sdl, JoystickSubsystem, ErrorMessage};

use std::borrow::Cow;

use {Context, Event};

pub struct NativeContext {
    sdl: Sdl,
    system: JoystickSubsystem,
    joysticks: Vec<NativeJoystick>
}
impl Context for NativeContext {
    type Joystick = NativeJoystick;
    fn new() -> Self {
        let sdl = init().unwrap();
        let system = sdl.joystick().unwrap();
        NativeContext {
            sdl: sdl,
            system: system,
            joysticks: Vec::new()
        }
    }
    fn num_joysticks(&self) -> usize {
        self.system.num_joysticks().unwrap_or(0) as usize
    }
    fn get_joysticks(&self) -> &[NativeJoystick] {
        &self.joysticks
    }
    fn poll(&mut self) -> Option<Event> {
        self.sdl.event_pump().unwrap().poll_iter().filter_map(|e| match e {
            event::Event::JoyDeviceAdded { which , ..} => {
                self.joysticks.push(self.system.open(which as u32).unwrap());
                Some(Event::Connected(which as ::JoystickIndex))
            },
            event::Event::JoyDeviceRemoved { which, .. } => {
                self.joysticks.remove(which as usize);
                Some(Event::Disconnected(which as ::JoystickIndex))
            },
            event::Event::JoyButtonDown { which, button_idx, .. } =>
                Some(Event::ButtonPressed(which as ::JoystickIndex, button_idx)),
            event::Event::JoyButtonUp { which, button_idx, .. } =>
                Some(Event::ButtonReleased(which as ::JoystickIndex, button_idx)),
            _ => None,
        }).next()
    }
}


/// A native joystick using SDL
pub type NativeJoystick = Joystick;

impl ::Joystick for NativeJoystick {
    type OpenError = ErrorMessage;
    fn open(index: u8) -> Result<NativeJoystick, ErrorMessage> {
        init().unwrap().joystick().unwrap().open(index as u32)
    }
    fn connected(&self) -> bool {
        self.attached()
    }
    fn index(&self) -> u8 {
        self.instance_id() as u8
    }
    fn id(&self) -> Cow<str> {
        self.name().into()
    }
    fn num_buttons(&self) -> u8 {
        self.num_buttons() as u8
    }
    fn num_axes(&self) -> u8 {
        self.num_axes() as u8
    }
    fn battery(&self) -> Option<f32> {
        None
    }
}