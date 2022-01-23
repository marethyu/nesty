mod macros;
mod traits;
mod cartridge;
mod m6502;
mod cpubus;
mod ppubus;
mod opcodes;

use std::rc::Rc;
use std::cell::RefCell;
use std::process;
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;

#[macro_use(lazy_static)]
extern crate lazy_static;

use cartridge::Cartridge;
use m6502::M6502;
use cpubus::CPUBus;
use ppubus::PPUBus;

use traits::IO;

const WIDTH: usize = 128; // 256
const HEIGHT: usize = 256; // 240

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("NESTY", (WIDTH * 2) as u32, (HEIGHT * 2) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let cart = Cartridge::new("roms/nestest.nes");

    let cpu = Rc::new(RefCell::new(M6502::new()));

    let cpu_bus = CPUBus::new(cart.prg_rom);
    let ppu_bus = PPUBus::new(cart.chr_rom, cart.mirroring_type);

    cpu.borrow_mut().load_bus(cpu_bus);
    cpu.borrow_mut().reset();

    let mut pixels = [0 as u8; WIDTH * HEIGHT * 3];

    let mut event_pump = sdl_context.event_pump().unwrap();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => process::exit(0),
                _ => {}
            }
        }

        let cycles = cpu.borrow_mut().step();
/*
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let offset = y * WIDTH * 3 + x * 3;
                pixels[offset    ] = 255;
                pixels[offset + 1] = 0;
                pixels[offset + 2] = 0;
            }
        }
*/
        // Display pattern table
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let ty = y / 8;
                let tx = x / 8;
                let pt_idx = ty * 16 + tx;
                let pt_addr = (pt_idx * 16) as u16;
                let yoffset = y % 8;
                let xoffset = x % 8;
                let tlow = ppu_bus.read_byte(pt_addr + (yoffset as u16));
                let thigh = ppu_bus.read_byte(pt_addr + (yoffset as u16) + 8);
                let val = ((test_bit!(thigh, 7 - xoffset) as u8) << 1) | (test_bit!(tlow, 7 - xoffset) as u8);
                let offset = y * WIDTH * 3 + x * 3;
                let color = val * 85;
                pixels[offset    ] = color;
                pixels[offset + 1] = color;
                pixels[offset + 2] = color;
            }
        }

        texture.update(None, &pixels, WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        thread::sleep(Duration::from_millis(17));
    }
}
