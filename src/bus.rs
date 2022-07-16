use std::cell::RefCell;
use std::cell::RefMut;
use std::sync::{Arc, Weak};

use crate::m6502::M6502;
use crate::ppu::PPU;
use crate::cartridge::Cartridge;
use crate::joypad::Joypad;

use crate::io::IO;

use crate::mirror;

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
    pub cpu: Arc<RefCell<M6502>>, /* requires access to bus */
    pub ppu: Arc<RefCell<PPU>>, /* requires access to cartridge */

    /* also shared by ppu since both bus and ppu need cartridge access. the real cartridge is created in main thread */
    pub cart: Weak<RefCell<Cartridge>>,
    pub joypad: Joypad,

    ram: [u8; RAM_SIZE],
    io_regs: [u8; IO_REGS_COUNT]
}

impl Bus {
    pub fn new(weak_bus: &Weak<RefCell<Bus>>,
               weak_cart: &Weak<RefCell<Cartridge>>) -> Self {
        Bus {
            cpu: Arc::new(RefCell::new(M6502::new(weak_bus.clone()))),
            ppu: Arc::new(RefCell::new(PPU::new(weak_cart.clone()))),

            cart: weak_cart.clone(),
            joypad: Joypad::new(),

            ram: [0; RAM_SIZE],
            io_regs: [0; IO_REGS_COUNT]
        }
    }

    pub fn cpu(&self) -> RefMut<'_, M6502> {
        self.cpu.borrow_mut()
    }

    pub fn ppu(&self) -> RefMut<'_, PPU> {
        self.ppu.borrow_mut()
    }

    pub fn cart(&self) -> Arc<RefCell<Cartridge>> {
        self.cart.upgrade().expect("Cartridge lost for bus")
    }
}

impl IO for Bus {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[mirror!(0x0000, addr, RAM_SIZE)],
            0x2000..=0x3FFF => self.ppu().read_register(mirror!(0x2000, addr, PPU_REG_COUNT)),
            0x4000..=0x401F => {
                if addr == 0x4016 {
                    return self.joypad.read()
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
            0x2000..=0x3FFF => { self.ppu().write_register(mirror!(0x2000, addr, PPU_REG_COUNT), data); }
            0x4000..=0x401F => {
                // OAM DMA
                if addr == 0x4014 {
                    let oam_start = (data as u16) << 8;
                    for i in 0..=255 {
                        let oam_data = self.read_byte(oam_start + i);
                        self.ppu().dma_write_oam(oam_data);
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
            0x4020..=0xFFFF => { self.cart().borrow_mut().write_byte(addr, data); }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
