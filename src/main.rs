mod macros;
mod traits;
mod cartridge;
mod m6502;
mod bus;
mod ppu;
mod joypad;
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

use cartridge::Cartridge;
use bus::Bus;
use ppu::PPU;
use m6502::M6502;

use crate::traits::IO;

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

    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/nes-test-roms-master/cpu_exec_space/test_cpu_exec_space_ppuio.nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Donkey Kong (World) (Rev A).nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super Mario Bros. (World).nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super_Mario_Forever_Clean_Patch.nes
    // C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/nestest.nes
    let cart = Cartridge::new("C:/Users/Jimmy/OneDrive/Documents/git/nesty/roms/Super Mario Bros. (World).nes");

    // Ugly but it prevents stack overflow...
    let mut cpu = M6502::new(Bus::new(cart.prg_rom, PPU::new(cart.chr_rom, cart.mirroring_type)));

    //let mut ppu = PPU::new(cart.chr_rom, cart.mirroring_type);
    //let mut bus = Bus::new(cart.prg_rom, ppu);
    //let mut cpu = M6502::new(bus);

    cpu.reset();
    cpu.bus.reset();
    cpu.bus.ppu.reset();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut cycles: u64;
    let mut prev: u64 = 0;

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        cpu.bus.joypad.press(*key);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        cpu.bus.joypad.release(*key);
                    }
                }
                _ => {}
            }
        }

        let mut total: u64 = 0;

        while total < CYCLES_PER_FRAME {
            if cpu.bus.ppu.nmi {
                cpu.nmi();
                cpu.bus.ppu.nmi = false;
            }

            // Check for more interrupts here...

            cpu.tick();

            cycles = cpu.total_cycles - prev;
            prev = cpu.total_cycles;

            for _i in 0..cycles {
                cpu.bus.tick();
            }

            /* Serial output */
/*
            if cpu.bus.read_byte(0x6001) == 0xDE /*&&
               cpu.bus.read_byte(0x6002) == 0xB0 &&
               cpu.bus.read_byte(0x6003) == 0xF1*/ { /* idk what is G1 */
                let status = cpu.bus.read_byte(0x6000);
                let mut addr = 0x6004;

                if status < 0x80 {
                    println!("Result: {}\n", status);

                    while cpu.bus.read_byte(addr) > 0 {
                        print!("{}", cpu.bus.read_byte(addr) as char);
                        addr += 1;
                    }
                }
            }
*/
            total += cycles;
        }

        texture.update(None, &cpu.bus.ppu.pixels, ppu::WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        thread::sleep(Duration::from_millis(DELAY));
    }
}
