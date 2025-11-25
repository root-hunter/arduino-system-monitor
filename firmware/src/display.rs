use arduino_hal::prelude::*;
use panic_halt as _;

// --- Maschere standard per il PCF8574 su moduli LCD I2C ---
// D4-D7 mappano a P4-P7
const RS_BIT: u8 = 0b0000_0001; // P0: Register Select (0=Command, 1=Data)
                                // const RW_BIT: u8 = 0b0000_0010; // P1: Read/Write (0=Write, 1=Read). Lo teniamo a 0 (omettendolo)
const EN_BIT: u8 = 0b0000_0100; // P2: Enable Strobe
const BL_BIT: u8 = 0b0000_1000; // P3: Backlight (0=OFF, 1=ON)

const LCD_ADDRESS: u8 = 0x27; // Indirizzo I2C, comune 0x27 o 0x3F

// Invia il byte dati al PCF8574 e gestisce il pulso di Enable.
// L'argomento 'data' contiene il nibble (D4-D7) + i bit di controllo (RS/BL).
pub fn pulse_en(i2c: &mut arduino_hal::i2c::I2c, data: u8, delay: &mut arduino_hal::Delay) {
    // 1. Manda i dati con EN=1
    let _ = i2c.write(LCD_ADDRESS, &[data | EN_BIT]);
    delay.delay_us(50u16); // Tempo minimo di EN pulse (tEW)

    // 2. Manda i dati con EN=0
    let _ = i2c.write(LCD_ADDRESS, &[data & !EN_BIT]);
    delay.delay_us(100u16); // Tempo minimo di attesa (tHD)
}

// Invia un nibble di 4 bit al display (parte alta o parte bassa del byte)
pub fn send_nibble(
    i2c: &mut arduino_hal::i2c::I2c,
    nibble: u8,
    rs: bool,
    delay: &mut arduino_hal::Delay,
) {
    // 1. Posiziona il nibble (0x0F) sui pin D7..D4 (P7..P4 del PCF8574)
    let mut data = (nibble & 0x0F) << 4;

    // 2. Aggiunge il bit RS (Command o Data)
    if rs {
        data |= RS_BIT;
    }

    // 3. Accende la retroilluminazione (BL)
    data |= BL_BIT;

    // 4. Esegue il pulso di Enable
    pulse_en(i2c, data, delay);
}

// Invia un byte completo, prima il nibble alto, poi quello basso
pub fn send_byte(i2c: &mut arduino_hal::i2c::I2c, byte: u8, rs: bool, delay: &mut arduino_hal::Delay) {
    send_nibble(i2c, byte >> 4, rs, delay); // Nibble Alto
    send_nibble(i2c, byte & 0x0F, rs, delay); // Nibble Basso
}

pub fn command(i2c: &mut arduino_hal::i2c::I2c, cmd: u8, delay: &mut arduino_hal::Delay) {
    send_byte(i2c, cmd, false, delay);
}

pub fn write_char(i2c: &mut arduino_hal::i2c::I2c, c: u8, delay: &mut arduino_hal::Delay) {
    send_byte(i2c, c, true, delay);
}

pub fn write_str(i2c: &mut arduino_hal::i2c::I2c, s: &str, delay: &mut arduino_hal::Delay) {
    for c in s.bytes() {
        write_char(i2c, c, delay);
    }
}

pub fn clear(i2c: &mut arduino_hal::i2c::I2c, delay: &mut arduino_hal::Delay) {
    command(i2c, 0x01, delay);
    delay.delay_ms(2u16);
}

pub fn init(i2c: &mut arduino_hal::i2c::I2c, delay: &mut arduino_hal::Delay) {
    // Step 1: Invia 0x3 (funzione di reset)
    send_nibble(i2c, 0x03, false, delay);
    delay.delay_ms(5u16);

    // Step 2: Invia 0x3
    send_nibble(i2c, 0x03, false, delay);
    delay.delay_us(100u16);

    // Step 3: Invia 0x3
    send_nibble(i2c, 0x03, false, delay);
    delay.delay_us(100u16);

    // Step 4: Passa alla modalità 4-bit (solo nibble alto 0x2)
    send_nibble(i2c, 0x02, false, delay);
    delay.delay_us(100u16);

    // Ora tutte le comunicazioni sono a 4-bit (due nibble per byte)

    command(i2c, 0x28, delay); // Function Set: 4-bit mode, 2 lines, 5x8 dots
    delay.delay_us(50u16);

    command(i2c, 0x0C, delay); // Display ON, Cursor OFF, Blink OFF
    delay.delay_us(50u16);

    command(i2c, 0x06, delay); // Entry Mode Set: Increment cursor, No shift
    delay.delay_us(50u16);

    command(i2c, 0x01, delay); // Clear Display (necessita di tempo)
    delay.delay_ms(2u16);

    command(i2c, 0x80, delay); // Set DDRAM Address to 0 (prima riga, prima colonna)
    delay.delay_us(50u16);
}

// Imposta la posizione del cursore (colonna 0-15, riga 0-1)
pub fn set_cursor(i2c: &mut arduino_hal::i2c::I2c, col: u8, row: u8, delay: &mut arduino_hal::Delay) {
    // Gli indirizzi DDRAM per un 16x2 sono:
    // Riga 0: 0x00 - 0x0F
    // Riga 1: 0x40 - 0x4F

    let mut address = col;

    if row == 1 {
        // Aggiunge l'offset per la seconda riga
        address += 0x40;
    }

    // Comanda Set DDRAM Address: 0x80 OR address
    command(i2c, 0x80 | address, delay);
}