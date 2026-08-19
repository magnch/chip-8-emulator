# chip-8-emulator

A CHIP-8 emulator written in Rust. `chip8-core` implements the interpreter (memory, opcodes, display buffer, timers), `chip8-desktop` runs it with SDL2 for video, sound, and keyboard input.

## Layout

- `core/` — `chip8-core`, the CHIP-8 interpreter. No I/O, just state and opcode execution.
- `desktop/` — `chip8-desktop`, an SDL2 frontend around `chip8-core`.
- `roms/` — test ROMs and a couple of games.

## Building

Requires SDL2 development libraries (the `bundled` feature builds SDL2 from source, so no system install is strictly needed on most platforms).

```
cargo build --release
```

## Running

The desktop binary currently loads a hardcoded ROM path in [main.rs](desktop/src/main.rs) — point it at a ROM under `roms/` and run:

```
cargo run --release -p chip8-desktop
```

## Controls

The original COSMAC VIP keypad is mapped onto the left side of a QWERTY keyboard:

```
1 2 3 4        1 2 3 C
Q W E R   ->   4 5 6 D
A S D F        7 8 9 E
Z X C V        A 0 B F
```

Escape quits.

## Quirks

CHIP-8 has a handful of behaviors that differ between interpreters (shift instructions, `BNNN`, `FX55`/`FX65` index handling, etc.). These are exposed as toggles in [Config](core/src/config.rs) rather than hardcoded, so ROMs written against different conventions can be supported.
