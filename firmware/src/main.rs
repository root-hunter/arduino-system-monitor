#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod joystick;
mod protocol;
mod render;
mod system;

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use asm_common::Packet;
use panic_halt as _;
use ufmt::uwriteln;

use crate::protocol::DeserializePacket;
use crate::system::Menu;
use crate::system::State;
use crate::system::System;

pub fn update_joystick(system: &mut System, x: u16, y: u16, pressed: bool) {
    system.joystick.update(x, y, pressed);
}

#[arduino_hal::entry]
fn main() -> ! {
    let init = System::init();
    let mut system = init;

    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let x_pin = pins.a0.into_analog_input(&mut adc);
    let y_pin = pins.a1.into_analog_input(&mut adc);
    let sw_pin = pins.a2.into_pull_up_input();

    let sda = pins.a4.into_pull_up_input();
    let scl = pins.a5.into_pull_up_input();

    let i2c = arduino_hal::i2c::I2c::new(dp.TWI, sda, scl, 100_000);

    let mut delay = arduino_hal::Delay::new();
    let mut display = render::Display::build(i2c);

    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    let mut buffer: heapless::String<32> = heapless::String::new();

    system.set_state(State::Running);
    System::init_clock();

    let mut x: u16;
    let mut y: u16;
    let mut pressed: bool;

    loop {
        x = x_pin.analog_read(&mut adc);
        y = y_pin.analog_read(&mut adc);
        pressed = sw_pin.is_low();

        update_joystick(&mut system, x, y, pressed);

        if x < 100 {
            system.set_menu_page(Menu::JoystickTest);
        } else if x > 600 {
            system.set_menu_page(Menu::System);
        }

        if system.menu_page == Menu::Booting {
            display.write_first_line("    ASM v0.1    ");
            display.write_second_line("  by roothunter ");

            system.set_menu_page(Menu::JoystickTest);
            delay.delay_ms(500u16);
        } else if system.menu_page == Menu::Home {
            display.write_first_line("   Home Menu   ");
            display.write_second_line("1:System 2:Data");
        } else if system.menu_page == Menu::JoystickTest {
            buffer.clear();

            buffer.push_str("X: ").unwrap();

            let mut num_buf = itoa::Buffer::new();
            let x_str = num_buf.format(x);
            buffer.push_str(x_str).unwrap();

            buffer.push_str(" Y: ").unwrap();
            let y_str = num_buf.format(y);
            buffer.push_str(y_str).unwrap();

            display.write_first_line(&buffer);

            buffer.clear();
            buffer.push_str("TIME: ").unwrap();
            let time_str = num_buf.format(System::get_ticks() / 1000);
            buffer.push_str(time_str).unwrap();
            buffer.push_str(" s").unwrap();

            display.write_second_line(&buffer);

            //delay.delay_ms(100u16);
        } else if system.menu_page == Menu::System {
            let packet = Packet::read_packet_bytes(&mut serial);

            if let Some(pkt) = packet {
                match pkt {
                    Packet::Metrics(m) => {
                        buffer.clear();

                        buffer.push_str("CPU: ").unwrap();

                        let mut num_buf = itoa::Buffer::new();
                        let cpu_str = num_buf.format(m.cpu);
                        buffer.push_str(cpu_str).unwrap();

                        display.write_first_line(&buffer);

                        // Print RAM to display
                        let ram_str = num_buf.format(m.ram);

                        buffer.clear();

                        buffer.push_str("RAM: ").unwrap();
                        buffer.push_str(ram_str).unwrap();

                        display.write_second_line(&buffer);

                        uwriteln!(&mut serial, "Received packet type: Metrics").unwrap();
                        uwriteln!(&mut serial, "x: {}, y: {}, sw: {}", x, y, pressed).unwrap();
                    }
                    Packet::Status(s) => {
                        uwriteln!(&mut serial, "Received packet type: Status").unwrap();

                        //buffer.clear();
                        let battery = s.battery;
                        //display.write_second_line(&buffer);
                    }
                }
            }
        }

        // RENDER DISPLAY EVERY 100 TICKS
        display.update();
    }
}
