mod macros;
mod mapper;
mod io;
mod cartridge;
mod m6502;
mod bus;
mod ppu;
mod joypad;
mod emulator;
mod opcodes;

use std::process;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

#[macro_use(lazy_static)]
extern crate lazy_static;

use emulator::Emulator;

const DELAY: u64 = 17; // 1000ms / 59.7fps

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("NESTY", (ppu::WIDTH * 2) as u32, (ppu::HEIGHT * 2) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, ppu::WIDTH as u32, ppu::HEIGHT as u32)
        .unwrap();

    let mut key_map = HashMap::new();

    key_map.insert(Keycode::Down, joypad::BUTTON_DOWN);
    key_map.insert(Keycode::Up, joypad::BUTTON_UP);
    key_map.insert(Keycode::Right, joypad::BUTTON_RIGHT);
    key_map.insert(Keycode::Left, joypad::BUTTON_LEFT);
    key_map.insert(Keycode::Space, joypad::BUTTON_SELECT);
    key_map.insert(Keycode::Return, joypad::BUTTON_START);
    key_map.insert(Keycode::A, joypad::BUTTON_A);
    key_map.insert(Keycode::S, joypad::BUTTON_B);

    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/nes-test-roms-master/cpu_exec_space/test_cpu_exec_space_ppuio.nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Donkey Kong (World) (Rev A).nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super Mario Bros. (World).nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super_Mario_Forever_Clean_Patch.nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/nestest.nes
    let mut nes = Emulator::new("C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super Mario Bros. (World).nes");

    nes.reset();

    let mut event_pump = sdl_context.event_pump().unwrap();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        nes.joypad().press(*key);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        nes.joypad().release(*key);
                    }
                }
                _ => {}
            }
        }

        nes.update();

        texture.update(None, &nes.ppu().pixels, ppu::WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        thread::sleep(Duration::from_millis(DELAY));
    }
}
