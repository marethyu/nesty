mod bitops;
mod traits;
mod cpubus;
mod m6502;

use cpubus::CPUBus;
use m6502::M6502;

fn main() {
    let mut cpu_bus = CPUBus::new();
    let mut cpu = M6502::new(cpu_bus);

    loop {
        let cycles = cpu.step();
    }
}
