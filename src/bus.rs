use std::cell::RefCell;
use std::sync::{Arc, Weak};

use crate::ppu::PPU;
use crate::cartridge::Cartridge;
use crate::joypad::Joypad;

use crate::io::IO;

#[macro_export]
macro_rules! mirror {
    ($base:expr, $addr:expr, $size:expr) => {
        (($addr - $base) & (($size as u16) - 1)) as usize
    }
}

const RAM_SIZE: usize = 0x800;
const PPU_REG_COUNT: usize = 0x8;
const IO_REGS_COUNT: usize = 0x20;

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
    cart: Weak<RefCell<Cartridge>>,
    ppu: Weak<RefCell<PPU>>,
    joypad: Weak<RefCell<Joypad>>,

    ram: Vec<u8>,
    io_regs: Vec<u8>
}

impl Bus {
    pub fn new(weak_cart: Weak<RefCell<Cartridge>>,
               weak_ppu: Weak<RefCell<PPU>>,
               weak_joypad: Weak<RefCell<Joypad>>) -> Self {
        Bus {
            cart: weak_cart.clone(),
            ppu: weak_ppu.clone(),
            joypad: weak_joypad.clone(),

            ram: vec![0; RAM_SIZE],
            io_regs: vec![0; IO_REGS_COUNT]
        }
    }

    pub fn cart(&self) -> Arc<RefCell<Cartridge>> {
        self.cart.upgrade().expect("Cartridge lost for bus")
    }

    pub fn ppu(&self) -> Arc<RefCell<PPU>> {
        self.ppu.upgrade().expect("PPU lost for bus")
    }

    pub fn joypad(&self) -> Arc<RefCell<Joypad>> {
        self.joypad.upgrade().expect("Joypad lost for bus")
    }
}

impl IO for Bus {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[mirror!(0x0000, addr, RAM_SIZE)],
            0x2000..=0x3FFF => self.ppu().borrow_mut().read_register(mirror!(0x2000, addr, PPU_REG_COUNT)),
            0x4000..=0x401F => {
                if addr == 0x4016 {
                    return self.joypad().borrow_mut().read()
                }
                return self.io_regs[(addr - 0x4000) as usize]
            },
            0x4020..=0xFFFF => self.cart().borrow_mut().read_byte(addr),
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
            0x0000..=0x1FFF => { self.ram[mirror!(0x0000, addr, RAM_SIZE)] = data; }
            0x2000..=0x3FFF => { self.ppu().borrow_mut().write_register(mirror!(0x2000, addr, PPU_REG_COUNT), data); }
            0x4000..=0x401F => {
                // OAM DMA
                if addr == 0x4014 {
                    let oam_start = (data as u16) << 8;
                    for i in 0..=255 {
                        let oam_data = self.read_byte(oam_start + i);
                        self.ppu().borrow_mut().dma_write_oam(oam_data);
                    }
                    // TODO increase CPU cycles
                    return;
                }
                if addr == 0x4016 {
                    self.joypad().borrow_mut().write(data);
                } else {
                    self.io_regs[(addr - 0x4000) as usize] = data;
                }
            }
            0x4020..=0xFFFF => { self.cart().borrow_mut().write_byte(addr, data); }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
