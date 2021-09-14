use crate::traits::IO;
use crate::cpubus::CPUBus;

use crate::{test_bit, modify_bit};

const FLAG_N: u8 = 7;
const FLAG_V: u8 = 6;
const FLAG_U: u8 = 5; // unused
const FLAG_B: u8 = 4;
const FLAG_D: u8 = 3;
const FLAG_I: u8 = 2;
const FLAG_Z: u8 = 1;
const FLAG_C: u8 = 0;

enum AddressingMode {
//  Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteXEc, /* AbsoluteX with an extra cycle */
    AbsoluteY,
    AbsoluteYEc, /* AbsoluteY with an extra cycle */
    Immediate,
//  Implied,
    Indirect,
    IndirectX,
    IndirectY,
    IndirectYEc, /* IndirectY with an extra cycle */
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY
}

pub struct M6502 {
    a:      u8,
    x:      u8,
    y:      u8,
    p:      u8,
    sp:     u8,
    pc:     u16,
    bus:    Option<CPUBus>,
    cycles: u32,
    total_cycles: u32
}

macro_rules! page_cross {
    ($addr1:expr, $addr2:expr) => {
        ($addr1 & 0xFF00) != ($addr2 & 0xFF00)
    }
}

impl M6502 {
    pub fn new() -> Self {
        M6502 {
            a:      0,
            x:      0,
            y:      0,
            p:      0x24,
            sp:     0xFD,
            pc:     0xC000,
            bus:    None,
            cycles: 0,
            total_cycles: 7
        }
    }

    pub fn load_bus(&mut self, bus: CPUBus) {
        self.bus = Some(bus);
    }

    pub fn step(&mut self) -> u32 {
        self.cycles = 0;

        println!("{:04X}  {:02X}\t\t\t\t\t\t\t\t\t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}\t\t\t  CYC:{}", self.pc, self.bus.as_ref().unwrap().read_byte(self.pc), self.a, self.x, self.y, self.p, self.sp, self.total_cycles);

        macro_rules! do_add {
            /*  for idiots who have no idea how to determine overflow,
                please read http://users.telenet.be/kim1-6502/6502/proman.html#362 */
            ($nn:expr) => {
                let val = $nn;
                let carry = test_bit!(self.p, FLAG_C) as u8;
                let signbits_same1 = test_bit!(self.a ^ val, 7);
                let (res, cf) = self.a.overflowing_add(val + carry);
                let signbits_same2 = test_bit!(self.a ^ res, 7);
                self.a = res;
                self.modify_zn(self.a);
                modify_bit!(self.p, FLAG_V, !signbits_same1 && signbits_same2);
                modify_bit!(self.p, FLAG_C, cf);
            }
        }

        macro_rules! adc {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                do_add!(self.cpu_read_byte(addr));
            }
        }

