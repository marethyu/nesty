mod macros;
mod mapper;
mod io;
mod cartridge;
mod m6502;
mod bus;
mod ppu;
mod joypad;
mod opcodes;

use std::cell::RefCell;
use std::sync::Arc;
use std::process;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

#[macro_use(lazy_static)]
extern crate lazy_static;

use cartridge::Cartridge;
use bus::Bus;

const CYCLES_PER_FRAME: u64 = 29781; // how many CPU cycles required to render one frame
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

    /* TODO create a emulator struct to enscapulate everything */

    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/nes-test-roms-master/cpu_exec_space/test_cpu_exec_space_ppuio.nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Donkey Kong (World) (Rev A).nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super Mario Bros. (World).nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super_Mario_Forever_Clean_Patch.nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/nestest.nes
    let cart = Cartridge::new("C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super Mario Bros. (World).nes");
    let cart_arc = Arc::new(RefCell::new(cart)); // IMPORTANT refernces must be created inside the main thread otherwise it will be destroyed immediately
    let weak_cart = &Arc::downgrade(&cart_arc);

    let bus = Arc::new_cyclic(|weak_bus| {
        RefCell::new(Bus::new(
            weak_bus,
            weak_cart
        ))
    });

    let cpu = Arc::clone(&bus.borrow().cpu);
    let ppu = Arc::clone(&bus.borrow().ppu);

    cpu.borrow_mut().reset();
    ppu.borrow_mut().reset();

    let mut prev_total_cycles = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        bus.borrow_mut().joypad.press(*key);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        bus.borrow_mut().joypad.release(*key);
                    }
                }
                _ => {}
            }
        }

        let mut total: u64 = 0;

        while total < CYCLES_PER_FRAME {
            if ppu.borrow().nmi {
                cpu.borrow_mut().nmi();
                ppu.borrow_mut().nmi = false;
            }

            // TODO IRQ here

            cpu.borrow_mut().tick();

            let total_cycles = cpu.borrow().total_cycles;
            let cycles = total_cycles - prev_total_cycles;
            prev_total_cycles = total_cycles;

            for _i in 0..cycles {
                ppu.borrow_mut().tick();
                ppu.borrow_mut().tick();
                ppu.borrow_mut().tick();
            }

            total += cycles;
        }

        texture.update(None, &ppu.borrow().pixels, ppu::WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        thread::sleep(Duration::from_millis(DELAY));
    }
}
