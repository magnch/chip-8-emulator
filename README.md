# chip-8-emulator

A CHIP-8 emulator written in Rust. `chip8-core` implements the interpreter (memory, opcodes, display buffer, timers), `chip8-desktop` runs it in a native [egui](https://github.com/emilk/egui)/`eframe` window, with audio via `rodio`.

## Layout

- `core/` — `chip8-core`, the CHIP-8 interpreter. No I/O, just state and opcode execution.
- `desktop/` — `chip8-desktop`, an egui/eframe frontend around `chip8-core`.
- `roms/` — test ROMs and a couple of games.

## Building

```
cargo build --release
```

eframe uses the OS's native windowing and GPU backend, so no SDL2 or other system video library is required. On Linux you'll need the usual GTK/X11 or Wayland development packages that `winit`/`eframe` depend on.

## Running

```
cargo run --release -p chip8-desktop
```

No ROM is loaded on startup — use **File > Open ROM…** in the app to pick one (e.g. from `roms/`).

## Menu

- **File** — Open ROM… (native file picker), Exit.
- **Settings** — Paused (toggle CPU/timer execution), Reset (reload the current ROM from a clean state), and a **Configuration** submenu for the compatibility toggles described below. Changes apply immediately.

## Controls

The original COSMAC VIP keypad is mapped onto the left side of a QWERTY keyboard:

```
1 2 3 4        1 2 3 C
Q W E R   ->   4 5 6 D
A S D F        7 8 9 E
Z X C V        A 0 B F
```

## Quirks

CHIP-8 has a handful of behaviors that differ between interpreters (shift instructions, `BNNN`, `FX55`/`FX65` index handling, etc.). These are exposed as toggles in [Config](core/src/config.rs), settable live from Settings > Configuration, so ROMs written against different conventions can be supported without a rebuild.
