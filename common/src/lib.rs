#![no_std]

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