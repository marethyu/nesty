use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::mapper::{Mirroring, Mapper, MapperBase, SRAM_SIZE};

use crate::savable::Savable;
use crate::{mirror, box_array};

pub struct Mapper0 {
    mirroring_type: Mirroring,

    prg_rom_size: usize,

    prg_rom: Vec<u8>, /* TODO it can be written?!? */
    chr_rom: Vec<u8>, /* TODO it can be written?!? */

    sram: Box<[u8; SRAM_SIZE]>
}

impl Mapper0 {
    pub fn new(mirroring_type: Mirroring, prg_rom_size: usize, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Mapper0 {
            mirroring_type: mirroring_type,
            prg_rom_size: prg_rom_size,
            prg_rom: prg_rom,
            chr_rom: chr_rom,
            sram: box_array![0; SRAM_SIZE]
        }
    }
}

impl MapperBase for Mapper0 {
    fn reset(&mut self) {}

    fn cpu_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.sram[mirror!(0x6000, addr, SRAM_SIZE)],
            0x8000..=0xFFFF => self.prg_rom[mirror!(0x8000, addr, self.prg_rom_size)],
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn cpu_write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.sram[mirror!(0x6000, addr, SRAM_SIZE)] = data;
            }
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

impl Savable for Mapper0 {
    fn save_state(&self, state: &mut Vec<u8>) {
        for i in 0..SRAM_SIZE {
            state.write_u8(self.sram[i]).expect("Unable to save u8");
        }
    }

    fn load_state(&mut self, state: &mut Cursor<Vec<u8>>) {
        for i in 0..SRAM_SIZE {
            self.sram[i] = state.read_u8().expect("Unable to load u8");
        }
    }
}

impl Mapper for Mapper0 {}
