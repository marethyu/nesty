use crate::savable::Savable;

pub mod mapper0;
pub mod mapper1;

pub const PRG_ROM_BANK_SIZE: usize = 0x4000;
pub const CHR_ROM_BANK_SIZE: usize = 0x2000;

const SRAM_SIZE: usize = 0x2000;

#[derive(Copy, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontial
}

pub trait MapperBase {
    fn reset(&mut self);
    fn cpu_read_byte(&self, addr: u16) -> u8;
    fn cpu_write_byte(&mut self, addr: u16, data: u8);
    fn ppu_read_byte(&self, addr: u16) -> u8;
    fn ppu_write_byte(&mut self, addr: u16, data: u8);
    fn mirroring(&self) -> Mirroring;
}

pub trait Mapper: MapperBase + Savable {}
