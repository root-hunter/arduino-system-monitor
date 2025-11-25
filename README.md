# ATmega328P Machine Instructions Overview

The **ATmega328P** uses the **AVR 8‑bit RISC architecture**, featuring a
rich instruction set of **130 instructions**, most of which execute in a
single clock cycle.

https://ww1.microchip.com/downloads/en/devicedoc/AVR-Instruction-Set-Manual-DS40002198A.pdf

## Instruction Categories

### 1. Arithmetic Instructions

-   **ADD**, **ADC**, **SUB**, **SBC**, **MUL**, **INC**, **DEC**
-   Used for integer math on registers R0--R31.

### 2. Logical & Bitwise Instructions

-   **AND**, **OR**, **EOR**, **COM**, **NEG**
-   **SBR**, **CBR** (set/clear bits in registers)

### 3. Shift & Rotate Instructions

-   **LSL**, **LSR**, **ASR**, **ROL**, **ROR**

### 4. Data Transfer

-   **MOV**, **LD**, **ST**, **LDS**, **STS**, **LDI**
-   Registers, SRAM, and I/O memory operations.

### 5. Branch Instructions

-   **RJMP**, **JMP**, **BREQ**, **BRNE**, **BRGE**, **BRLT**, **CALL**,
    **RET**
-   Conditional and unconditional program-flow control.

### 6. Bit Manipulation

-   **SBI**, **CBI** (set/clear I/O bits)
-   **SBIC**, **SBIS** (skip next instruction based on bit state)

### 7. MCU Control Instructions

-   **NOP**, **SLEEP**, **WDR**, **CLI**, **SEI**
-   System-level behavior and interrupt control.

## Register File

-   32 general‑purpose registers (**R0--R31**)
-   First 16 are more limited; upper registers support immediate
    operations.

## Instruction Format

Most instructions use: - **16‑bit opcodes** - Single clock cycle
execution

## Useful Reference

Full instruction set: *ATmega328P datasheet, Instruction Set Summary
section*.