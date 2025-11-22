build:
	cargo build --release

build-firmware: build
	avr-objcopy -O ihex target/avr-none/release/arduino-system-display.elf firmware.hex

flash-firmware: build-firmware
	avrdude -p atmega328p -c arduino -P /dev/ttyACM0 -b 115200 -U flash:w:firmware.hex