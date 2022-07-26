#[macro_use]
extern crate lazy_static;

mod interface;

use std::process;
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use nesty::ppu;

use crate::interface::Nesty;

const DELAY: u32 = 17; // 1000ms / 59.7fps

pub fn main() {
    let mut nesty = Nesty::new();

    nesty.init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("NESTY", (ppu::WIDTH * 2) as u32, (ppu::HEIGHT * 2) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ABGR8888, ppu::WIDTH as u32, ppu::HEIGHT as u32)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let timer_subsystem = sdl_context.timer().unwrap();
    let mut next = timer_subsystem.ticks() + DELAY;

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if matches!(keycode.unwrap(), Keycode::I) {
                        nesty.open_rom();
                    } else if matches!(keycode.unwrap(), Keycode::O) {
                        nesty.save_state();
                    } else if matches!(keycode.unwrap(), Keycode::P) {
                        nesty.load_state();
                    } else {
                        nesty.press_key(keycode.unwrap());
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    nesty.release_key(keycode.unwrap());
                }
                _ => {}
            }
        }

        nesty.update(&mut texture);

        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        let now = timer_subsystem.ticks();
        let delay = if now < next {
            next - now
        } else {
            0
        };

        thread::sleep(Duration::from_millis(delay as u64));

        next += DELAY;
    }
}
