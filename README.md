# Nesty

WIP NES emulator

![mario](media/mario.gif)

![space harrier](media/space-harrier.gif)

Run:
```
cargo run --release "[ROM FILE].nes"
```

Keybindings:

<kbd>←</kbd> = Left

<kbd>→</kbd> = Right

<kbd>↑</kbd> = Up

<kbd>↓</kbd> = Down

<kbd>A</kbd> = A

<kbd>S</kbd> = B

<kbd>Space</kbd> = Select

<kbd>Return</kbd> = Start

TODO:

- Fix some PPU bugs
- Run test ROMs
- More mappers, esp MMC3
- Savestates and maybe rewind
- Improve webpage design
- imgui+wgpu for debugger frontend (might need to implement message queues using [std::sync::mpsc](https://doc.rust-lang.org/std/sync/mpsc/))
- Nametable previews
- SNES emulation support
