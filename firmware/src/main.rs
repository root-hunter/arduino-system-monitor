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
use heapless::Vec;
use panic_halt as _;
use sh1106::interface::DisplayInterface;
use ufmt::uwriteln;

use crate::protocol::DeserializePacket;
use crate::render::Display;
use crate::render::Func;
use crate::system::Menu;
use crate::system::State;
use crate::system::System;
use sh1106::{prelude::*, Builder};

pub fn update_joystick(system: &mut System, x: u16, y: u16, pressed: bool) {
    system.joystick.update(x, y, pressed);
}

const DISP_WIDTH: u32 = 128;
const DISP_HEIGHT: u32 = 64;

fn draw_rectangle(
    display: &mut GraphicsMode<impl DisplayInterface>,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) {
    for i in 0..w {
        for j in 0..h {
            display.set_pixel(x + i, y + j, 1);
        }
    }
}

fn draw_square(display: &mut GraphicsMode<impl DisplayInterface>, x: u32, y: u32, size: u32) {
    draw_rectangle(display, x, y, size, size);
}

fn draw_circle(display: &mut GraphicsMode<impl DisplayInterface>, x0: u32, y0: u32, radius: u32) {
    let mut x = radius as i32;
    let mut y = 0i32;
    let mut err = 0i32;

    while x >= y {
        display.set_pixel((x0 as i32 + x) as u32, (y0 as i32 + y) as u32, 1);
        display.set_pixel((x0 as i32 + y) as u32, (y0 as i32 + x) as u32, 1);
        display.set_pixel((x0 as i32 - y) as u32, (y0 as i32 + x) as u32, 1);
        display.set_pixel((x0 as i32 - x) as u32, (y0 as i32 + y) as u32, 1);
        display.set_pixel((x0 as i32 - x) as u32, (y0 as i32 - y) as u32, 1);
        display.set_pixel((x0 as i32 - y) as u32, (y0 as i32 - x) as u32, 1);
        display.set_pixel((x0 as i32 + y) as u32, (y0 as i32 - x) as u32, 1);
        display.set_pixel((x0 as i32 + x) as u32, (y0 as i32 - y) as u32, 1);

        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err += 2 * (y - x) + 1;
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut system = System::init();

    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let x_pin = pins.a0.into_analog_input(&mut adc);
    let y_pin = pins.a1.into_analog_input(&mut adc);
    let sw_pin = pins.a2.into_pull_up_input();

    let sda = pins.a4.into_pull_up_input();
    let scl = pins.a5.into_pull_up_input();

    let i2c = arduino_hal::i2c::I2c::new(dp.TWI, sda, scl, 100_000);

    let mut display = Display::build(i2c);
    display.init();

    let mut x: u32 = 0;
    let mut y: u32 = 0;
    let mut pressed = false;

    let mut square_size: u32 = 16;
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);

    System::init_clock();

    loop {
        // normalize joystick readings
        let read_x = x_pin.analog_read(&mut adc) as u32;
        let read_y = y_pin.analog_read(&mut adc) as u32;

        let x_diff: u32 = (DISP_WIDTH - square_size);

        x = ((read_x * x_diff) / 690).min(x_diff).into();
        y = ((read_y * (DISP_HEIGHT - square_size)) / 690).into();
        pressed = sw_pin.is_low();

        display.draw_square(x, y, square_size);
        display.draw_circle(x + square_size / 2, y + square_size / 2, square_size);

        //uwriteln!(&mut serial, "READ X: {}, READ Y: {}", read_x, read_y).unwrap();
        //uwriteln!(&mut serial, "X: {}, Y: {}, size: {}", x, y, square_size).unwrap();

        //display.draw_square(x, y, square_size);
        //display.draw_circle(x + square_size / 2, y + square_size / 2, square_size);

        if pressed {
            square_size += 1;
        } else if square_size > 1 {
            square_size -= 1;
        }

        display.update();
    }
}
