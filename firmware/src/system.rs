use crate::joystick::ArduinoJoystick;

#[derive(PartialEq)]
pub enum ArduinoState {
    Initializing,
    Running,
    Error,
}

#[derive(PartialEq)]
pub enum ArduinoMenu {
    Booting,
    Home,
    System,
    Data,
    Monitor,
    JoystickTest,
}

pub struct ArduinoSystem {
    pub state: ArduinoState,
    pub menu_page: ArduinoMenu,
    pub joystick: ArduinoJoystick,
}

impl ArduinoSystem {
    pub fn init() -> Self {
        ArduinoSystem {
            state: ArduinoState::Initializing,
            menu_page: ArduinoMenu::Booting,
            joystick: ArduinoJoystick::init(),
        }
    }

    pub fn set_state(&mut self, state: ArduinoState) {
        self.state = state;
    }

    pub fn set_menu_page(&mut self, menu_page: ArduinoMenu) {
        self.menu_page = menu_page;
    }
}