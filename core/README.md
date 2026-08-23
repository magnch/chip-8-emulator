# chip8-core

`chip8-core` is the CHIP-8 interpreter used by the desktop frontend. It contains the emulator state and execution logic, but no windowing, audio, or input-device code.

## Responsibilities

- Fetch, decode, and execute CHIP-8 instructions
- Manage registers, stack, program counter, and index register
- Provide 4 KiB of emulated memory
- Maintain the 64 x 32 display buffer
- Track keypad state
- Update the delay and sound timers

## Basic usage

```rust
use chip8_core::chip8::Chip8;

let mut chip8 = Chip8::new();
chip8.load_rom(&rom)?;

loop {
    chip8.step()?;
    chip8.tick_timers();
}
```

The frontend is responsible for loading the ROM, scheduling CPU steps and timer ticks, reading the display, handling keyboard events, and playing sound while `is_beeping()` is true.

## API

| Method | Purpose |
| --- | --- |
| `Chip8::new` | Create a reset emulator |
| `Chip8::load_rom` | Copy a ROM into memory and set the program counter |
| `Chip8::reset` | Clear CPU, memory, and display state back to a fresh boot |
| `Chip8::step` | Execute one CPU instruction |
| `Chip8::tick_timers` | Decrement the delay and sound timers |
| `Chip8::get_display` | Read the current 64 x 32 display buffer |
| `Chip8::key_down` | Mark a CHIP-8 key as pressed |
| `Chip8::key_up` | Mark a CHIP-8 key as released |
| `Chip8::is_beeping` | Check whether the sound timer is active |
| `Chip8::get_state` | Obtain a copy of the CPU state |

`step` and `tick_timers` are separate operations. A frontend should schedule them according to the desired CPU and timer frequencies rather than assuming that every instruction step represents one timer tick.

## Errors

Operations return `Chip8Error` when they cannot complete. The main errors are:

- `UnknownOpcode` for an unsupported instruction
- `RomTooLarge` when a ROM does not fit in memory
- `MemoryOutOfBounds` for an invalid memory access
- `DisplayOutOfBounds` for an invalid display coordinate
- `KeypadOutOfBounds` for an invalid key
- `StackOverflow` and `StackUnderflow` for invalid subroutine calls and returns

## Tests

Run the core test suite from the workspace root:

```text
cargo test -p chip8-core
```

The tests cover the main instruction families, memory, display, keypad, timers, and error paths.

## Current scope

The core crate is designed to be driven by a frontend. It does not open windows, process operating-system input, or produce audio. Compatibility options are defined in `Config` and exposed through the public `pub config: Config` field on `Chip8` — a frontend can read or replace it at any time, and the new settings take effect starting with the next instruction executed.
