use {Axis, Button, Event, Joystick, StatefulJoystick};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::rc::Rc;

/// A Joystick mapper
///
/// This allows you to map a joystick's buttons and axes
pub struct JoystickMapper<J> where J:Joystick {
    joystick: J,
    axes: Rc<BTreeMap<Axis, Axis>>,
    buttons: Rc<BTreeMap<Button, Button>>
}
impl<J> JoystickMapper<J> where J:Joystick {

    /// Start mapping a joystick
    pub fn new(joystick: J) -> JoystickMapper<J> {
        JoystickMapper {
            joystick: joystick,
            axes: Rc::new(BTreeMap::new()),
            buttons: Rc::new(BTreeMap::new())
        }
    }

    /// Map a button to another button
    pub fn map_button(&mut self, from: Button, to: Button) {
        self.buttons.make_unique().insert(from, to);
    }

    /// Map a multitude of buttons to other buttons
    ///
    /// This is more efficient than doing them inidividually
    pub fn map_buttons<I>(&mut self, iter: I) where I:Iterator<Item = (Button, Button)> {
        let mut buttons = self.buttons.make_unique();
        for (from, to) in iter {
            buttons.insert(from, to);
        }
    }

    /// Map an axis to another axis
    pub fn map_axis(&mut self, from: Axis, to: Axis) {
        self.axes.make_unique().insert(from, to);
    }

    /// Map a multitude of axes to other axes
    ///
    /// This is more efficient than doing them inidividually
    pub fn map_axes<I>(&mut self, iter: I) where I:Iterator<Item = (Axis, Axis)> {
        let mut axes = self.axes.make_unique();
        for (from, to) in iter {
            axes.insert(from, to);
        }
    }

}
impl<J> Joystick for JoystickMapper<J> where J:Joystick {
    type WithState = JoystickMapper<<J as Joystick>::WithState>;
    type NativeEvent = <J as Joystick>::NativeEvent;
    type OpenError = <J as Joystick>::OpenError;

    fn open(index: u8) -> Result<JoystickMapper<J>, <J as Joystick>::OpenError> {
        Ok(JoystickMapper {
            joystick: try!(Joystick::open(index)),
            axes: Rc::new(BTreeMap::new()),
            buttons: Rc::new(BTreeMap::new())
        })
    }

    fn is_connected(&self) -> bool {
        self.joystick.is_connected()
    }
    fn get_id(&self) -> Cow<str> {
        self.joystick.get_id()
    }
    fn get_index(&self) -> u8 {
        self.joystick.get_index()
    }
    fn get_num_axes(&self) -> u8 {
        self.joystick.get_num_axes()
    }
    fn get_num_buttons(&self) -> u8 {
        self.joystick.get_num_buttons()
    }
    fn get_battery(&self) -> Option<f32> {
        self.joystick.get_battery()
    }
    fn poll_native(&mut self) -> Option<<J as Joystick>::NativeEvent> {
        self.joystick.poll_native()
    }
    fn poll(&mut self) -> Option<Event> {
        self.joystick.poll().map(|event|
            match event {
                Event::ButtonPressed(mut btn) => {
                    if let Some(&button) = self.buttons.get(&btn) {
                        btn = button
                    }
                    Event::ButtonPressed(btn)
                },
                Event::ButtonReleased(mut btn) => {
                    if let Some(&button) = self.buttons.get(&btn) {
                        btn = button
                    }
                    Event::ButtonReleased(btn)
                },
                Event::AxisMoved(mut axis, value) => {
                    if let Some(&new_axis) = self.axes.get(&axis) {
                        axis = new_axis
                    }
                    Event::AxisMoved(axis, value)
                },
            })
    }
    fn with_state(self) -> JoystickMapper<<J as Joystick>::WithState> {
        JoystickMapper {
            joystick: self.joystick.with_state(),
            axes: self.axes.clone(),
            buttons: self.buttons.clone()
        }
    }
}
impl<J> StatefulJoystick for JoystickMapper<J> where J:StatefulJoystick {
    fn get_axis(&self, mut axis: Axis) -> Option<i16> {
        use StatefulJoystick;
        if let Some(&new_axis) = self.axes.get(&axis) {
            axis = new_axis
        }
        self.joystick.get_axis(axis)
    }
    fn get_button(&self, mut button: Button) -> Option<bool> {
        use StatefulJoystick;
        if let Some(&btn) = self.buttons.get(&button) {
            button = btn
        }
        self.joystick.get_button(button)
    }
    fn update(&mut self) {
        use StatefulJoystick;
        self.joystick.update()
    }
}
