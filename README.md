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

<kbd>I</kbd> = Open ROM

<kbd>O</kbd> = Save state

<kbd>P</kbd> = Load state

TODO:

- Fix some PPU bugs
- Run test ROMs
  * Pass ppuio cpu exec space test by implementing dummy reads
- More mappers, esp MMC3
- Run length encoding for savestates
- Rewind
- Add more features to the startup rom like flashing text
- Better error handling for web
- Improve webpage design - try NES.css
- native windows gui kit
- imgui+wgpu for debugger frontend (might need to implement message queues using [std::sync::mpsc](https://doc.rust-lang.org/std/sync/mpsc/))
- Nametable previews
- SNES emulation support
