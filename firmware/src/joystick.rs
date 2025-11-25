pub struct ArduinoJoystick {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
}

impl ArduinoJoystick {
    pub fn init() -> Self {
        ArduinoJoystick {
            x: 0,
            y: 0,
            pressed: false,
        }
    }

    pub fn update(&mut self, x: u16, y: u16, pressed: bool) {
        self.x = x;
        self.y = y;
        self.pressed = pressed;
    }
}