const JOYSTICK_X_MIN: u16 = 0;
const JOYSTICK_X_MAX: u16 = 1023;
const JOYSTICK_Y_MIN: u16 = 0;
const JOYSTICK_Y_MAX: u16 = 1023;

pub struct Joystick {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
}

impl Joystick {
    pub fn init() -> Self {
        Joystick {
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