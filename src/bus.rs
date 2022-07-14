use crate::ppu::PPU;
use crate::joypad::Joypad;

use crate::traits::IO;

use crate::mirror;

const RAM_SIZE: usize = 0x800;
const PPU_REG_COUNT: usize = 0x8;
const IO_REGS_COUNT: usize = 0x20;
const CART_OTHER_SIZE: usize = 0x3FE0;

/*
CPU Memory Map (16bit buswidth, 0-FFFFh)
  0000h-07FFh   Internal 2K Work RAM (mirrored to 800h-1FFFh)
  2000h-2007h   Internal PPU Registers (mirrored to 2008h-3FFFh)
  4000h-401Fh   For use in APU and other IO devices
  4020h-5FFFh   Cartridge Expansion Area almost 8K
  6000h-7FFFh   Cartridge SRAM Area 8K
  8000h-FFFFh   Cartridge PRG-ROM Area 32K
*/
pub struct Bus {
    pub ppu: PPU,
    pub joypad: Joypad,

    ram: [u8; RAM_SIZE],
    io_regs: [u8; IO_REGS_COUNT], /* TODO replace this field with other devices like APU */

    /* cartridge (TODO implement mapper) */
    cart_other: Vec<u8>, /* 4020h-7FFFh */
    cart_prg_rom: Vec<u8>
}

impl Bus {
    pub fn new(prg_rom: Vec<u8>, ppu: PPU) -> Self {
        Bus {
            ppu: ppu,
            joypad: Joypad::new(),
            ram: [0; RAM_SIZE],
            io_regs: [0; IO_REGS_COUNT],
            cart_other: vec![0; CART_OTHER_SIZE],
            cart_prg_rom: prg_rom
        }
    }

    pub fn reset(&mut self) {
        self.ppu.reset();
    }

    // Update all devices in bus for each CPU cycle
    pub fn tick(&mut self) {
        self.ppu.tick();
        self.ppu.tick();
        self.ppu.tick();
    }
}

impl IO for Bus {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[mirror!(0x0000, addr, RAM_SIZE)],
            0x2000..=0x3FFF => self.ppu.read_register(mirror!(0x2000, addr, PPU_REG_COUNT)),
            0x4000..=0x401F => {
                if addr == 0x4016 {
                    return self.joypad.read()
                }
                return self.io_regs[(addr - 0x4000) as usize]
            },
            0x4020..=0x7FFF => self.cart_other[(addr - 0x4020) as usize],
            0x8000..=0xFFFF => self.cart_prg_rom[mirror!(0x8000, addr, self.cart_prg_rom.len())]
        }
    }

    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        /* TODO I think some address ranges are read only... */
        match addr {
            0x0000..=0x1FFF => { self.ram[mirror!(0x0000, addr, RAM_SIZE)] = data; }
            0x2000..=0x3FFF => { self.ppu.write_register(mirror!(0x2000, addr, PPU_REG_COUNT), data); }
            0x4000..=0x401F => {
                // OAM DMA
                if addr == 0x4014 {
                    let oam_start = (data as u16) << 8;
                    for i in 0..=255 {
                        let oam_data = self.read_byte(oam_start + i);
                        self.ppu.dma_write_oam(oam_data);
                    }
                    // TODO increase CPU cycles
                    return;
                }
                if addr == 0x4016 {
                    self.joypad.write(data);
                } else {
                    self.io_regs[(addr - 0x4000) as usize] = data;
                }
            }
            0x4020..=0x7FFF => { self.cart_other[(addr - 0x4020) as usize] = data; }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
