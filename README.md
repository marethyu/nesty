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
  * Scrolling in Legend of Zelda has some issues (horizontal scrolling is fine but vertical scrolling is not)
- Run test ROMs
- More mappers
- Savestates and maybe rewind
- Improve webpage design
- imgui+wgpu for debugger frontend (might need to implement message queues using [std::sync::mpsc](https://doc.rust-lang.org/std/sync/mpsc/))
- Nametable previews
- SNES emulation support
