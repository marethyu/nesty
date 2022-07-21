mod utils;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use core::emulator::Emulator;

#[wasm_bindgen]
pub struct Nesty {
    emu: Option<Emulator>
}

#[wasm_bindgen]
impl Nesty {
    pub fn new() -> Self {
        Nesty {
            emu: None
        }
    }

    pub fn load_rom(&mut self, rom_data: Uint8Array) {
        self.emu = Some(Emulator::new(rom_data.to_vec()));
    }

    pub fn reset(&mut self) {
        if let Some(ref mut emu) = self.emu {
            emu.reset();
        } else {
            println!("Nesty is not initialized!");
        }
    }

    pub fn update(&mut self) {
        utils::set_panic_hook();

        if let Some(ref mut emu) = self.emu {
            emu.update();
        } else {
            println!("Nesty is not initialized!");
        }
    }

    pub fn pixel_buffer(&self) -> Uint8Array {
        if let Some(ref emu) = self.emu {
            let pixels: &[u8] = &emu.ppu().pixels;
            return Uint8Array::from(pixels)
        } else {
            println!("Nesty is not initialized!");
            return Uint8Array::new_with_length(0);
        }
    }

    pub fn press_key(&mut self, key: u8) {
        if let Some(ref emu) = self.emu {
            emu.joypad().press(key);
        } else {
            println!("Nesty is not initialized!");
        }
    }

    pub fn release_key(&mut self, key: u8) {
        if let Some(ref emu) = self.emu {
            emu.joypad().release(key);
        } else {
            println!("Nesty is not initialized!");
        }
    }
}
