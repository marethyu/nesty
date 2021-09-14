use crate::traits::IO;

const RAM_SIZE: usize = 0x800;
const PPU_REG_SIZE: usize = 0x8;
const OTHER_SIZE: usize = 0x20;
const CART_MEM_SIZE: usize = 0xBFE0;

pub struct CPUBus {
    ram: [u8; RAM_SIZE],
    ppu_regs: [u8; PPU_REG_SIZE], /* TODO replace this field with PPU in future impl */
    other: [u8; OTHER_SIZE], /* TODO replace this field with other devices like joystick */
    cart: [u8; CART_MEM_SIZE]
}

macro_rules! mirror {
    ($base:expr, $addr:expr, $size:expr) => {
        (($addr - $base) & (($size as u16) - 1)) as usize
    }
}

impl CPUBus {
    pub fn new(cart_mem: [u8; CART_MEM_SIZE]) -> Self {
        CPUBus {
            ram: [0; RAM_SIZE],
            ppu_regs: [0; PPU_REG_SIZE],
            other: [0; OTHER_SIZE],
            cart: cart_mem
        }
    }
}

impl IO for CPUBus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[mirror!(0x0000, addr, RAM_SIZE)],
            0x2000..=0x3FFF => self.ppu_regs[mirror!(0x2000, addr, PPU_REG_SIZE)],
            0x4000..=0x401F => self.other[mirror!(0x4000, addr, OTHER_SIZE)],
            0x4020..=0xFFFF => self.cart[(addr - 0x4020) as usize],
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
            0x4000..=0x401F => { self.other[mirror!(0x4000, addr, OTHER_SIZE)] = data; }
            0x4020..=0xFFFF => { self.cart[(addr - 0x4020) as usize] = data; }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
