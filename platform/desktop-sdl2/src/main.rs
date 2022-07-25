use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::prelude::*;
use std::process;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use native_dialog::{FileDialog, MessageDialog, MessageType};

use nesty::emulator::*;
use nesty::{savable::Savable, ppu, joypad};

const DELAY: u32 = 17; // 1000ms / 59.7fps

pub fn main() {
    // TODO custom rom
    let rom = fs::read("roms/nestest.nes").unwrap();
    let mut nes = Emulator::new(rom);

    nes.reset();

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

    let mut event_pump = sdl_context.event_pump().unwrap();

    let timer_subsystem = sdl_context.timer().unwrap();
    let mut next = timer_subsystem.ticks() + DELAY;

    let mut saving = false;
    let mut wait_for_nmi = false;

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if matches!(keycode.unwrap(), Keycode::I) {
                        /* Load ROM */
                        let path = FileDialog::new()
                            .add_filter(".nes ROM", &["nes"])
                            .show_open_single_file()
                            .expect("There are problems when creating a file dialog");
                        if !path.is_none() {
                            let rom = fs::read(path.unwrap()).unwrap();

                            let result = nes.load_rom(rom);
                            match result {
                                Ok(_) => nes.reset(),
                                Err(err) => {
                                    let _ = MessageDialog::new()
                                        .set_type(MessageType::Error)
                                        .set_title("Error opening ROM")
                                        .set_text(&format!("{}", err))
                                        .show_confirm()
                                        .unwrap();
                                }
                            }
                        }
                    } else if matches!(keycode.unwrap(), Keycode::O) {
                        /* Save state */
                        saving = true;
                        wait_for_nmi = true;
                    } else if matches!(keycode.unwrap(), Keycode::P) {
                        /* Load state */
                        saving = false;
                        wait_for_nmi = true;
                    }
                    else if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
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

        let mut total: u64 = 0;

        while total < CYCLES_PER_FRAME {
            if wait_for_nmi && nes.ppu().nmi {
                if saving {
                    let mut file = File::create("nesty.sav").expect("Unable to create save file");
                    let mut state = Vec::new();
                    nes.save_state(&mut state);
                    state.write_u64::<LittleEndian>(total).expect("Unable to save u64");
                    file.write_all(&state).expect("Unable to write to the save file");
                    println!("Saved state!");
                } else {
                    let mut file = File::open("nesty.sav").expect("Unable to open the save file");
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).expect("Unable to read the save file");
                    let mut cursor = Cursor::new(buffer);
                    nes.load_state(&mut cursor);
                    total = cursor.read_u64::<LittleEndian>().expect("Unable to load u64");
                    println!("Loaded state!");
                }
    
                wait_for_nmi = false;
            }

            total += nes.tick();
        }

        texture.update(None, &nes.ppu().pixels, ppu::WIDTH * 3).unwrap();
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