        macro_rules! sbc {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                do_add!(self.cpu_read_byte(addr) ^ 0xFF);
            }
        }

        macro_rules! and {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                self.a &= self.cpu_read_byte(addr);
                self.modify_zn(self.a);
            }
        }

        macro_rules! branch {
            ($cond:expr, $test:literal) => {
                let new_pc = self.fetch_address(AddressingMode::Relative);
                if $cond == $test {
                    if page_cross!(self.pc, new_pc) {
                        self.cycles += 2;
                    } else {
                        self.cycles += 1;
                    }
                    self.pc = new_pc;
                }
            }
        }

        macro_rules! bit {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                let m = self.cpu_read_byte(addr);
                modify_bit!(self.p, FLAG_Z, (self.a & m) == 0);
                modify_bit!(self.p, FLAG_V, test_bit!(m, 6));
                modify_bit!(self.p, FLAG_N, test_bit!(m, 7));
            }
        }

        macro_rules! compare {
            ($reg:expr, $mode:expr) => {
                let addr = self.fetch_address($mode);
                let m = self.cpu_read_byte(addr);
                modify_bit!(self.p, FLAG_C, $reg >= m);
                modify_bit!(self.p, FLAG_Z, $reg == m);
                modify_bit!(self.p, FLAG_N, test_bit!($reg.wrapping_sub(m), 7));
            }
        }

        macro_rules! eor {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                self.a ^= self.cpu_read_byte(addr);
                self.modify_zn(self.a);
            }
        }

        macro_rules! inc_mem {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                let mut val = self.cpu_read_byte(addr);
                val = val.wrapping_add(1);
                self.modify_zn(val);
                self.cycles += 1; // dummy write
                self.cpu_write_byte(addr, val);
            }
        }

        macro_rules! dec_mem {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                let mut val = self.cpu_read_byte(addr);
                val = val.wrapping_sub(1);
                self.modify_zn(val);
                self.cycles += 1; // dummy write
                self.cpu_write_byte(addr, val);
            }
        }

        macro_rules! inc_reg {
            ($reg:expr) => {
                $reg = $reg.wrapping_add(1);
                self.modify_zn($reg);
            }
        }

        macro_rules! dec_reg {
            ($reg:expr) => {
                $reg = $reg.wrapping_sub(1);
                self.modify_zn($reg);
            }
        }

        macro_rules! jump {
            ($mode:expr) => {
                self.pc = self.fetch_address($mode);
            }
        }

        macro_rules! load {
            ($reg:expr, $mode:expr) => {
                let addr = self.fetch_address($mode);
                $reg = self.cpu_read_byte(addr);
                self.modify_zn($reg);
            }
        }

        macro_rules! ora {
            ($mode:expr) => {
                let addr = self.fetch_address($mode);
                self.a |= self.cpu_read_byte(addr);
                self.modify_zn(self.a);
            }
        }

        macro_rules! pull_p {
            () => {
                self.p = (self.pull_byte() & 0b11001111) | (self.p & 0b00110000);
            }
        }

        macro_rules! shift_reg {
            ($reg:expr, $rl:literal, $logical:literal) => {
                if $rl {
                    let new_cfv = test_bit!($reg, 7);
                    $reg <<= 1;
                    if !$logical {
                        modify_bit!($reg, 0, test_bit!(self.p, FLAG_C));
                    }
                    modify_bit!(self.p, FLAG_C, new_cfv);
                } else {
                    let new_cfv = test_bit!($reg, 0);
                    $reg >>= 1;
                    if !$logical {
                        modify_bit!($reg, 7, test_bit!(self.p, FLAG_C));
                    }
                    modify_bit!(self.p, FLAG_C, new_cfv);
                }
                self.modify_zn($reg);
            }
        }

        macro_rules! shift_mem {
            ($mode:expr, $rl:literal, $logical:literal) => {
                let addr = self.fetch_address($mode);
                let mut val = self.cpu_read_byte(addr);
                if $rl {
                    let new_cfv = test_bit!(val, 7);
                    val <<= 1;
                    if !$logical {
                        modify_bit!(val, 0, test_bit!(self.p, FLAG_C));
                    }
                    modify_bit!(self.p, FLAG_C, new_cfv);
                } else {
                    let new_cfv = test_bit!(val, 0);
                    val >>= 1;
                    if !$logical {
                        modify_bit!(val, 7, test_bit!(self.p, FLAG_C));
                    }
                    modify_bit!(self.p, FLAG_C, new_cfv);
                }
                self.modify_zn(val);
                self.cycles += 1; // dummy write
                self.cpu_write_byte(addr, val);
            }
        }

        macro_rules! stk_adjust_pl_cycles {
            () => {
                self.cycles += 2;
            }
        }

        macro_rules! stk_adjust_ph_cycles {
            () => {
                self.cycles += 1;
            }
        }

        macro_rules! store {
            ($reg:expr, $mode:expr) => {
                let addr = self.fetch_address($mode);
                self.cpu_write_byte(addr, $reg);
            }
        }

        macro_rules! transfer {
            ($reg1:expr, $reg2:expr) => {
                $reg2 = $reg1;
                self.modify_zn($reg2);
            }
        }

        let opcode = self.fetch_byte();

        match opcode {
            0x69 => { /* ADC #oper; 2c */       adc!(AddressingMode::Immediate); }
            0x65 => { /* ADC oper; 3c */        adc!(AddressingMode::ZeroPage); }
            0x75 => { /* ADC oper,X; 4c */      adc!(AddressingMode::ZeroPageX); }
            0x6D => { /* ADC oper; 4c */        adc!(AddressingMode::Absolute); }
            0x7D => { /* ADC oper,X; 4+c */     adc!(AddressingMode::AbsoluteX); }
            0x79 => { /* ADC oper,Y; 4+c */     adc!(AddressingMode::AbsoluteY); }
            0x61 => { /* ADC (oper,X); 6c */    adc!(AddressingMode::IndirectX); }
            0x71 => { /* ADC (oper),Y; 5+c */   adc!(AddressingMode::IndirectY); }
            0x29 => { /* AND #oper; 2c */       and!(AddressingMode::Immediate); }
            0x25 => { /* AND oper; 3c */        and!(AddressingMode::ZeroPage); }
            0x35 => { /* AND oper,X; 4c */      and!(AddressingMode::ZeroPageX); }
            0x2D => { /* AND oper; 4c */        and!(AddressingMode::Absolute); }
            0x3D => { /* AND oper,X; 4+c */     and!(AddressingMode::AbsoluteX); }
            0x39 => { /* AND oper,Y; 4+c */     and!(AddressingMode::AbsoluteY); }
            0x21 => { /* AND (oper,X); 6c */    and!(AddressingMode::IndirectX); }
            0x31 => { /* AND (oper),Y; 5+c */   and!(AddressingMode::IndirectY); }
            0x0A => { /* ASL A; 2c */           shift_reg!(self.a, true, true); }
            0x06 => { /* ASL oper; 5c */        shift_mem!(AddressingMode::ZeroPage, true, true); }
            0x16 => { /* ASL oper,X; 6c */      shift_mem!(AddressingMode::ZeroPageX, true, true); }
            0x0E => { /* ASL oper; 6c */        shift_mem!(AddressingMode::Absolute, true, true); }
            0x1E => { /* ASL oper,X; 7c */      shift_mem!(AddressingMode::AbsoluteXEc, true, true); }
            0x90 => { /* BCC oper; 2++c */      branch!(test_bit!(self.p, FLAG_C), false); }
            0xB0 => { /* BCS oper; 2++c */      branch!(test_bit!(self.p, FLAG_C), true); }
            0xF0 => { /* BEQ oper; 2++c */      branch!(test_bit!(self.p, FLAG_Z), true); }
            0x24 => { /* BIT oper; 3c */        bit!(AddressingMode::ZeroPage); }
            0x2C => { /* BIT oper; 4c */        bit!(AddressingMode::Absolute); }
            0x30 => { /* BMI oper; 2++c */      branch!(test_bit!(self.p, FLAG_N), true); }
            0xD0 => { /* BNE oper; 2++c */      branch!(test_bit!(self.p, FLAG_Z), false); }
            0x10 => { /* BPL oper; 2++c */      branch!(test_bit!(self.p, FLAG_N), false); }
            0x00 => { /* BRK; 7c */             todo!("BRK not implemented yet!"); }
            0x50 => { /* BVC oper; 2++c */      branch!(test_bit!(self.p, FLAG_V), false); }
            0x70 => { /* BVS oper; 2++c */      branch!(test_bit!(self.p, FLAG_V), true); }
            0x18 => { /* CLC; 2c */             modify_bit!(self.p, FLAG_C, false); }
            0xD8 => { /* CLD; 2c */             modify_bit!(self.p, FLAG_D, false); }
            0x58 => { /* CLI; 2c */             modify_bit!(self.p, FLAG_I, false); }
            0xB8 => { /* CLV; 2c */             modify_bit!(self.p, FLAG_V, false); }
            0xC9 => { /* CMP #oper; 2c */       compare!(self.a, AddressingMode::Immediate); }
            0xC5 => { /* CMP oper; 3c */        compare!(self.a, AddressingMode::ZeroPage); }
            0xD5 => { /* CMP oper,X; 4c */      compare!(self.a, AddressingMode::ZeroPageX); }
            0xCD => { /* CMP oper; 4c */        compare!(self.a, AddressingMode::Absolute); }
            0xDD => { /* CMP oper,X; 4+c */     compare!(self.a, AddressingMode::AbsoluteX); }
            0xD9 => { /* CMP oper,Y; 4+c */     compare!(self.a, AddressingMode::AbsoluteY); }
            0xC1 => { /* CMP (oper,X); 6c */    compare!(self.a, AddressingMode::IndirectX); }
            0xD1 => { /* CMP (oper),Y; 5+c */   compare!(self.a, AddressingMode::IndirectY); }
            0xE0 => { /* CPX #oper; 2c */       compare!(self.x, AddressingMode::Immediate); }
            0xE4 => { /* CPX oper; 3c */        compare!(self.x, AddressingMode::ZeroPage); }
            0xEC => { /* CPX oper; 4c */        compare!(self.x, AddressingMode::Absolute); }
            0xC0 => { /* CPY #oper; 2c */       compare!(self.y, AddressingMode::Immediate); }
            0xC4 => { /* CPY oper; 3c */        compare!(self.y, AddressingMode::ZeroPage); }
            0xCC => { /* CPY oper; 4c */        compare!(self.y, AddressingMode::Absolute); }
            0xC6 => { /* DEC oper; 5c */        dec_mem!(AddressingMode::ZeroPage); }
            0xD6 => { /* DEC oper,X; 6c */      dec_mem!(AddressingMode::ZeroPageX); }
            0xCE => { /* DEC oper; 6c */        dec_mem!(AddressingMode::Absolute); }
            0xDE => { /* DEC oper,X; 7c */      dec_mem!(AddressingMode::AbsoluteXEc); }
            0xCA => { /* DEX; 2c */             dec_reg!(self.x); }
            0x88 => { /* DEY; 2c */             dec_reg!(self.y); }
            0x49 => { /* EOR #oper; 2c */       eor!(AddressingMode::Immediate); }
            0x45 => { /* EOR oper; 3c */        eor!(AddressingMode::ZeroPage); }
            0x55 => { /* EOR oper,X; 4c */      eor!(AddressingMode::ZeroPageX); }
            0x4D => { /* EOR oper; 4c */        eor!(AddressingMode::Absolute); }
            0x5D => { /* EOR oper,X; 4+c */     eor!(AddressingMode::AbsoluteX); }
            0x59 => { /* EOR oper,Y; 4+c */     eor!(AddressingMode::AbsoluteY); }
            0x41 => { /* EOR (oper,X); 6c */    eor!(AddressingMode::IndirectX); }
            0x51 => { /* EOR (oper),Y; 5+c */   eor!(AddressingMode::IndirectY); }
            0xE6 => { /* INC oper; 5c */        inc_mem!(AddressingMode::ZeroPage); }
            0xF6 => { /* INC oper,X; 6c */      inc_mem!(AddressingMode::ZeroPageX); }
            0xEE => { /* INC oper; 6c */        inc_mem!(AddressingMode::Absolute); }
            0xFE => { /* INC oper,X; 7c */      inc_mem!(AddressingMode::AbsoluteXEc); }
            0xE8 => { /* INX; 2c */             inc_reg!(self.x); }
            0xC8 => { /* INY; 2c */             inc_reg!(self.y); }
            0x4C => { /* JMP oper; 3c */        jump!(AddressingMode::Absolute); }
            0x6C => { /* JMP (oper); 5c */      jump!(AddressingMode::Indirect); }
            0x20 => { /* JSR oper; 6c */        self.push_word(self.pc + 1); jump!(AddressingMode::Absolute); stk_adjust_ph_cycles!(); }
            0xA9 => { /* LDA #oper; 2c */       load!(self.a, AddressingMode::Immediate); }
            0xA5 => { /* LDA oper; 3c */        load!(self.a, AddressingMode::ZeroPage); }
            0xB5 => { /* LDA oper,X; 4c */      load!(self.a, AddressingMode::ZeroPageX); }
            0xAD => { /* LDA oper; 4c */        load!(self.a, AddressingMode::Absolute); }
            0xBD => { /* LDA oper,X; 4+c */     load!(self.a, AddressingMode::AbsoluteX); }
            0xB9 => { /* LDA oper,Y; 4+c */     load!(self.a, AddressingMode::AbsoluteY); }
            0xA1 => { /* LDA (oper,X); 6c */    load!(self.a, AddressingMode::IndirectX); }
            0xB1 => { /* LDA (oper),Y; 5+c */   load!(self.a, AddressingMode::IndirectY); }
            0xA2 => { /* LDX #oper; 2c */       load!(self.x, AddressingMode::Immediate); }
            0xA6 => { /* LDX oper; 3c */        load!(self.x, AddressingMode::ZeroPage); }
            0xB6 => { /* LDX oper,Y; 4c */      load!(self.x, AddressingMode::ZeroPageY); }
            0xAE => { /* LDX oper; 4c */        load!(self.x, AddressingMode::Absolute); }
            0xBE => { /* LDX oper,Y; 4+c */     load!(self.x, AddressingMode::AbsoluteY); }
            0xA0 => { /* LDY #oper; 2c */       load!(self.y, AddressingMode::Immediate); }
            0xA4 => { /* LDY oper; 3c */        load!(self.y, AddressingMode::ZeroPage); }
            0xB4 => { /* LDY oper,X; 4c */      load!(self.y, AddressingMode::ZeroPageX); }
            0xAC => { /* LDY oper; 4c */        load!(self.y, AddressingMode::Absolute); }
            0xBC => { /* LDY oper,X; 4+c */     load!(self.y, AddressingMode::AbsoluteX); }
            0x4A => { /* LSR A; 2c */           shift_reg!(self.a, false, true); }
            0x46 => { /* LSR oper; 5c */        shift_mem!(AddressingMode::ZeroPage, false, true); }
            0x56 => { /* LSR oper,X; 6c */      shift_mem!(AddressingMode::ZeroPageX, false, true); }
            0x4E => { /* LSR oper; 6c */        shift_mem!(AddressingMode::Absolute, false, true); }
            0x5E => { /* LSR oper,X; 7c */      shift_mem!(AddressingMode::AbsoluteXEc, false, true); }
            0xEA => { /* NOP; 2c */ }
            0x09 => { /* ORA #oper; 2c */       ora!(AddressingMode::Immediate); }
            0x05 => { /* ORA oper; 3c */        ora!(AddressingMode::ZeroPage); }
            0x15 => { /* ORA oper,X; 4c */      ora!(AddressingMode::ZeroPageX); }
            0x0D => { /* ORA oper; 4c */        ora!(AddressingMode::Absolute); }
            0x1D => { /* ORA oper,X; 4+c */     ora!(AddressingMode::AbsoluteX); }
            0x19 => { /* ORA oper,Y; 4+c */     ora!(AddressingMode::AbsoluteY); }
            0x01 => { /* ORA (oper,X); 6c */    ora!(AddressingMode::IndirectX); }
            0x11 => { /* ORA (oper),Y; 5+c */   ora!(AddressingMode::IndirectY); }
            0x48 => { /* PHA; 3c */             self.push_byte(self.a); stk_adjust_ph_cycles!(); }
            0x08 => { /* PHP; 3c */             self.push_byte(self.p | 0b00110000); stk_adjust_ph_cycles!(); }
            0x68 => { /* PLA; 4c */             self.a = self.pull_byte(); self.modify_zn(self.a); stk_adjust_pl_cycles!(); }
            0x28 => { /* PLP; 4c */             pull_p!(); stk_adjust_pl_cycles!(); }
            0x2A => { /* ROL A; 2c */           shift_reg!(self.a, true, false); }
            0x26 => { /* ROL oper; 5c */        shift_mem!(AddressingMode::ZeroPage, true, false); }
            0x36 => { /* ROL oper,X; 6c */      shift_mem!(AddressingMode::ZeroPageX, true, false); }
            0x2E => { /* ROL oper; 6c */        shift_mem!(AddressingMode::Absolute, true, false); }
            0x3E => { /* ROL oper,X; 7c */      shift_mem!(AddressingMode::AbsoluteXEc, true, false); }
            0x6A => { /* ROR A; 2c */           shift_reg!(self.a, false, false); }
            0x66 => { /* ROR oper; 5c */        shift_mem!(AddressingMode::ZeroPage, false, false); }
            0x76 => { /* ROR oper,X; 6c */      shift_mem!(AddressingMode::ZeroPageX, false, false); }
            0x6E => { /* ROR oper; 6c */        shift_mem!(AddressingMode::Absolute, false, false); }
            0x7E => { /* ROR oper,X; 7c */      shift_mem!(AddressingMode::AbsoluteXEc, false, false); }
            0x40 => { /* RTI; 6c */             pull_p!(); self.pc = self.pull_word(); stk_adjust_pl_cycles!(); }
            0x60 => { /* RTS; 6c */             self.pc = self.pull_word() + 1; stk_adjust_pl_cycles!(); self.cycles += 1; }
            0xE9 => { /* SBC #oper; 2c */       sbc!(AddressingMode::Immediate); }
            0xE5 => { /* SBC oper; 3c */        sbc!(AddressingMode::ZeroPage); }
            0xF5 => { /* SBC oper,X; 4c */      sbc!(AddressingMode::ZeroPageX); }
            0xED => { /* SBC oper; 4c */        sbc!(AddressingMode::Absolute); }
            0xFD => { /* SBC oper,X; 4+c */     sbc!(AddressingMode::AbsoluteX); }
            0xF9 => { /* SBC oper,Y; 4+c */     sbc!(AddressingMode::AbsoluteY); }
            0xE1 => { /* SBC (oper,X); 6c */    sbc!(AddressingMode::IndirectX); }
            0xF1 => { /* SBC (oper),Y; 5+c */   sbc!(AddressingMode::IndirectY); }
            0x38 => { /* SEC; 2c */             modify_bit!(self.p, FLAG_C, true); }
            0xF8 => { /* SED; 2c */             modify_bit!(self.p, FLAG_D, true); }
            0x78 => { /* SEI; 2c */             modify_bit!(self.p, FLAG_I, true); }
            0x85 => { /* STA oper; 3c */        store!(self.a, AddressingMode::ZeroPage); }
            0x95 => { /* STA oper,X; 4c */      store!(self.a, AddressingMode::ZeroPageX); }
            0x8D => { /* STA oper; 4c */        store!(self.a, AddressingMode::Absolute); }
            0x9D => { /* STA oper,X; 5c */      store!(self.a, AddressingMode::AbsoluteXEc); }
            0x99 => { /* STA oper,Y; 5c */      store!(self.a, AddressingMode::AbsoluteYEc); }
            0x81 => { /* STA (oper,X); 6c */    store!(self.a, AddressingMode::IndirectX); }
            0x91 => { /* STA (oper),Y; 6c */    store!(self.a, AddressingMode::IndirectYEc); }
            0x86 => { /* STX oper; 3c */        store!(self.x, AddressingMode::ZeroPage); }
            0x96 => { /* STX oper,Y; 4c */      store!(self.x, AddressingMode::ZeroPageY); }
            0x8E => { /* STX oper; 4c */        store!(self.x, AddressingMode::Absolute); }
            0x84 => { /* STY oper; 3c */        store!(self.y, AddressingMode::ZeroPage); }
            0x94 => { /* STY oper,X; 4c */      store!(self.y, AddressingMode::ZeroPageX); }
            0x8C => { /* STY oper; 4c */        store!(self.y, AddressingMode::Absolute); }
            0xAA => { /* TAX; 2c */             transfer!(self.a, self.x); }
            0xA8 => { /* TAY; 2c */             transfer!(self.a, self.y); }
            0xBA => { /* TSX; 2c */             transfer!(self.sp, self.x); }
            0x8A => { /* TXA; 2c */             transfer!(self.x, self.a); }
            0x9A => { /* TXS; 2c */             self.sp = self.x; }
            0x98 => { /* TYA; 2c */             transfer!(self.y, self.a); }
            _ => todo!("Halted at PC={:04X}; Unimplemented opcode: {:02X}", self.pc - 1, opcode)
        }

        if self.cycles < 2 {
            self.cycles += 1;
        }
        self.total_cycles += self.cycles;

        self.cycles
    }

    fn pull_word(&mut self) -> u16 {
        let lo = self.pull_byte() as u16;
        let hi = self.pull_byte() as u16;
        (hi << 8) | lo
    }

    fn pull_byte(&mut self) -> u8 {
        self.sp += 1;
        self.cpu_read_byte((self.sp as u16) | 0x100)
    }

    fn push_word(&mut self, data: u16) {
        self.push_byte((data >> 8) as u8);
        self.push_byte((data & 0xFF) as u8);
    }

    fn push_byte(&mut self, data: u8) {
        self.cpu_write_byte((self.sp as u16) | 0x100, data);
        self.sp -= 1;
    }

    fn modify_zn(&mut self, nn: u8) {
        modify_bit!(self.p, FLAG_Z, nn == 0);
        modify_bit!(self.p, FLAG_N, test_bit!(nn, 7));
    }

    fn fetch_address(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Absolute => self.fetch_word(),
            AddressingMode::AbsoluteX => {
                let addr: u16 = self.fetch_word();
                let eff_addr: u16 = addr.wrapping_add(self.x as u16);
                if page_cross!(addr, eff_addr) {
                    self.cycles += 1;
                }
                eff_addr
            },
            AddressingMode::AbsoluteXEc => {
                let addr: u16 = self.fetch_word();
                let eff_addr: u16 = addr.wrapping_add(self.x as u16);
                self.cycles += 1;
                eff_addr
            },
            AddressingMode::AbsoluteY => {
                let addr: u16 = self.fetch_word();
                let eff_addr: u16 = addr.wrapping_add(self.y as u16);
                if page_cross!(addr, eff_addr) {
                    self.cycles += 1;
                }
                eff_addr
            },
            AddressingMode::AbsoluteYEc => {
                let addr: u16 = self.fetch_word();
                let eff_addr: u16 = addr.wrapping_add(self.y as u16);
                self.cycles += 1;
                eff_addr
            },
            AddressingMode::Immediate => {
                self.pc += 1;
                self.pc - 1
            },
            AddressingMode::Indirect => {
                /* INDIRECT JUMP CAN'T CROSS PAGES!! */
                let nnnn = self.fetch_word();
                let lo = self.cpu_read_byte(nnnn) as u16;
                let hi = self.cpu_read_byte((nnnn & 0xFF00) | ((nnnn + 1) & 0xFF)) as u16;
                (hi << 8) | lo
            },
            AddressingMode::IndirectX => {
                let nn = self.fetch_byte();
                self.cycles += 1; // simulate a throwaway read `self.cpu_read_byte(nn)` (for incrementing nn)
                let eff_addr = self.read_zp16(nn.wrapping_add(self.x));
                eff_addr
            },
            AddressingMode::IndirectY => {
                let nn = self.fetch_byte();
                let addr: u16 = self.read_zp16(nn);
                let eff_addr: u16 = addr.wrapping_add(self.y as u16);
                if page_cross!(addr, eff_addr) {
                    self.cycles += 1;
                }
                eff_addr
            },
            AddressingMode::IndirectYEc => {
                let nn = self.fetch_byte();
                let addr: u16 = self.read_zp16(nn);
                let eff_addr: u16 = addr.wrapping_add(self.y as u16);
                self.cycles += 1;
                eff_addr
            },
            AddressingMode::Relative => {
                let nn = self.fetch_byte() as i8;
                self.pc.wrapping_add(nn as u16)
            },
            AddressingMode::ZeroPage => {
                self.fetch_byte() as u16
            },
            AddressingMode::ZeroPageX => {
                let nn = self.fetch_byte();
                self.cycles += 1; // simulate a throwaway read `self.cpu_read_byte(nn)` (for incrementing nn)
                nn.wrapping_add(self.x) as u16
            },
            AddressingMode::ZeroPageY => {
                let nn = self.fetch_byte();
                self.cycles += 1; // simulate a throwaway read `self.cpu_read_byte(nn)` (for incrementing nn)
                nn.wrapping_add(self.y) as u16
            }
        }
    }

    fn read_zp16(&mut self, addr: u8) -> u16 {
        let lo = self.cpu_read_byte(addr as u16) as u16;
        let hi = self.cpu_read_byte(addr.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.cpu_read_word(self.pc);
        self.pc += 2;
        word
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.cpu_read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn cpu_write_word(&mut self, addr: u16, data: u16) {
        self.bus.as_mut().unwrap().write_word(addr, data);
        self.cycles += 2;
    }

    fn cpu_write_byte(&mut self, addr: u16, data: u8) {
        self.bus.as_mut().unwrap().write_byte(addr, data);
        self.cycles += 1;
    }

    fn cpu_read_word(&mut self, addr: u16) -> u16 {
        self.cycles += 2;
        self.bus.as_ref().unwrap().read_word(addr)
    }

    fn cpu_read_byte(&mut self, addr: u16) -> u8 {
        self.cycles += 1;
        self.bus.as_ref().unwrap().read_byte(addr)
    }
}
