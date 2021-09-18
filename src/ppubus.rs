use crate::cartridge::Mirroring;
use crate::traits::IO;

use crate::mirror;

const NAMETABLE_SIZE: usize = 0x400;
const PALETTE_RAM_SIZE: usize = 0x20;

/*
PPU Memory Map (14bit buswidth, 0-3FFFh)
  0000h-0FFFh   Pattern Table 0 (4K) (256 Tiles)
  1000h-1FFFh   Pattern Table 1 (4K) (256 Tiles)
  2000h-23FFh   Name Table 0 and Attribute Table 0 (1K) (32x30 BG Map)
  2400h-27FFh   Name Table 1 and Attribute Table 1 (1K) (32x30 BG Map)
  2800h-2BFFh   Name Table 2 and Attribute Table 2 (1K) (32x30 BG Map)
  2C00h-2FFFh   Name Table 3 and Attribute Table 3 (1K) (32x30 BG Map)
  3000h-3EFFh   Mirror of 2000h-2EFFh
  3F00h-3F1Fh   Background and Sprite Palettes (25 entries used)
  3F20h-3FFFh   Mirrors of 3F00h-3F1Fh
*/
pub struct PPUBus {
    cart_chr_rom: Vec<u8>, /* contains pattern table */
    mirroring_type: Mirroring,
    nametable: [[u8; NAMETABLE_SIZE]; 4],
    palette_ram: [u8; PALETTE_RAM_SIZE]
}

impl PPUBus {
    pub fn new(chr_rom: Vec<u8>, mirroring_type: Mirroring) -> Self {
        PPUBus {
            cart_chr_rom: chr_rom,
            mirroring_type: mirroring_type,
            nametable: [[0; NAMETABLE_SIZE]; 4],
            palette_ram: [0; PALETTE_RAM_SIZE]
        }
    }
}

impl IO for PPUBus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.cart_chr_rom[addr as usize],
            0x2000..=0x3EFF => {
                let a = mirror!(0x2000, addr, NAMETABLE_SIZE * 4);
                let idx = match self.mirroring_type {
                    Mirroring::Vertical => (a & (NAMETABLE_SIZE * 2 - 1)) / NAMETABLE_SIZE,
                    Mirroring::Horizontial => a / (NAMETABLE_SIZE * 2),
                    Mirroring::FourScreen => a / NAMETABLE_SIZE
                };
                self.nametable[idx][a & (NAMETABLE_SIZE - 1)]
            }
            0x3F00..=0x3FFF => self.palette_ram[mirror!(0x3F00, addr, PALETTE_RAM_SIZE)],
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000..=0x3EFF => {
                let a = mirror!(0x2000, addr, NAMETABLE_SIZE * 4);
                let idx = match self.mirroring_type {
                    Mirroring::Vertical => (a & (NAMETABLE_SIZE * 2 - 1)) / NAMETABLE_SIZE,
                    Mirroring::Horizontial => a / (NAMETABLE_SIZE * 2),
                    Mirroring::FourScreen => a / NAMETABLE_SIZE
                };
                self.nametable[idx][a & (NAMETABLE_SIZE - 1)] = data;
            }
            0x3F00..=0x3FFF => {
                self.palette_ram[mirror!(0x3F00, addr, PALETTE_RAM_SIZE)] = data;
            }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
