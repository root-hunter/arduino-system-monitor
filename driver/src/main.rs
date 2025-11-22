use std::{io, string, thread, time::Duration};

use asm_common::{Metrics, Packet, Status};

const PORT_NAME: &str = "/dev/ttyACM0"; // Cambia con la tua porta (es. "COM3" su Windows)
const BAUD_RATE: u32 = 9600;

enum MenuPages {
    System,
    Data,
    Monitor,
}
// Serializza i pacchetti
fn serialize_packet(packet: Packet) -> Vec<u8> {
    match packet {
        Packet::Metrics(m) => {
            let mut buf = vec![0x01]; // tipo = 1 = Metrics
            buf.push(m.cpu);
            buf.push((m.ram >> 8) as u8);
            buf.push((m.ram & 0xFF) as u8);
            buf
        }
        Packet::Status(s) => {
            let mut buf = vec![0x02]; // tipo = 2 = Status
            buf.push(s.battery);
            buf.push(s.led_on as u8);
            buf
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    println!("Apertura porta seriale...");

    let mut port = serialport::new(PORT_NAME, BAUD_RATE)
        .timeout(Duration::from_millis(50))
        .open()?;

    thread::sleep(Duration::from_secs(2)); // Arduino reset workaround

    // --- THREAD DI LETTURA ---
    let mut port_reader = port.try_clone().expect("Clone serial port failed");
    thread::spawn(move || {
        let mut message = String::new();
        let mut buf = [0u8; 256];
        loop {
            match port_reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let buf = &buf[..n];
                    message.push_str(&String::from_utf8_lossy(buf));

                    if message.ends_with('\n') {
                        print!("[RX] {}", message);
                        message.clear();
                    }
                }
                _ => {
                    // Nessun byte, continua
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    // --- INVIO DATI ---
    println!("Invio pacchetti...");

    println!("In ascolto... premi Ctrl+C per uscire.");
    loop {
        let stat = sysinfo::System::new_all();
        let cpu_usage = stat.global_cpu_usage();
        let ram_usage = stat.used_memory();

        let p1 = Packet::Metrics(Metrics {
            cpu: cpu_usage as u8,
            ram: (ram_usage / 1024 / 1024 / 1024) as u16,
        });
        let p2 = Packet::Status(Status {
            battery: 80,
            led_on: true,
        });

        let v1 = serialize_packet(p1);
        let v2 = serialize_packet(p2);

        println!("[TX] {:?}", v1);
        port.write_all(&v1)?;

        thread::sleep(Duration::from_millis(500));

        println!("[TX] {:?}", v2);
        port.write_all(&v2)?;

        thread::sleep(Duration::from_secs(1));
    }
}
