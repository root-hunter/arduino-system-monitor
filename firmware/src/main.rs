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

    let sda = pins.a4.into_pull_up_input();
    let scl = pins.a5.into_pull_up_input();

    let mut i2c = arduino_hal::i2c::I2c::new(dp.TWI, sda, scl, 100_000);

    display::init(&mut i2c, &mut delay);

    display::write_str(&mut i2c, "    ASM v0.1    ", &mut delay);
    display::command(&mut i2c, 0xC0, &mut delay); // Sposta alla seconda riga (0x40 + 0x80 = 0xC0)
    display::write_str(&mut i2c, "  by roothunter ", &mut delay);

    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    let mut buffer: heapless::String<32> = heapless::String::new();

    system.set_state(asm_common::ArduinoState::Running);

    loop {
        if system.menu_page == asm_common::ArduinoMenu::Booting {
            // Mostra schermata iniziale
            display::command(&mut i2c, 0x01, &mut delay); // Clear Display
            display::set_cursor(&mut i2c, 0, 0, &mut delay);
            display::write_str(&mut i2c, "    ASM v0.1    ", &mut delay);
            display::command(&mut i2c, 0xC0, &mut delay); // Sposta alla seconda riga
            display::write_str(&mut i2c, "  by roothunter ", &mut delay);

            // Dopo aver mostrato la schermata iniziale, passa alla pagina System
            system.set_menu_page(asm_common::ArduinoMenu::System);
            delay.delay_ms(2000u16);

            system.set_menu_page(asm_common::ArduinoMenu::Home);
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
        } else {
            let packet = Packet::read_packet_bytes(&mut serial);

            if let Some(pkt) = packet {
                match pkt {
                    Packet::Metrics(m) => {
                        uwriteln!(&mut serial, "Received packet type: Metrics").unwrap();

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