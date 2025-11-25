use arduino_hal::prelude::*;
use panic_halt as _;
use asm_common::{PACKET_METRICS, PACKET_STATUS, Packet};

pub trait DeserializePacket {
    fn read_packet_bytes(
        serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>,
    ) -> Option<asm_common::Packet>;
}


pub fn deserialize_u16(serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>) -> u16 {
    let high = nb::block!(serial.read()).unwrap();
    let low = nb::block!(serial.read()).unwrap();

    ((high as u16) << 8) | (low as u16)
}

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
            PACKET_METRICS => {
                let cpu = nb::block!(serial.read()).unwrap();
                let ram = deserialize_u16(serial);

                Some(Packet::Metrics(asm_common::Metrics { cpu, ram }))
            }
            PACKET_STATUS => {
                let battery = nb::block!(serial.read()).unwrap();
                let led_on = nb::block!(serial.read()).unwrap() != 0;

                Some(Packet::Status(asm_common::Status { battery, led_on }))
            }
            _ => None,
        }
    }
}