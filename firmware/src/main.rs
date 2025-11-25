#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod display;
mod joystick;
mod protocol;
mod system;

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use asm_common::Packet;
use avr_device::interrupt;
use panic_halt as _;
use ufmt::uwriteln;

use crate::protocol::DeserializePacket;
use crate::system::Menu;
use crate::system::State;
use crate::system::System;

pub fn update_joystick(system: &mut System, x: u16, y: u16, pressed: bool) {
    system.joystick.update(x, y, pressed);
}

static mut TICKS: u32 = 0;

#[interrupt(atmega328p)]
fn TIMER0_COMPA() {
    unsafe {
        TICKS += 1;
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut system = System::init();

    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let x_pin = pins.a0.into_analog_input(&mut adc);
    let y_pin = pins.a1.into_analog_input(&mut adc);
    let sw_pin = pins.a2.into_pull_up_input();

    let sda = pins.a4.into_pull_up_input();
    let scl = pins.a5.into_pull_up_input();

    let mut i2c = arduino_hal::i2c::I2c::new(dp.TWI, sda, scl, 100_000);

    display::init(&mut i2c, &mut delay);

    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    let mut buffer: heapless::String<32> = heapless::String::new();

    system.set_state(State::Running);

    dp.TC0.tccr0a.write(|w| w.wgm0().bits(2)); // CTC mode
    dp.TC0.tccr0b.write(|w| w.cs0().prescale_64());
    dp.TC0.ocr0a.write(|w| unsafe { w.bits(249) }); // 16_000_000 / 64 / 250 = 1kHz
    dp.TC0.timsk0.write(|w| w.ocie0a().set_bit());

    unsafe { avr_device::interrupt::enable() };

    let mut last_action = 0u32;

    loop {
        let ticks_now = unsafe { TICKS };

        let x = x_pin.analog_read(&mut adc);
        let y = y_pin.analog_read(&mut adc);
        let pressed = sw_pin.is_low();

        update_joystick(&mut system, x, y, pressed);

        if x < 100 {
            system.set_menu_page(Menu::JoystickTest);
        } else if x > 600 {
            system.set_menu_page(Menu::System);
        }

        if system.menu_page == Menu::Booting {
            // Mostra schermata iniziale
            display::clear(&mut i2c, &mut delay);
            display::set_cursor(&mut i2c, 0, 0, &mut delay);
            display::write_str(&mut i2c, "    ASM v0.1    ", &mut delay);
            display::command(&mut i2c, 0xC0, &mut delay); // Sposta alla seconda riga
            display::write_str(&mut i2c, "  by roothunter ", &mut delay);

            // Dopo aver mostrato la schermata iniziale, passa alla pagina System
            //system.set_menu_page(asm_common::ArduinoMenu::System);
            system.set_menu_page(Menu::Home);
            delay.delay_ms(500u16);
        } else if system.menu_page == Menu::Home {
            // Mostra schermata Home
            display::command(&mut i2c, 0x01, &mut delay); // Clear Display
            display::set_cursor(&mut i2c, 0, 0, &mut delay);
            display::write_str(&mut i2c, "   Home Menu   ", &mut delay);
            display::command(&mut i2c, 0xC0, &mut delay); // Sposta alla seconda riga
            display::write_str(&mut i2c, "1:System 2:Data", &mut delay);

            // Rimani nella schermata Home finché non viene cambiata la pagina
            loop {
                let x = x_pin.analog_read(&mut adc);
                let y = y_pin.analog_read(&mut adc);
                let pressed = sw_pin.is_low();

                update_joystick(&mut system, x, y, pressed);

                if x < 100 {
                    system.set_menu_page(Menu::JoystickTest);
                } else if x > 600 {
                    system.set_menu_page(Menu::System);
                }

                let packet = Packet::read_packet_bytes(&mut serial);

                if let Some(pkt) = packet {
                    match pkt {
                        Packet::Metrics(_) | Packet::Status(_) => {
                            system.set_menu_page(Menu::System);
                            break;
                        }
                    }
                }
            }
        } else if system.menu_page == Menu::JoystickTest {
            display::clear(&mut i2c, &mut delay);

            buffer.clear();

            buffer.push_str("X: ").unwrap();

            let mut num_buf = itoa::Buffer::new();
            let x_str = num_buf.format(x);
            buffer.push_str(x_str).unwrap();

            buffer.push_str(" Y: ").unwrap();
            let y_str = num_buf.format(y);
            buffer.push_str(y_str).unwrap();

            display::set_cursor(&mut i2c, 0, 0, &mut delay);
            display::write_str(&mut i2c, &buffer, &mut delay);

            buffer.clear();
            buffer.push_str("TIME: ").unwrap();
            let time_str = num_buf.format(ticks_now / 1000);
            buffer.push_str(time_str).unwrap();
            buffer.push_str(" s").unwrap();

            display::set_cursor(&mut i2c, 0, 1, &mut delay);
            display::write_str(&mut i2c, &buffer, &mut delay);

            delay.delay_ms(200u16);
        } else {
            let packet = Packet::read_packet_bytes(&mut serial);

            if let Some(pkt) = packet {
                match pkt {
                    Packet::Metrics(m) => {
                        uwriteln!(&mut serial, "Received packet type: Metrics").unwrap();
                        uwriteln!(&mut serial, "x: {}, y: {}, sw: {}", x, y, pressed).unwrap();

                        buffer.clear();

                        buffer.push_str("CPU: ").unwrap();

                        let mut num_buf = itoa::Buffer::new();
                        let cpu_str = num_buf.format(m.cpu);
                        buffer.push_str(cpu_str).unwrap();

                        // Print CPU to display
                        display::clear(&mut i2c, &mut delay);
                        display::set_cursor(&mut i2c, 0, 0, &mut delay);
                        display::write_str(&mut i2c, &buffer, &mut delay);

                        // Print RAM to display
                        let ram_str = num_buf.format(m.ram);
                        buffer.clear();
                        buffer.push_str("RAM: ").unwrap();
                        buffer.push_str(ram_str).unwrap();

                        display::set_cursor(&mut i2c, 0, 1, &mut delay);
                        display::write_str(&mut i2c, &buffer, &mut delay);
                    }
                    Packet::Status(s) => {
                        uwriteln!(&mut serial, "Received packet type: Status").unwrap();

                        buffer.clear();
                        let battery = s.battery;
                        display::set_cursor(&mut i2c, 0, 1, &mut delay);
                        display::write_str(&mut i2c, &buffer, &mut delay);

                        //display::command(&mut i2c, 0xC0, &mut delay); // Move to second
                    }
                }
            }
        }

        // RENDER DISPLAY EVERY 100 TICKS

        if ticks_now - last_action >= 100 {
            last_action = ticks_now;

            // Aggiorna il display in base alla pagina corrente
        }
    }
}
