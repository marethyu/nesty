use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::m6502::M6502;
use crate::ppu::PPU;
use crate::dma::DMA;
use crate::cartridge::Cartridge;
use crate::bus::Bus;
use crate::joypad::Joypad;

use crate::savable::Savable;

pub const CYCLES_PER_FRAME: u64 = 29781; // how many CPU cycles required to render one frame

pub struct Emulator {
    cart: Rc<RefCell<Cartridge>>,
    bus: Rc<RefCell<Bus>>, /* requirs access to cartridge, ppu, and joypad */
    cpu: Rc<RefCell<M6502>>, /* requires access to bus */
    ppu: Rc<RefCell<PPU>>, /* requires access to cartridge */
    dma: Rc<RefCell<DMA>>, /* requires access to cpu, ppu, and bus */
    joypad: Rc<RefCell<Joypad>>,

    prev_total_cycles: u64
}

impl Emulator {
    pub fn new() -> Self {
        let cart_ref = Rc::new(RefCell::new(Cartridge::new()));
        let weak_cart = Rc::downgrade(&cart_ref);

        let ppu_ref = Rc::new(RefCell::new(PPU::new(weak_cart.clone())));
        let weak_ppu = Rc::downgrade(&ppu_ref);

        let joypad_ref = Rc::new(RefCell::new(Joypad::new()));
        let weak_joypad = Rc::downgrade(&joypad_ref);

        let bus_ref = Rc::new(RefCell::new(Bus::new(
            weak_cart.clone(),
            weak_ppu.clone(),
            weak_joypad.clone()
        )));
        let weak_bus = Rc::downgrade(&bus_ref);

        let cpu_ref = Rc::new(RefCell::new(M6502::new(weak_bus.clone())));
        let weak_cpu = Rc::downgrade(&cpu_ref);

        let dma_ref = Rc::new(RefCell::new(DMA::new(
            weak_cpu.clone(),
            weak_ppu.clone(),
            weak_bus.clone())
        ));

        Emulator {
            cart: cart_ref,
            bus: bus_ref,
            cpu: cpu_ref,
            ppu: ppu_ref,
            dma: dma_ref,
            joypad: joypad_ref,

            prev_total_cycles: 0
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

    pub fn dma(&self) -> RefMut<'_, DMA> {
        self.dma.borrow_mut()
    }

    pub fn joypad(&self) -> RefMut<'_, Joypad> {
        self.joypad.borrow_mut()
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) -> Result<String, String> {
        return self.cart().load(rom);
    }

    pub fn reset(&mut self) {
        self.cart().reset();
        self.bus().reset();
        self.cpu().reset();
        self.ppu().reset();
        self.joypad().reset();
    }

    pub fn tick(&mut self) -> u64 {
        if self.ppu().nmi {
            self.cpu().nmi();
            self.ppu().nmi = false;
        }

        // TODO IRQ here

        if self.dma().active {
            // cpu is stalled during dma transfer
            self.dma().do_transfer();
        } else {
            self.cpu().tick();

            if self.bus().init_dma {
                self.dma().init_transfer(self.bus().dma_start_addr);
                self.bus().init_dma = false;
            }
        }

        let total_cycles = self.cpu().total_cycles;
        let cycles = total_cycles - self.prev_total_cycles;
        self.prev_total_cycles = total_cycles;

        for _i in 0..cycles {
            self.ppu().tick();
            self.ppu().tick();
            self.ppu().tick();
        }

        cycles
    }

    pub fn update(&mut self) {
        let mut total: u64 = 0;

        while total < CYCLES_PER_FRAME {
            total += self.tick();
        }
    }
}

impl Savable for Emulator {
    fn save_state(&self, state: &mut Vec<u8>) {
        self.cart().save_state(state);
        self.bus().save_state(state);
        self.cpu().save_state(state);
        self.ppu().save_state(state);
        self.joypad().save_state(state);
        state.write_u64::<LittleEndian>(self.prev_total_cycles).expect("Unable to save u64");
    }

    fn load_state(&mut self, state: &mut Cursor<Vec<u8>>) {
        self.cart().load_state(state);
        self.bus().load_state(state);
        self.cpu().load_state(state);
        self.ppu().load_state(state);
        self.joypad().load_state(state);
        self.prev_total_cycles = state.read_u64::<LittleEndian>().expect("Unable to load u64");
    }
}
