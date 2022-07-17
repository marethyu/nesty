use crate::mapper::{Mirroring, Mapper, SRAM_SIZE};

use crate::{test_bit, mirror};

pub struct Mapper1 {
    mirroring_type: Mirroring,

    prg_rom_banks: usize,
    chr_rom_banks: usize,

    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    sram: Vec<u8>
}

impl Mapper1 {
    pub fn new(mirroring_type: Mirroring, prg_rom_banks: usize, chr_rom_banks: usize, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Mapper1 {
            mirroring_type: mirroring_type,

            prg_rom_banks: prg_rom_banks,
            chr_rom_banks: chr_rom_banks,

            prg_rom: prg_rom,
            chr_rom: chr_rom,

            sram: vec![0; SRAM_SIZE]
        }
    }

    /* TODO reset */
}

impl Mapper for Mapper1 {
    fn cpu_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => 0,
            0x8000..=0xFFFF => 0,
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn cpu_write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => {}
            0x8000..=0xFFFF => {}
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn ppu_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => 0,
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
