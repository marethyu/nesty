use crate::mapper::{Mirroring, Mapper};

use crate::mirror;

pub struct Mapper0 {
    mirroring_type: Mirroring,

    prg_rom_size: usize,

    prg_rom: Vec<u8>, /* TODO it can be written?!? */
    chr_rom: Vec<u8>, /* TODO it can be written?!? */
}

impl Mapper0 {
    pub fn new(mirroring_type: Mirroring, prg_rom_size: usize, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Mapper0 {
            mirroring_type: mirroring_type,
            prg_rom_size: prg_rom_size,
            prg_rom: prg_rom,
            chr_rom: chr_rom
        }
    }
}

impl Mapper for Mapper0 {
    fn cpu_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => self.prg_rom[mirror!(0x8000, addr, self.prg_rom_size)],
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn cpu_write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0xFFFF => {}
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn ppu_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.chr_rom[addr as usize],
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn ppu_write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {}
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn mirroring(&self) -> Mirroring {
        return self.mirroring_type;
    }
}
