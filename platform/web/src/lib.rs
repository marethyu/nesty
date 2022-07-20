use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use core::emulator::Emulator;

#[wasm_bindgen]
pub struct Nesty {
    emu: Emulator
}

#[wasm_bindgen]
impl Nesty {
    pub fn new(fname: &str) -> Self {
        Nesty {
            emu: Emulator::new(fname)
        }
    }

    pub fn reset(&mut self) {
        self.emu.reset();
    }

    pub fn update(&mut self) {
        self.emu.update();
    }

    pub fn pixel_buffer(&self) -> Uint8Array {
        let pixels: &[u8] = &self.emu.ppu().pixels;
        Uint8Array::from(pixels)
    }

    pub fn press_key(&mut self, key: u8) {
        self.emu.joypad().press(key);
    }

    pub fn release_key(&mut self, key: u8) {
        self.emu.joypad().release(key);
    }
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
