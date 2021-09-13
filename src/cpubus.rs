use crate::traits::IO;

use std::fs;

pub struct CPUBus {
    mem: Vec<u8>
}

fn load_mem() -> Vec<u8> {
    let mut mem = vec![0; 0x10000];
    let cart = fs::read("roms/nestest.nes").unwrap();

    for i in 0..0x4000 {
        mem[0xC000 + i] = cart[i + 0x10];
    }

    mem
}

impl CPUBus {
    pub fn new() -> Self {
        CPUBus {
            mem: load_mem()
        }
    }
}

impl IO for CPUBus {
    fn read_byte(&self, addr: u16) -> u8 {
        if addr <= 0x2000 {
            self.mem[(addr & 0x7FF) as usize]
        } else {
            self.mem[addr as usize]
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        if addr <= 0x2000 {
            self.mem[(addr & 0x7FF) as usize] = data;
        } else {
            self.mem[addr as usize] = data;
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
