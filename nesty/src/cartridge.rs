use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::io::IO;
use crate::savable::Savable;
use crate::mapper::{Mirroring, Mapper, PRG_ROM_BANK_SIZE, CHR_ROM_BANK_SIZE};

use crate::mapper::mapper0::Mapper0;
use crate::mapper::mapper1::Mapper1;

use crate::{test_bit, box_array};

const INES_IDENT: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

const EXPANSION_AREA_SIZE: usize = 0x1FE0;

pub struct Cartridge {
    mapper: Box<dyn Mapper>,
    expansion_area: Box<[u8; EXPANSION_AREA_SIZE]>
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let (mapper, _) = Cartridge::parse_metadata(rom);

        Cartridge {
            mapper: mapper.unwrap(),
            expansion_area: box_array![0; EXPANSION_AREA_SIZE]
        }
    }

    pub fn load(&mut self, rom: Vec<u8>) -> Result<String, String> {
        let (mapper, mapno) = Cartridge::parse_metadata(rom);

        if mapper.is_none() {
            return Err(format!("Mapper {} is not supported yet sorry my man", mapno))
        }

        self.mapper = mapper.unwrap();

        Ok("".to_string())
    }

    pub fn reset(&mut self) {
        self.mapper.reset();
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring()
    }

    fn parse_metadata(rom: Vec<u8>) -> (Option<Box<dyn Mapper>>, u8) {
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

        let use_chr_ram = chr_rom_banks == 0;

        let prg_rom_size = prg_rom_banks * PRG_ROM_BANK_SIZE;
        let chr_rom_size = if use_chr_ram {
            CHR_ROM_BANK_SIZE
        } else {
            chr_rom_banks * CHR_ROM_BANK_SIZE
        };

        let trainer_exists = test_bit!(rom[6], 2);

        let prg_rom_start = 0x10 + if trainer_exists { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let prg_rom = rom[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();
        let chr_rom = if use_chr_ram {
            vec![0; chr_rom_size]
        } else {
            rom[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        };

        let mapper_type = (rom[6] >> 4) | (rom[7] & 0b11110000);
        let mapper: Option<Box<dyn Mapper>> = match mapper_type {
            0 => Some(Box::new(Mapper0::new(mirroring_type, prg_rom_size, prg_rom, chr_rom))),
            1 => Some(Box::new(Mapper1::new(mirroring_type, prg_rom_banks, chr_rom_banks, prg_rom, chr_rom))),
            _ => None
        };

        (mapper, mapper_type)
    }
}

impl IO for Cartridge {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            /* Accessed by PPU */
            0x0000..=0x1FFF => self.mapper.ppu_read_byte(addr),

            /* Accessed by CPU */
            0x4020..=0x5FFF => self.expansion_area[(addr - 0x4020) as usize],
            0x6000..=0xFFFF => self.mapper.cpu_read_byte(addr),
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
            0x6000..=0xFFFF => {
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

impl Savable for Cartridge {
    fn save_state(&self, state: &mut Vec<u8>) {
        self.mapper.save_state(state);
        for i in 0..EXPANSION_AREA_SIZE {
            state.write_u8(self.expansion_area[i]).expect("Unable to save u8");
        }
    }

    fn load_state(&mut self, state: &mut Cursor<Vec<u8>>) {
        self.mapper.load_state(state);
        for i in 0..EXPANSION_AREA_SIZE {
            self.expansion_area[i] = state.read_u8().expect("Unable to load u8");
        }
    }
}
