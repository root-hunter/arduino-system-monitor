#![no_std]
#![no_main]

mod display;

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use panic_halt as _;
use ufmt::uwrite;
use ufmt::uwriteln;

trait DeserializePacket {
    fn read_packet_bytes(
        serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>,
    ) -> Option<asm_common::Packet>;
}

use asm_common::Packet;

impl DeserializePacket for Packet {
    fn read_packet_bytes(
        serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>,
    ) -> Option<asm_common::Packet> {
        let packet_type = nb::block!(serial.read());

        if packet_type.is_err() {
            return None;
        }

        let packet_type = packet_type.unwrap();

        //uwriteln!(&mut serial, "Ricevuto pacchetto tipo: {}", packet_type).unwrap();

        match packet_type {
            0x01 => {
                let cpu = nb::block!(serial.read()).unwrap();
                let ram_high = nb::block!(serial.read()).unwrap();
                let ram_low = nb::block!(serial.read()).unwrap();
                let ram: u16 = ((ram_high as u16) << 8) | ram_low as u16;

                Some(Packet::Metrics(asm_common::Metrics { cpu, ram }))
            }
            0x02 => {
                // Status
                let battery = nb::block!(serial.read()).unwrap();
                let led_on = nb::block!(serial.read()).unwrap() != 0;

                Some(Packet::Status(asm_common::Status { battery, led_on }))
            }
            _ => None,
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    let sda = pins.a4.into_pull_up_input();
    let scl = pins.a5.into_pull_up_input();

    let mut i2c = arduino_hal::i2c::I2c::new(dp.TWI, sda, scl, 100_000);

    display::init(&mut i2c, &mut delay);

    display::write_str(&mut i2c, "_____SYSTEM_____", &mut delay);
    display::command(&mut i2c, 0xC0, &mut delay); // Sposta alla seconda riga (0x40 + 0x80 = 0xC0)
    display::write_str(&mut i2c, "< S D M", &mut delay);

    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    let mut buffer: heapless::String<32> = heapless::String::new();

    loop {
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

                    //display::command(&mut i2c, 0xC0, &mut delay); // Move to second line
                }
            }
        }
    }
}
