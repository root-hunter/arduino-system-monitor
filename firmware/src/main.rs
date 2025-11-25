#![no_std]
#![no_main]

mod display;
mod protocol;

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use asm_common::ArduinoSystem;
use asm_common::Packet;
use panic_halt as _;
use ufmt::uwriteln;

use crate::protocol::DeserializePacket;

#[arduino_hal::entry]
fn main() -> ! {
    let mut system = ArduinoSystem::init();

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

    system.set_state(asm_common::ArduinoState::Running);

    loop {
        let x = x_pin.analog_read(&mut adc);
        let y = y_pin.analog_read(&mut adc);
        let pressed = sw_pin.is_low();

        if system.menu_page == asm_common::ArduinoMenu::Booting {
            // Mostra schermata iniziale
            display::command(&mut i2c, 0x01, &mut delay); // Clear Display
            display::set_cursor(&mut i2c, 0, 0, &mut delay);
            display::write_str(&mut i2c, "    ASM v0.1    ", &mut delay);
            display::command(&mut i2c, 0xC0, &mut delay); // Sposta alla seconda riga
            display::write_str(&mut i2c, "  by roothunter ", &mut delay);

            // Dopo aver mostrato la schermata iniziale, passa alla pagina System
            //system.set_menu_page(asm_common::ArduinoMenu::System);
            system.set_menu_page(asm_common::ArduinoMenu::JoystickTest);
            delay.delay_ms(2000u16);

        } else if system.menu_page == asm_common::ArduinoMenu::Home {
            // Mostra schermata Home
            display::command(&mut i2c, 0x01, &mut delay); // Clear Display
            display::set_cursor(&mut i2c, 0, 0, &mut delay);
            display::write_str(&mut i2c, "   Home Menu   ", &mut delay);
            display::command(&mut i2c, 0xC0, &mut delay); // Sposta alla seconda riga
            display::write_str(&mut i2c, "1:System 2:Data", &mut delay);

            // Rimani nella schermata Home finché non viene cambiata la pagina
            loop {
                let packet = Packet::read_packet_bytes(&mut serial);

                if let Some(pkt) = packet {
                    match pkt {
                        Packet::Metrics(_) | Packet::Status(_) => {
                            system.set_menu_page(asm_common::ArduinoMenu::System);
                            break;
                        }
                    }
                }
            }
        } else if system.menu_page == asm_common::ArduinoMenu::JoystickTest {
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

            // delay

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
                        display::command(&mut i2c, 0x01, &mut delay); // Clear Display
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
    }
}
