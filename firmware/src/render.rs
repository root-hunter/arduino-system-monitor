use arduino_hal::prelude::*;
use panic_halt as _;

// --- Maschere standard per il PCF8574 su moduli LCD I2C ---
// D4-D7 mappano a P4-P7
const RS_BIT: u8 = 0b0000_0001; // P0: Register Select (0=Command, 1=Data)
                                // const RW_BIT: u8 = 0b0000_0010; // P1: Read/Write (0=Write, 1=Read). Lo teniamo a 0 (omettendolo)
const EN_BIT: u8 = 0b0000_0100; // P2: Enable Strobe
const BL_BIT: u8 = 0b0000_1000; // P3: Backlight (0=OFF, 1=ON)

const LCD_ADDRESS: u8 = 0x27; // Indirizzo I2C, comune 0x27 o 0x3F

pub struct Display {
    i2c: arduino_hal::i2c::I2c,
    delay: arduino_hal::Delay,
    last_frame: u32,
    buffer_line_1: heapless::String<16>,
    buffer_line_2: heapless::String<16>,
}

impl Display {
    pub fn build(i2c: arduino_hal::i2c::I2c) -> Self {
        let mut display = Display {
            i2c,
            delay: arduino_hal::Delay::new(),
            last_frame: 0,
            buffer_line_1: heapless::String::new(),
            buffer_line_2: heapless::String::new(),
        };
        display.init();
        display
    }

    pub fn init(&mut self) {
        // Step 1: Invia 0x3 (funzione di reset)
        self.send_nibble(0x03, false);
        self.delay.delay_ms(5u16);

        // Step 2: Invia 0x3
        self.send_nibble(0x03, false);
        self.delay.delay_us(100u16);

        // Step 3: Invia 0x3
        self.send_nibble(0x03, false);
        self.delay.delay_us(100u16);
        // Step 4: Passa alla modalità 4-bit (solo nibble alto 0x2)
        self.send_nibble(0x02, false);
        self.delay.delay_us(100u16);

        // Ora tutte le comunicazioni sono a 4-bit (due nibble per byte)

        self.command(0x28); // Function Set: 4-bit mode, 2 lines, 5x8 dots
        self.delay.delay_us(50u16);

        self.command(0x0C); // Display ON, Cursor OFF, Blink OFF
        self.delay.delay_us(50u16);

        self.command(0x06); // Entry Mode Set: Increment cursor, No shift
        self.delay.delay_us(50u16);

        self.command(0x01); // Clear Display (necessita di tempo)
        self.delay.delay_ms(2u16);

        self.command(0x80); // Set DDRAM Address to 0 (prima riga, prima colonna)
        self.delay.delay_us(50u16);
    }

    // Invia il byte dati al PCF8574 e gestisce il pulso di Enable.
    // L'argomento 'data' contiene il nibble (D4-D7) + i bit di controllo (RS/BL).
    pub fn pulse_en(&mut self, data: u8) {
        // 1. Manda i dati con EN=1
        let _ = self.i2c.write(LCD_ADDRESS, &[data | EN_BIT]);
        self.delay.delay_us(50u16); // Tempo minimo di EN pulse (tEW)

        // 2. Manda i dati con EN=0
        let _ = self.i2c.write(LCD_ADDRESS, &[data & !EN_BIT]);
        self.delay.delay_us(100u16); // Tempo minimo di attesa (tHD)
    }

    // Invia un nibble di 4 bit al display (parte alta o parte bassa del byte)
    pub fn send_nibble(&mut self, nibble: u8, rs: bool) {
        // 1. Posiziona il nibble (0x0F) sui pin D7..D4 (P7..P4 del PCF8574)
        let mut data = (nibble & 0x0F) << 4;

        // 2. Aggiunge il bit RS (Command o Data)
        if rs {
            data |= RS_BIT;
        }

        // 3. Accende la retroilluminazione (BL)
        data |= BL_BIT;

        // 4. Esegue il pulso di Enable
        self.pulse_en(data);
    }

    // Invia un byte completo, prima il nibble alto, poi quello basso
    pub fn send_byte(&mut self, byte: u8, rs: bool) {
        self.send_nibble(byte >> 4, rs); // Nibble Alto
        self.send_nibble(byte & 0x0F, rs); // Nibble Basso
    }

    pub fn command(&mut self, cmd: u8) {
        self.send_byte(cmd, false);
    }

    pub fn write_char(&mut self, c: u8) {
        self.send_byte(c, true);
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.bytes() {
            self.write_char(c);
        }
    }

    pub fn clear(&mut self) {
        self.command(0x01);
        self.delay.delay_ms(2u16);
    }

    pub fn set_first_line(&mut self) {
        self.set_cursor(0, 0);
    }

    pub fn set_second_line(&mut self) {
        self.set_cursor(0, 1);
    }

    // Imposta la posizione del cursore (colonna 0-15, riga 0-1)
    pub fn set_cursor(&mut self, col: u8, row: u8) {
        // Gli indirizzi DDRAM per un 16x2 sono:
        // Riga 0: 0x00 - 0x0F
        // Riga 1: 0x40 - 0x4F

        let mut address = col;

        if row == 1 {
            // Aggiunge l'offset per la seconda riga
            address += 0x40;
        }

        // Comanda Set DDRAM Address: 0x80 OR address
        self.command(0x80 | address);
    }

    pub fn should_update(&mut self, interval_ms: u32) -> bool {
        let current_ticks = crate::system::System::get_ticks();
        if current_ticks.wrapping_sub(self.last_frame) >= interval_ms {
            self.last_frame = current_ticks;
            true
        } else {
            false
        }
    }

    pub fn update(&mut self) {
        self.clear();

        if self.should_update(100) {
            // First line
            self.set_first_line();

            let first_line: heapless::String<16> = self.buffer_line_1.clone();
            self.write_str(&first_line);

            // Second line
            self.set_second_line();

            let second_line: heapless::String<16> = self.buffer_line_2.clone();
            self.write_str(&second_line);
        } else {
            self.set_first_line();
            self.write_str("Error: Timeout");
        }
    }

    pub fn write_buffer_first_line(&mut self, text: &str) {
        self.buffer_line_1.clear();
        self.buffer_line_1.push_str(text).unwrap();
    }

    pub fn write_buffer_second_line(&mut self, text: &str) {
        self.buffer_line_2.clear();
        self.buffer_line_2.push_str(text).unwrap();
    }
}
