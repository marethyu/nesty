use std::fs;

use crate::test_bit;

const INES_IDENT: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

pub enum Mirroring {
    Vertical,
    Horizontial
}

pub struct Cartridge {
    pub mapper_type: u8,
    pub mirroring_type: Mirroring,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>
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

        let mapper_type = (rom[6] >> 4) | (rom[7] & 0b11110000);
        if mapper_type != 0 {
            panic!("Mapper must be NROM, other types are not supported yet");
        }

        let mirroring_type = if test_bit!(rom[6], 0) {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontial
        };

        let prg_rom_size = rom[4] as usize * 0x4000;
        let chr_rom_size = rom[5] as usize * 0x2000;

        let trainer_exists = test_bit!(rom[6], 2);

        let prg_rom_start = 0x10 + if trainer_exists { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Cartridge {
            mapper_type: mapper_type,
            mirroring_type: mirroring_type,
            prg_rom: rom[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: rom[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        }
    }
}
