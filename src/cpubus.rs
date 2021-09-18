use crate::traits::IO;

const RAM_SIZE: usize = 0x800;
const PPU_REG_SIZE: usize = 0x8;
const IO_REGS_SIZE: usize = 0x20;
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
pub struct CPUBus {
    ram: [u8; RAM_SIZE],
    ppu_regs: [u8; PPU_REG_SIZE], /* TODO replace this field with PPU in future impl */
    io_regs: [u8; IO_REGS_SIZE], /* TODO replace this field with other devices like APU, joystick */

    /* cartridge (TODO implement mapper) */
    cart_other: [u8; CART_OTHER_SIZE], /* 4020h-7FFFh */
    cart_prg_rom: Vec<u8>
}

macro_rules! mirror {
    ($base:expr, $addr:expr, $size:expr) => {
        (($addr - $base) & (($size as u16) - 1)) as usize
    }
}

impl CPUBus {
    pub fn new(prg_rom: Vec<u8>) -> Self {
        CPUBus {
            ram: [0; RAM_SIZE],
            ppu_regs: [0; PPU_REG_SIZE],
            io_regs: [0; IO_REGS_SIZE],
            cart_other: [0; CART_OTHER_SIZE],
            cart_prg_rom: prg_rom
        }
    }
}

impl IO for CPUBus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[mirror!(0x0000, addr, RAM_SIZE)],
            0x2000..=0x3FFF => self.ppu_regs[mirror!(0x2000, addr, PPU_REG_SIZE)],
            0x4000..=0x401F => self.io_regs[(addr - 0x4000) as usize],
            0x4020..=0x7FFF => self.cart_other[(addr - 0x4020) as usize],
            0x8000..=0xFFFF => self.cart_prg_rom[mirror!(0x8000, addr, self.cart_prg_rom.len())],
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        /* TODO I think some address ranges are read only... */
        match addr {
            0x0000..=0x1FFF => { self.ram[mirror!(0x0000, addr, RAM_SIZE)] = data; }
            0x2000..=0x3FFF => { self.ppu_regs[mirror!(0x2000, addr, PPU_REG_SIZE)] = data; }
            0x4000..=0x401F => { self.io_regs[(addr - 0x4000) as usize] = data; }
            0x4020..=0x7FFF => { self.cart_other[(addr - 0x4020) as usize] = data; }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
