use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::mapper::{Mirroring, Mapper, MapperBase, SRAM_SIZE};

use crate::savable::Savable;
use crate::{test_bit, modify_bit, mirror, box_array};

pub struct Mapper1 {
    mirroring_type: Mirroring,

    prg_rom_banks: usize,
    chr_rom_banks: usize,

    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    sram: Box<[u8; SRAM_SIZE]>,

    /* 5 bit shift register */
    shift_register: u8,

    /* 5 bit control register
        4bit0
        -----
        CPPMM
        |||||
        |||++- Mirroring (0: one-screen, lower bank; 1: one-screen, upper bank;
        |||               2: vertical; 3: horizontal)
        |++--- PRG ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
        |                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
        |                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
        +----- CHR ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks) */
    control_register: u8,

    // chr bank select for 4Kb mode ($0000-$0FFF)
    chr_bank_select_lo: usize,
    // chr bank select for 4Kb mode ($1000-$1FFF)
    chr_bank_select_hi: usize,
    // chr bank select for 8Kb mode
    chr_bank_select: usize,

    // prg bank select for 16Kb mode ($8000-$BFFF)
    prg_bank_select_lo: usize,
    // prg bank select for 16Kb mode ($C000-$FFFF)
    prg_bank_select_hi: usize,
    // prg bank select for 32Kb mode
    prg_bank_select: usize
}

impl Mapper1 {
    pub fn new(mirroring_type: Mirroring, prg_rom_banks: usize, chr_rom_banks: usize, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Mapper1 {
            mirroring_type: mirroring_type,

            prg_rom_banks: prg_rom_banks,
            chr_rom_banks: chr_rom_banks,

            prg_rom: prg_rom,
            chr_rom: chr_rom,

            sram: box_array![0; SRAM_SIZE],

            shift_register: 0,
            control_register: 0,

            chr_bank_select_lo: 0,
            chr_bank_select_hi: 0,
            chr_bank_select: 0,

            prg_bank_select_lo: 0,
            prg_bank_select_hi: 0,
            prg_bank_select: 0
        }
    }

    fn modify_registers(&mut self, addr: u16) {
        match addr {
            /* Control (internal, $8000-$9FFF)  */
            0x8000..=0x9FFF => {
                let mirroring = self.shift_register & 0b00011;

                match mirroring {
                    2 => {
                        self.mirroring_type = Mirroring::Vertical;
                    }
                    3 => {
                        self.mirroring_type = Mirroring::Horizontial;
                    }
                    _ => panic!("This mirroring mode not supported: {}", mirroring)
                }

                self.control_register = self.shift_register;
            },
            /* CHR bank 0 (internal, $A000-$BFFF) */
            0xA000..=0xBFFF => {
                let mode8 = !test_bit!(self.control_register, 4);

                if mode8 {
                    self.chr_bank_select = (self.shift_register & 0b11110) as usize;
                } else {
                    self.chr_bank_select_lo = self.shift_register as usize;
                }
            },
            /* CHR bank 1 (internal, $C000-$DFFF) */
            0xC000..=0xDFFF => {
                let mode8 = !test_bit!(self.control_register, 4);

                if !mode8 {
                    self.chr_bank_select_hi = self.shift_register as usize;
                }

                // ignored in 8 KB mode
            },
            /* PRG bank (internal, $E000-$FFFF) */
            0xE000..=0xFFFF => {
                let mode = (self.control_register & 0b01100) >> 2;

                match mode {
                    0..=1 => {
                        self.prg_bank_select = ((self.shift_register as usize) & 0b01110) >> 1; // the last bit is ignored
                    }
                    2 => {
                        // fix *first* bank at $8000 and switch 16 KB bank at $C000
                        self.prg_bank_select_lo = 0;
                        self.prg_bank_select_hi = (self.shift_register as usize) & 0b01111;
                    }
                    3 => {
                        // fix *last* bank at $C000 and switch 16 KB bank at $8000
                        self.prg_bank_select_hi = self.prg_rom_banks - 1;
                        self.prg_bank_select_lo = (self.shift_register as usize) & 0b01111;
                    }
                    _ => {}
                }
            }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn reset_shift_reg(&mut self) {
        self.shift_register = 0b10000; // 1 is used to determine whether it is full
    }
}

/* https://www.nesdev.org/wiki/MMC1 */
impl MapperBase for Mapper1 {
    fn reset(&mut self) {
        self.shift_register = 0;
        self.control_register = 0x1C;

        self.chr_bank_select_lo = 0;
        self.chr_bank_select_hi = 0;
        self.chr_bank_select = 0;

        self.prg_bank_select_lo = 0;
        self.prg_bank_select_hi = self.prg_rom_banks - 1;
        self.prg_bank_select = 0;
    }

    fn cpu_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.sram[mirror!(0x6000, addr, SRAM_SIZE)],
            0x8000..=0xFFFF => {
                let mode = (self.control_register & 0b01100) >> 2;

                if mode == 0 || mode == 1 {
                    return self.prg_rom[self.prg_bank_select * 0x8000 + ((addr as usize) & 0x7FFF)];
                }

                if addr < 0xC000 {
                    return self.prg_rom[self.prg_bank_select_lo * 0x4000 + ((addr as usize) & 0x3FFF)];
                } else {
                    return self.prg_rom[self.prg_bank_select_hi * 0x4000 + ((addr as usize) & 0x3FFF)];
                }
            },
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn cpu_write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.sram[mirror!(0x6000, addr, SRAM_SIZE)] = data;
            }
            0x8000..=0xFFFF => {
                /*  'data' is known as a load register here
                    7  bit  0
                    ---- ----
                    Rxxx xxxD
                    |       |
                    |       +- Data bit to be shifted into shift register, LSB first
                    +--------- 1: Reset shift register and write Control with (Control OR $0C),
                    locking PRG ROM at $C000-$FFFF to the last bank. */
                if test_bit!(data, 7) {
                    self.reset_shift_reg();
                    self.control_register |= 0x0C;
                    self.prg_bank_select_hi = self.prg_rom_banks - 1;
                } else {
                    // Once a 1 is shifted into the last position, the SR is full.
                    let is_full = test_bit!(self.shift_register, 0);

                    // Store the LSB of data into shift register's MSB.
                    self.shift_register >>= 1;
                    modify_bit!(self.shift_register, 4, test_bit!(data, 0));

                    if is_full {
                        self.modify_registers(addr);
                        self.reset_shift_reg();
                    }
                }
            }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn ppu_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom_banks == 0 {
                    // chr ram read
                    return self.chr_rom[addr as usize];
                }

                let mode8 = !test_bit!(self.control_register, 4);

                if !mode8 {
                    if addr < 0x1000 {
                        return self.chr_rom[self.chr_bank_select_lo * 0x1000 + (addr as usize)];
                    } else {
                        return self.chr_rom[self.chr_bank_select_hi * 0x1000 + ((addr as usize) & 0x0FFF)];
                    }
                }

                return self.chr_rom[self.chr_bank_select * 0x2000 + (addr as usize)];
            },
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn ppu_write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom_banks == 0 {
                    // chr ram write
                    self.chr_rom[addr as usize] = data;
                }
            }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn mirroring(&self) -> Mirroring {
        return self.mirroring_type;
    }
}

