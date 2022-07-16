use std::fs;

use crate::io::IO;
use crate::mapper::{Mirroring, Mapper, PRG_ROM_BANK_SIZE, CHR_ROM_BANK_SIZE};

use crate::mapper::mapper0::Mapper0;

use crate::test_bit;

const INES_IDENT: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

const EXPANSION_AREA_SIZE: usize = 0x1FE0;
const SRAM_SIZE: usize = 0x2000;

pub struct Cartridge {
    mapper: Box<dyn Mapper>,
    expansion_area: Vec<u8>,
    sram: Vec<u8>
}

impl Cartridge {
    pub fn new(fname: &str) -> Self {
        let rom = fs::read(fname).unwrap();

        if &rom[0..4] != INES_IDENT {
            panic!("File is not in iNES file format");
        }

        if ((rom[7] >> 2) & 0b11) != 0 {
            panic!("NES2.0 format is not supported");
        }

        let mirroring_type = if test_bit!(rom[6], 0) {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontial
        };

        let prg_rom_banks = rom[4] as usize;
        let chr_rom_banks = rom[5] as usize;

        let prg_rom_size = prg_rom_banks * PRG_ROM_BANK_SIZE;
        let chr_rom_size = chr_rom_banks * CHR_ROM_BANK_SIZE;

        let trainer_exists = test_bit!(rom[6], 2);

        let prg_rom_start = 0x10 + if trainer_exists { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let prg_rom = rom[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();
        let chr_rom = rom[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec();

        let mapper_type = (rom[6] >> 4) | (rom[7] & 0b11110000);
        let mapper = match mapper_type {
            0 => Box::new(Mapper0::new(mirroring_type, prg_rom_size, prg_rom, chr_rom)),
            _ => panic!("Unknown mapper: {}", mapper_type)
        };

        Cartridge {
            mapper: mapper,
            expansion_area: vec![0; EXPANSION_AREA_SIZE],
            sram: vec![0; SRAM_SIZE]
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring()
    }
}

impl IO for Cartridge {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            /* Accessed by PPU */
            0x0000..=0x1FFF => self.mapper.ppu_read_byte(addr),

            /* Accessed by CPU */
            0x4020..=0x5FFF => self.expansion_area[(addr - 0x4020) as usize],
            0x6000..=0x7FFF => self.sram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => self.mapper.cpu_read_byte(addr),
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            /* Accessed by PPU */
            0x0000..=0x1FFF => {
                self.mapper.ppu_write_byte(addr, data);
            }

            /* Accessed by CPU */
            0x4020..=0x5FFF => {
                self.expansion_area[(addr - 0x4020) as usize] = data;
            }
            0x6000..=0x7FFF => {
                self.sram[(addr - 0x6000) as usize] = data;
            }
            0x8000..=0xFFFF => {
                self.mapper.cpu_write_byte(addr, data);
            }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
