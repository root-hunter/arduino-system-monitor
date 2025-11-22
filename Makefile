firmware-toolchain-install:
	rustup toolchain install nightly-2025-04-27

firmware-build:
	cd firmware; cargo build --release

firmware-export: firmware-build
	cd firmware; mkdir -p dist
	cd firmware; avr-objcopy -O ihex target/avr-none/release/arduino-system-display.elf dist/firmware.hex

firmware-flash: firmware-export
	cd firmware; avrdude -p atmega328p -c arduino -P /dev/ttyACM0 -b 115200 -U flash:w:dist/firmware.hex

driver-dev:
	cd driver; cargo run