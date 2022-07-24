use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::m6502::M6502;
use crate::ppu::PPU;
use crate::bus::Bus;

use crate::io::IO;

pub struct DMA {
    cpu: Weak<RefCell<M6502>>,
    ppu: Weak<RefCell<PPU>>,
    bus: Weak<RefCell<Bus>>,

    idle_cycles: u64,
    addr: u16,
    ntransferred: u32,

    pub active: bool
}

/* https://www.nesdev.org/wiki/PPU_OAM#DMA */
impl DMA {
    pub fn new(weak_cpu: Weak<RefCell<M6502>>,
               weak_ppu: Weak<RefCell<PPU>>,
               weak_bus: Weak<RefCell<Bus>>) -> Self {
        DMA {
            cpu: weak_cpu.clone(),
            ppu: weak_ppu.clone(),
            bus: weak_bus.clone(),

            idle_cycles: 0,
            addr: 0,
            ntransferred: 0,

            active: false
        }
    }

    pub fn cpu(&self) -> Rc<RefCell<M6502>> {
        self.cpu.upgrade().expect("CPU lost for dma")
    }

    pub fn ppu(&self) -> Rc<RefCell<PPU>> {
        self.ppu.upgrade().expect("PPU lost for dma")
    }

    pub fn bus(&self) -> Rc<RefCell<Bus>> {
        self.bus.upgrade().expect("Bus lost for dma")
    }

    pub fn init_transfer(&mut self, start_addr: u16) {
        self.active = true;

        self.idle_cycles = if self.cpu().borrow().total_cycles % 2 == 1 {
            2
        } else {
            1
        };

        self.addr = start_addr;
        self.ntransferred = 0;
    }

    pub fn do_transfer(&mut self) {
        if self.idle_cycles > 0 {
            self.cpu().borrow_mut().total_cycles += 1;
            self.idle_cycles -= 1;
            return;
        }

        let oam_data = self.bus().borrow_mut().read_byte(self.addr);
        self.ppu().borrow_mut().dma_write_oam(oam_data);

        self.addr += 1;
        self.ntransferred += 1;

        self.cpu().borrow_mut().total_cycles += 2;

        if self.ntransferred == 256 {
            self.active = false;
        }
    }
}
