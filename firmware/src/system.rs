use arduino_hal::Peripherals;
use avr_device::interrupt;

use crate::joystick::Joystick;

static mut TICKS: u32 = 0;

#[interrupt(atmega328p)]
fn TIMER0_COMPA() {
    unsafe {
        TICKS += 1;
    }
}

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

    pub fn init_clock() {
        let dp = unsafe { Peripherals::steal() };

        dp.TC0.tccr0a.write(|w| w.wgm0().bits(2)); // CTC mode
        dp.TC0.tccr0b.write(|w| w.cs0().prescale_64());
        dp.TC0.ocr0a.write(|w| unsafe { w.bits(249) }); // 16_000_000 / 64 / 250 = 1kHz
        dp.TC0.timsk0.write(|w| w.ocie0a().set_bit());

        unsafe { avr_device::interrupt::enable() };
    }

    pub fn get_ticks() -> u32 {
        unsafe { TICKS }
    }
}
