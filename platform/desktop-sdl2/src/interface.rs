use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;

use sdl2::render::Texture;
use sdl2::keyboard::Keycode;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use native_dialog::{FileDialog, MessageDialog, MessageType};

use nesty::emulator::*;
use nesty::{savable::Savable, ppu, joypad};

lazy_static! {
    static ref KEY_MAP: HashMap<Keycode, u8> = {
        let mut key_map = HashMap::new();

        key_map.insert(Keycode::Down, joypad::BUTTON_DOWN);
        key_map.insert(Keycode::Up, joypad::BUTTON_UP);
        key_map.insert(Keycode::Right, joypad::BUTTON_RIGHT);
        key_map.insert(Keycode::Left, joypad::BUTTON_LEFT);
        key_map.insert(Keycode::Space, joypad::BUTTON_SELECT);
        key_map.insert(Keycode::Return, joypad::BUTTON_START);
        key_map.insert(Keycode::A, joypad::BUTTON_A);
        key_map.insert(Keycode::S, joypad::BUTTON_B);

        key_map
    };
}

pub struct Nesty {
    nes: Emulator,

    saving: bool,
    wait_for_nmi: bool,

    path: Option<PathBuf>
}

impl Nesty {
    pub fn new() -> Self {
        Nesty {
            nes: Emulator::new(),
            saving: false,
            wait_for_nmi: false,
            path: None
        }
    }

    pub fn init(&mut self) {
        self.nes.reset();
    }

    pub fn open_rom(&mut self) {
        let path = FileDialog::new()
            .add_filter(".nes ROM", &["nes"])
            .show_open_single_file()
            .expect("There are problems when creating a file dialog");
        if !path.is_none() {
            let rom = fs::read(path.unwrap()).unwrap();
            let result = self.nes.load_rom(rom);
            match result {
                Ok(_) => self.nes.reset(),
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
    }

    pub fn save_state(&mut self) {
        let path = FileDialog::new()
            .add_filter(".sav", &["sav"])
            .show_save_single_file()
            .expect("There are problems when creating a file dialog");
        if !path.is_none() {
            self.path = path;
            self.saving = true;
            self.wait_for_nmi = true;
        }
    }

    pub fn load_state(&mut self) {
        let path = FileDialog::new()
            .add_filter(".sav", &["sav"])
            .show_open_single_file()
            .expect("There are problems when creating a file dialog");
        if !path.is_none() {
            self.path = path;
            self.saving = false;
            self.wait_for_nmi = true;
        }
    }

    pub fn update(&mut self, texture: &mut Texture) {
        let mut total: u64 = 0;

        while total < CYCLES_PER_FRAME {
            if self.wait_for_nmi && self.nes.ppu().nmi {
                if self.saving {
                    let mut file = File::create(self.path.as_ref().unwrap()).expect("Unable to create save file");
                    let mut state = Vec::new();

                    self.nes.save_state(&mut state);
                    state.write_u64::<LittleEndian>(total).expect("Unable to save u64");
                    file.write_all(&state).expect("Unable to write to the save file");
                } else {
                    let mut file = File::open(self.path.as_ref().unwrap()).expect("Unable to open the save file");
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).expect("Unable to read the save file");
                    let mut cursor = Cursor::new(buffer);

                    self.nes.load_state(&mut cursor);
                    total = cursor.read_u64::<LittleEndian>().expect("Unable to load u64");
                }
    
                self.wait_for_nmi = false;
            }

            total += self.nes.tick();
        }

        texture.update(None, &self.nes.ppu().pixels, ppu::WIDTH * 3).unwrap();
    }

    pub fn press_key(&mut self, keycode: Keycode) {
        let key = KEY_MAP.get(&keycode);
        if !key.is_none() {
            self.nes.joypad().press(*key.unwrap());
        }
    }

    pub fn release_key(&mut self, keycode: Keycode) {
        let key = KEY_MAP.get(&keycode);
        if !key.is_none() {
            self.nes.joypad().release(*key.unwrap());
        }
    }
}
