# Tetris - Extreme Lightweight

A minimal Tetris implementation for Windows x86_64, written in pure Rust with **zero external dependencies**.

## Performance

```
EXE:     118,784 bytes (116 KB)
Memory:  ~8 MB Working Set / ~2 MB Private Bytes
CPU idle: ~0%
```

## Features

- 10x20 playfield
- 7 tetrominoes (I, O, T, S, Z, J, L)
- Left / Right movement
- Soft drop (+1 pt/cell)
- Hard drop (+2 pt/cell)
- Rotation with wall kicks
- Line clearing (1 line = 100pt, 2 = 300pt, 3 = 500pt, 4 = 800pt)
- Score / Lines / Level display
- Next piece preview
- Level progression (speed increases every 10 lines)
- Game over detection + restart

## How to Build

```
cargo build --release
```

Output: `target/release/tetris.exe` (~116 KB)

## Controls

| Key | Action |
|-----|--------|
| Arrow Left | Move left |
| Arrow Right | Move right |
| Arrow Down | Soft drop |
| Arrow Up | Rotate |
| Space | Hard drop |
| R | Restart (game over) |

## Architecture

- **Zero dependencies** - no winit, no softbuffer, no game engine
- Raw Win32 API via FFI (RegisterClassExW, CreateWindowExW, StretchDIBits, PeekMessageW)
- Single source file (`src/main.rs`, ~630 lines)
- Fixed-size arrays only (`[u8; 200]` board), no heap allocation in game loop
- XOR-shift PRNG (no std dependency)
- Custom 5x7 bitmap font (hardcoded, no font files)
- Event-driven loop with `PeekMessageW` + `Sleep()` for minimal CPU usage

## Binary Optimization

- `opt-level = "z"` (size optimization)
- `lto = true` (link-time optimization)
- `codegen-units = 1`
- `panic = "abort"`
- `strip = true`
