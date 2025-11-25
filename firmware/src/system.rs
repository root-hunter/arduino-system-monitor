use crate::joystick::Joystick;

#[derive(PartialEq)]
pub enum State {
    Initializing,
    Running,
    Error,
}

#[derive(PartialEq)]
pub enum Menu {
    Booting,
    Home,
    System,
    Data,
    Monitor,
    JoystickTest,
}

pub struct System {
    pub state: State,
    pub menu_page: Menu,
    pub joystick: Joystick,
}

impl System {
    pub fn init() -> Self {
        System {
            state: State::Initializing,
            menu_page: Menu::Booting,
            joystick: Joystick::init(),
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn set_menu_page(&mut self, menu_page: Menu) {
        self.menu_page = menu_page;
    }
}