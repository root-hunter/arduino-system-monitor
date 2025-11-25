#![no_std]

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
}

impl ArduinoSystem {
    pub fn init() -> Self {
        ArduinoSystem {
            state: ArduinoState::Initializing,
            menu_page: ArduinoMenu::Booting,
        }
    }

    pub fn set_state(&mut self, state: ArduinoState) {
        self.state = state;
    }

    pub fn set_menu_page(&mut self, menu_page: ArduinoMenu) {
        self.menu_page = menu_page;
    }
}

pub struct Metrics {
    pub cpu: u8,
    pub ram: u16,
}

pub struct Status {
    pub battery: u8,
    pub led_on: bool,
}

pub enum Packet {
    Metrics(Metrics),
    Status(Status),
}

pub const PACKET_METRICS: u8 = 0x01;
pub const PACKET_STATUS: u8 = 0x02;