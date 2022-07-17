use std::cell::{RefCell, RefMut};
use std::sync::Arc;

use crate::m6502::M6502;
use crate::ppu::PPU;
use crate::cartridge::Cartridge;
use crate::bus::Bus;
use crate::joypad::Joypad;

const CYCLES_PER_FRAME: u64 = 29781; // how many CPU cycles required to render one frame

pub struct Emulator {
    cart: Arc<RefCell<Cartridge>>,
    bus: Arc<RefCell<Bus>>, /* requirs access to cartridge, ppu, and joypad */
    cpu: Arc<RefCell<M6502>>, /* requires access to bus */
    ppu: Arc<RefCell<PPU>>, /* requires access to cartridge */
    joypad: Arc<RefCell<Joypad>>,

    prev_total_cycles: u64,
    penalty: u64 /* for dma timing */
}

impl Emulator {
    pub fn new(fname: &str) -> Self {
        let cart_arc = Arc::new(RefCell::new(Cartridge::new(fname)));
        let weak_cart = Arc::downgrade(&cart_arc);

        let ppu_arc = Arc::new(RefCell::new(PPU::new(weak_cart.clone())));
        let weak_ppu = Arc::downgrade(&ppu_arc);

        let joypad_arc = Arc::new(RefCell::new(Joypad::new()));
        let weak_joypad = Arc::downgrade(&joypad_arc);

        let bus_arc = Arc::new(RefCell::new(Bus::new(weak_cart.clone(), weak_ppu.clone(), weak_joypad.clone())));
        let weak_bus = Arc::downgrade(&bus_arc);

        let cpu_arc = Arc::new(RefCell::new(M6502::new(weak_bus.clone())));

        Emulator {
            cart: cart_arc,
            bus: bus_arc,
            cpu: cpu_arc,
            ppu: ppu_arc,
            joypad: joypad_arc,
            prev_total_cycles: 0,
            penalty: 0
        }
    }

    pub fn cart(&self) -> RefMut<'_, Cartridge> {
        self.cart.borrow_mut()
    }

    pub fn bus(&self) -> RefMut<'_, Bus> {
        self.bus.borrow_mut()
    }

    pub fn cpu(&self) -> RefMut<'_, M6502> {
        self.cpu.borrow_mut()
    }

    pub fn ppu(&self) -> RefMut<'_, PPU> {
        self.ppu.borrow_mut()
    }

    pub fn joypad(&self) -> RefMut<'_, Joypad> {
        self.joypad.borrow_mut()
    }

    pub fn reset(&mut self) {
        /* TODO reset cartridge as well */
        self.cpu().reset();
        self.ppu().reset();
    }

    pub fn update(&mut self) {
        let mut total: u64 = 0;

        while total < CYCLES_PER_FRAME {
            if self.ppu().nmi {
                self.cpu().nmi();
                self.ppu().nmi = false;
            }

            // TODO IRQ here

            if self.penalty > 0 {
                self.cpu().total_cycles += 1;
                self.penalty -= 1;
            } else {
                self.cpu().tick();
            }

            let total_cycles = self.cpu().total_cycles;
            let cycles = total_cycles - self.prev_total_cycles;
            self.prev_total_cycles = total_cycles;

            if self.bus().dma {
                self.penalty = if total_cycles % 2 == 1 { 514 } else { 513 };
                self.bus().dma = false;
            }

            for _i in 0..cycles {
                self.ppu().tick();
                self.ppu().tick();
                self.ppu().tick();
            }

            total += cycles;
        }
    }
}
