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
use sh1106::{prelude::*, Builder};

pub fn update_joystick(system: &mut System, x: u16, y: u16, pressed: bool) {
    system.joystick.update(x, y, pressed);
}

const DISP_WIDTH: u32 = 128;
const DISP_HEIGHT: u32 = 64;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let x_pin = pins.a0.into_analog_input(&mut adc);
    let y_pin = pins.a1.into_analog_input(&mut adc);
    let sw_pin = pins.a2.into_pull_up_input();

    let sda = pins.a4.into_pull_up_input();
    let scl = pins.a5.into_pull_up_input();

    let i2c = arduino_hal::i2c::I2c::new(dp.TWI, sda, scl, 100_000);

    let mut display: GraphicsMode<_> = Builder::new()
        .with_size(DisplaySize::Display128x64)
        .with_rotation(sh1106::displayrotation::DisplayRotation::Rotate0)
        .connect_i2c(i2c)
        .into();

    display.init().unwrap();
    display.flush().unwrap();

    let mut x: u32 = 0;
    let mut y: u32 = 0;
    let mut pressed = false;

    let mut square_size: u32 = 16;
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);

    loop {
        // normalize joystick readings
        let read_x = x_pin.analog_read(&mut adc) as u32;
        let read_y = y_pin.analog_read(&mut adc) as u32;

        let x_diff: u32 = (DISP_WIDTH - square_size); 

        x = ((read_x * x_diff) / 690).min(x_diff).into();
        y = ((read_y * (DISP_HEIGHT - square_size)) / 690).into();
        pressed = sw_pin.is_low();

        uwriteln!(&mut serial, "READ X: {}, READ Y: {}", read_x, read_y).unwrap();
        uwriteln!(&mut serial, "X: {}, Y: {}, size: {}", x, y, square_size).unwrap();

        display.clear();

        for i in 0..square_size {
            for j in 0..square_size {
                display.set_pixel(x + i, y + j, 1);
            }
        }

        if pressed {
            square_size += 1;
        } else if square_size > 1 {
            square_size -= 1;
        }

        display.flush().unwrap();
    }
}
