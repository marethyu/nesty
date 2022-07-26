#[macro_use]
extern crate lazy_static;

mod utils;

use std::io::Cursor;
use std::collections::HashMap;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys;
use web_sys::KeyEvent;
use serde_json;

use nesty::emulator::*;
use nesty::{savable::Savable, joypad};

/* TODO keycodes are deprecated, need something else... */
lazy_static! {
    static ref KEY_MAP: HashMap<u32, u8> = {
        let mut key_map = HashMap::new();

        key_map.insert(KeyEvent::DOM_VK_DOWN, joypad::BUTTON_DOWN);
        key_map.insert(KeyEvent::DOM_VK_UP, joypad::BUTTON_UP);
        key_map.insert(KeyEvent::DOM_VK_RIGHT, joypad::BUTTON_RIGHT);
        key_map.insert(KeyEvent::DOM_VK_LEFT, joypad::BUTTON_LEFT);
        key_map.insert(KeyEvent::DOM_VK_SPACE, joypad::BUTTON_SELECT);
        key_map.insert(KeyEvent::DOM_VK_RETURN, joypad::BUTTON_START);
        key_map.insert(KeyEvent::DOM_VK_A, joypad::BUTTON_A);
        key_map.insert(KeyEvent::DOM_VK_S, joypad::BUTTON_B);

        key_map
    };
}

#[wasm_bindgen]
pub struct NestyWeb {
    emu: Emulator,
    saving: bool,
    wait_for_nmi: bool
}

#[wasm_bindgen]
impl NestyWeb {
    pub fn new() -> Self {
        NestyWeb {
            emu: Emulator::new(),
            saving: false,
            wait_for_nmi: false
        }
    }

    pub fn load_rom(&mut self, rom_data: Uint8Array) -> bool {
        let result = self.emu.load_rom(rom_data.to_vec());
        match result {
            Ok(_) => true,
            _ => false
        }
    }

    pub fn reset(&mut self) {
        self.emu.reset();
    }

    pub fn save_state(&mut self) {
        self.saving = true;
        self.wait_for_nmi = true;
    }

    pub fn load_state(&mut self) {
        self.saving = false;
        self.wait_for_nmi = true;
    }

    pub fn update(&mut self) {
        utils::set_panic_hook();

        let mut total: u64 = 0;

        while total < CYCLES_PER_FRAME {
            if self.wait_for_nmi && self.emu.ppu().nmi {
                let window = web_sys::window().unwrap();
                let storage = window.local_storage().unwrap().unwrap();

                if self.saving {
                    let mut state = Vec::new();
                    self.emu.save_state(&mut state);
                    state.write_u64::<LittleEndian>(total).expect("Unable to save u64");

                    let serialized_json = serde_json::to_string(&state).expect("Unable to create JSON");
                    storage.set_item("nesty-save-state", &serialized_json);
                    window.alert_with_message("State saved!");
                } else {
                    let serialized_json = storage.get_item("nesty-save-state").expect("Failed to retrieve save state").unwrap();
                    let buffer: Vec<u8> = serde_json::from_str(&serialized_json).expect("JSON deserialization failed");
                    let mut cursor = Cursor::new(buffer);

                    self.emu.load_state(&mut cursor);
                    total = cursor.read_u64::<LittleEndian>().expect("Unable to load u64");
                    window.alert_with_message("State loaded!");
                }
    
                self.wait_for_nmi = false;
            }

            total += self.emu.tick();
        }
    }

    pub fn pixel_buffer(&self) -> Uint8Array {
        let pixels: &[u8] = &self.emu.ppu().pixels;
        Uint8Array::from(pixels)
    }

    pub fn press_key(&mut self, keycode: u32) {
        let key = KEY_MAP.get(&keycode);
        if !key.is_none() {
            self.emu.joypad().press(*key.unwrap());
        }
    }

    pub fn release_key(&mut self, keycode: u32) {
        let key = KEY_MAP.get(&keycode);
        if !key.is_none() {
            self.emu.joypad().release(*key.unwrap());
        }
    }
}
