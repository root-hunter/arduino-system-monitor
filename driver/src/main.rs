// NOTA BENE: Questo è un esempio concettuale. 
// Per l'uso reale, dovresti aggiungere le dipendenze 'sysinfo' e 'serialport' al tuo Cargo.toml.

use std::{io, thread, time::Duration};
// use serialport::{SerialPort, self}; // Necessario per la comunicazione seriale reale
// use sysinfo::{System, SystemExt, CpuExt}; // Necessario per le metriche reali

// --- Impostazioni di Configurazione ---
const PORT_NAME: &str = "/dev/ttyACM0"; // Cambia con la tua porta (es. "COM3" su Windows)
const BAUD_RATE: u32 = 9600;

// Funzione che simula l'ottenimento delle metriche di sistema
fn get_simulated_metrics(iteration: u32) -> (f32, f32) {
    // In un'applicazione reale, useremmo 'sysinfo' qui.
    // Esempio: 
    // let mut sys = System::new_all();
    // sys.refresh_all();
    // let cpu_usage = sys.global_cpu_info().cpu_usage();
    // let used_ram = sys.used_memory() as f32 / 1024.0 / 1024.0 / 1024.0; // GB
    
    // Simulazione di dati che cambiano nel tempo
    let cpu_sim = 20.0 + (iteration as f32 % 50.0);
    let ram_sim = 4.5 + (iteration as f32 % 10.0) / 10.0;
    
    (cpu_sim, ram_sim)
}

fn main() -> io::Result<()> {
    println!("--- Client Seriale Rust per Arduino ---");
    println!("Tentativo di connessione a {} @ {} Baud...", PORT_NAME, BAUD_RATE);

    let mut port = serialport::new(PORT_NAME, BAUD_RATE)
        .open()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Errore porta seriale: {}", e)))?;
    
    // Simulazione di un ritardo di apertura della porta
    thread::sleep(Duration::from_secs(2));
    println!("Connessione stabilita (Simulata). Inizio invio dati...");
    
    let mut iteration = 0;

    loop {
        // 1. Ottieni le metriche
        let (cpu_percent, ram_used_gb) = get_simulated_metrics(iteration);
        
        // 2. Formatta il messaggio (formato atteso da Arduino: "CPU_VAL,RAM_VAL\n")
        let message = format!("{:.1}%, {:.2} GB\n", cpu_percent, ram_used_gb);
        
        // 3. Invia il messaggio (Simulato)
        
        // In un'applicazione reale:
        port.write_all(message.as_bytes())?;

        // Simulazione di invio:
        print!("Invio simulato: {}", message.trim());
        
        // 4. Se la funzione write_all ha successo, incrementa e attendi
        iteration += 1;
        thread::sleep(Duration::from_secs(1));
        println!(); // Aggiungi newline dopo l'invio
    }
}