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