impl Savable for Mapper1 {
    fn save_state(&self, state: &mut Vec<u8>) {
        match self.mirroring_type {
            Mirroring::Vertical => {
                state.write_u8(0).expect("Unable to save u8");
            }
            Mirroring::Horizontial => {
                state.write_u8(1).expect("Unable to save u8");
            }
        }

        let use_chr_ram = self.chr_rom_banks == 0;

        state.write_u8(use_chr_ram as u8).expect("Unable to save u8");
        if use_chr_ram {
            for i in 0..self.chr_rom.len() {
                state.write_u8(self.chr_rom[i]).expect("Unable to save u8");
            }
        }

        for i in 0..SRAM_SIZE {
            state.write_u8(self.sram[i]).expect("Unable to save u8");
        }

        state.write_u8(self.shift_register).expect("Unable to save u8");
        state.write_u8(self.control_register).expect("Unable to save u8");

        state.write_u32::<LittleEndian>(self.chr_bank_select_lo as u32).expect("Unable to save u32");
        state.write_u32::<LittleEndian>(self.chr_bank_select_hi as u32).expect("Unable to save u32");
        state.write_u32::<LittleEndian>(self.chr_bank_select as u32).expect("Unable to save u32");

        state.write_u32::<LittleEndian>(self.prg_bank_select_lo as u32).expect("Unable to save u32");
        state.write_u32::<LittleEndian>(self.prg_bank_select_hi as u32).expect("Unable to save u32");
        state.write_u32::<LittleEndian>(self.prg_bank_select as u32).expect("Unable to save u32");
    }

    fn load_state(&mut self, state: &mut Cursor<Vec<u8>>) {
        let mirroring = state.read_u8().expect("Unable to load u8");
        match mirroring {
            0 => {
                self.mirroring_type = Mirroring::Vertical;
            }
            1 => {
                self.mirroring_type = Mirroring::Horizontial;
            }
            _ => panic!("Unknown byte when reading mirroring configuration: {}", mirroring)
        }

        let use_chr_ram = state.read_u8().expect("Unable to load u8") != 0;
        if use_chr_ram {
            for i in 0..self.chr_rom.len() {
                self.chr_rom[i] = state.read_u8().expect("Unable to load u8");
            }
        }

        for i in 0..SRAM_SIZE {
            self.sram[i] = state.read_u8().expect("Unable to load u8");
        }

        self.shift_register = state.read_u8().expect("Unable to load u8");
        self.control_register = state.read_u8().expect("Unable to load u8");

        self.chr_bank_select_lo = state.read_u32::<LittleEndian>().expect("Unable to load u32") as usize;
        self.chr_bank_select_hi = state.read_u32::<LittleEndian>().expect("Unable to load u32") as usize;
        self.chr_bank_select = state.read_u32::<LittleEndian>().expect("Unable to load u32") as usize;

        self.prg_bank_select_lo = state.read_u32::<LittleEndian>().expect("Unable to load u32") as usize;
        self.prg_bank_select_hi = state.read_u32::<LittleEndian>().expect("Unable to load u32") as usize;
        self.prg_bank_select = state.read_u32::<LittleEndian>().expect("Unable to load u32") as usize;
    }
}

impl Mapper for Mapper1 {}
