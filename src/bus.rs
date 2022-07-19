use std::cell::RefCell;
use std::rc::{Rc, Weak};

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

/// A macro similar to `vec![$elem; $size]` which returns a boxed array.
///
/// ```rustc
///     let _: Box<[u8; 1024]> = box_array![0; 1024];
/// ```
/// Source: https://stackoverflow.com/a/68122278/12126813
#[macro_export]
macro_rules! box_array {
    ($val:expr ; $len:expr) => {{
        // Use a generic function so that the pointer cast remains type-safe
        fn vec_to_boxed_array<T>(vec: Vec<T>) -> Box<[T; $len]> {
            let boxed_slice = vec.into_boxed_slice();

            let ptr = ::std::boxed::Box::into_raw(boxed_slice) as *mut [T; $len];

            unsafe { Box::from_raw(ptr) }
        }

        vec_to_boxed_array(vec![$val; $len])
    }};
}

const RAM_SIZE: usize = 0x800;
const PPU_REG_COUNT: usize = 0x8;
const IO_REGS_COUNT: usize = 0x20;

pub struct Bus {
    cart: Weak<RefCell<Cartridge>>,
    ppu: Weak<RefCell<PPU>>,
    joypad: Weak<RefCell<Joypad>>,

    ram: Box<[u8; RAM_SIZE]>,
    io_regs: Box<[u8; IO_REGS_COUNT]>,

    pub init_dma: bool,
    pub dma_start_addr: u16
}

impl Bus {
    pub fn new(weak_cart: Weak<RefCell<Cartridge>>,
               weak_ppu: Weak<RefCell<PPU>>,
               weak_joypad: Weak<RefCell<Joypad>>) -> Self {
        Bus {
            cart: weak_cart.clone(),
            ppu: weak_ppu.clone(),
            joypad: weak_joypad.clone(),

            ram: box_array![0; RAM_SIZE],
            io_regs: box_array![0; IO_REGS_COUNT],

            init_dma: false,
            dma_start_addr: 0
        }
    }

    pub fn cart(&self) -> Rc<RefCell<Cartridge>> {
        self.cart.upgrade().expect("Cartridge lost for bus")
    }

    pub fn ppu(&self) -> Rc<RefCell<PPU>> {
        self.ppu.upgrade().expect("PPU lost for bus")
    }

    pub fn joypad(&self) -> Rc<RefCell<Joypad>> {
        self.joypad.upgrade().expect("Joypad lost for bus")
    }
}

/*
CPU Memory Map (16bit buswidth, 0-FFFFh)
  0000h-07FFh   Internal 2K Work RAM (mirrored to 800h-1FFFh)
  2000h-2007h   Internal PPU Registers (mirrored to 2008h-3FFFh)
  4000h-401Fh   For use in APU and other IO devices
  4020h-5FFFh   Cartridge Expansion Area almost 8K
  6000h-7FFFh   Cartridge SRAM Area 8K
  8000h-FFFFh   Cartridge PRG-ROM Area 32K
*/
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
                    self.init_dma = true;
                    self.dma_start_addr = (data as u16) << 8;
                    return;
                }

                // JOYPAD 1
                if addr == 0x4016 {
                    self.joypad().borrow_mut().write(data);
                    return;
                }

                self.io_regs[(addr - 0x4000) as usize] = data;
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
