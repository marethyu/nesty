mod bitops;
mod traits;
mod m6502;
mod cpubus;

use std::rc::Rc;
use std::cell::RefCell;
use std::fs;

use m6502::M6502;
use cpubus::CPUBus;

fn load_cart_mem() -> [u8; 0xBFE0] {
    let mut mem = [0; 0xBFE0];
    let cart = fs::read("roms/nestest.nes").unwrap();

    for i in 0..0x4000 {
        mem[0xC000 - 0x4020 + i] = cart[i + 0x10];
    }

    mem
}

fn main() {
    let cpu = Rc::new(RefCell::new(M6502::new()));
    let cpu_bus = CPUBus::new(load_cart_mem());

    cpu.borrow_mut().load_bus(cpu_bus);

    loop {
        let cycles = cpu.borrow_mut().step();
    }
}
