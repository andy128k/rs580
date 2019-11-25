use crate::memory::Memory;

#[inline]
fn from_pair(h: u8, l: u8) -> u16 {
    (h as u16) << 8 | (l as u16)
}

#[inline]
fn to_pair(hl: u16) -> (u8, u8) {
    ((hl >> 8) as u8, (hl & 0xFF) as u8)
}

#[inline]
fn neg(v: u8) -> u8 {
    ((0x100_i16 - (v as i16)) & 0xFF) as u8
}

#[inline]
fn add16(x: &mut u16, value: u16) {
    *x = x.overflowing_add(value).0;
}

#[inline]
fn sub16(x: &mut u16, value: u16) {
    *x = x.overflowing_sub(value).0;
}

type OpMask = (char, char, char, char, char, char, char, char);

const fn mask((b7, b6, b5, b4, b3, b2, b1, b0): OpMask) -> u8 {
    ((b7 <= '1') as u8) << 7 |
    ((b6 <= '1') as u8) << 6 |
    ((b5 <= '1') as u8) << 5 |
    ((b4 <= '1') as u8) << 4 |
    ((b3 <= '1') as u8) << 3 |
    ((b2 <= '1') as u8) << 2 |
    ((b1 <= '1') as u8) << 1 |
    ((b0 <= '1') as u8)
}

const fn bits((b7, b6, b5, b4, b3, b2, b1, b0): OpMask, what: char) -> u8 {
    ((b7 == what) as u8) << 7 |
    ((b6 == what) as u8) << 6 |
    ((b5 == what) as u8) << 5 |
    ((b4 == what) as u8) << 4 |
    ((b3 == what) as u8) << 3 |
    ((b2 == what) as u8) << 2 |
    ((b1 == what) as u8) << 1 |
    ((b0 == what) as u8)
}

macro_rules! ops {
    ($opcode:expr, $e:expr) => {
        {
            const MASK: u8 = mask($e);
            const MASKED_VALUE: u8 = bits($e, '1');
            const VAR_MASK: u8 = bits($e, 'X');
            if $opcode & MASK == MASKED_VALUE {
                Some(
                    ($opcode & VAR_MASK) >> (VAR_MASK.trailing_zeros())
                )
            } else {
                None
            }
        }
    };
}

macro_rules! ops2 {
    ($opcode:expr, $e:expr) => {
        {
            const MASK: u8 = mask($e);
            const MASKED_VALUE: u8 = bits($e, '1');
            const VAR1_MASK: u8 = bits($e, 'X');
            const VAR2_MASK: u8 = bits($e, 'Y');
            if $opcode & MASK == MASKED_VALUE {
                Some((
                    ($opcode & VAR1_MASK) >> (VAR1_MASK.trailing_zeros()),
                    ($opcode & VAR2_MASK) >> (VAR2_MASK.trailing_zeros()),
                ))
            } else {
                None
            }
        }
    };
}

#[derive(Default, Debug)]
pub struct Registers {
    pub a: u8,
    pub flag_s: bool,
    pub flag_z: bool,
    pub flag_ac: bool,
    pub flag_p: bool,
    pub flag_c: bool,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl Registers {
    pub fn hl(&self) -> u16 {
        from_pair(self.h, self.l)
    }

    pub fn get_flags_byte(&self) -> u8 {
        (self.flag_s as u8) << 7 |
        (self.flag_z as u8) << 6 |
        (self.flag_ac as u8) << 4 |
        (self.flag_p as u8) << 2 |
        (self.flag_c as u8)
    }

    pub fn set_flags_byte(&mut self, flags_byte: u8) {
        self.flag_s = flags_byte & 0b_1000_0000 != 0;
        self.flag_z = flags_byte & 0b_0100_0000 != 0;
        self.flag_ac = flags_byte & 0b_0001_0000 != 0;
        self.flag_p = flags_byte & 0b_0000_0100 != 0;
        self.flag_c = flags_byte & 0b_0000_0001 != 0;
    }
}

pub struct Machine {
    pub registers: Registers,
    pub halted: bool,
    pub interruption_enabled: bool,
    pub memory: Box<dyn Memory>,
}

impl Machine {
    pub fn new(memory: Box<dyn Memory>) -> Self {
        Self {
            registers: Registers::default(),
            halted: false,
            interruption_enabled: true,
            memory,
        }
    }

    pub fn reset(&mut self) {
        self.registers.pc = 0;
    }

    pub fn step(&mut self) {
        let opcode = self.memory.get_u8(self.registers.pc);
        if opcode == 0 {
            // NOP
            add16(&mut self.registers.pc, 1);
        } else if let Some(rp) = ops!(opcode, ('0', '0', 'X', 'X', '0', '0', '0', '1')) {
            // LXI
            let data16 = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
            self.set_pair(rp, data16);
            add16(&mut self.registers.pc, 3);
        } else if let Some(r) = ops!(opcode, ('0', '0', '0', 'X', '0', '0', '1', '0')) {
            // STAX
            let addr = self.get_pair(r);
            self.memory.set_u8(addr, self.registers.a);
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0b_0010_0010 {
            // SHLD
            let addr = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
            self.memory.set_u8(addr, self.registers.l);
            self.memory.set_u8(addr + 1, self.registers.h);
            add16(&mut self.registers.pc, 3);
        } else if opcode == 0b_0011_0010 {
            // STA
            let addr = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
            self.memory.set_u8(addr, self.registers.a);
            add16(&mut self.registers.pc, 3);
        } else if let Some(rp) = ops!(opcode, ('0', '0', 'X', 'X', '0', '0', '1', '1')) {
            // INX
            self.set_pair(rp, self.get_pair(rp) + 1);
            add16(&mut self.registers.pc, 1);
        } else if let Some(reg) = ops!(opcode, ('0', '0', 'X', 'X', 'X', '1', '0', '0')) {
            // INR
            let value = self.get_location(reg);
            self.set_location(reg, value.overflowing_add(1).0);
            self.registers.flag_ac = (value & 0x0F) + 1 > 0x0F;
            let value = self.get_location(reg);
            self.set_flags(value);
            add16(&mut self.registers.pc, 1);
        } else if let Some(reg) = ops!(opcode, ('0', '0', 'X', 'X', 'X', '1', '0', '1')) {
            // DCR
            let value = self.get_location(reg);
            self.set_location(reg, value.overflowing_sub(1).0);
            self.registers.flag_ac = (value & 0x0F) + 0x0F <= 0x0F;
            let value = self.get_location(reg);
            self.set_flags(value);
            add16(&mut self.registers.pc, 1);
        } else if let Some(reg) = ops!(opcode, ('0', '0', 'X', 'X', 'X', '1', '1', '0')) {
            // MVI
            let data = self.memory.get_u8(self.registers.pc.overflowing_add(1).0);
            self.set_location(reg, data);
            self.registers.pc = self.registers.pc.overflowing_add(2).0;
        } else if opcode == 0b_0000_0111 {
            // RLC
            self.registers.flag_c = (self.registers.a & 0x80) != 0;
            self.registers.a = self.registers.a.rotate_left(1);
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0b_0000_1111 {
            // RRC
            self.registers.flag_c = (self.registers.a & 1) != 0;
            self.registers.a = self.registers.a.rotate_right(1);
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0b_0001_0111 {
            // RAL
            let c = self.registers.flag_c;
            self.registers.flag_c = (self.registers.a & 0x80) != 0;
            self.registers.a = self.registers.a << 1;
            if c {
                self.registers.a |= 1;
            }
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0b_0001_1111 {
            // RAR
            let c = self.registers.flag_c;
            self.registers.flag_c = (self.registers.a & 1) != 0;
            self.registers.a = self.registers.a >> 1;
            if c {
                self.registers.a |= 0x80;
            }
            add16(&mut self.registers.pc, 1);
        } else if let Some(rp) = ops!(opcode, ('0', '0', 'X', 'X', '1', '0', '0', '1')) {
            // DAD
            let operand = self.get_pair(rp);
            let hl = from_pair(self.registers.h, self.registers.l);
            let (value, overflow) = hl.overflowing_add(operand);
            let (h, l) = to_pair(value);
            self.registers.flag_c = overflow;
            self.registers.h = h;
            self.registers.l = l;
            add16(&mut self.registers.pc, 1);
        } else if let Some(r) = ops!(opcode, ('0', '0', '0', 'X', '1', '0', '1', '0')) {
            // LDAX
            self.registers.a = self.memory.get_u8(self.get_pair(r));
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0b_0010_1010 {
            // LHLD
            let addr = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
            let (l, h) = self.memory.get_u8_u8(addr);
            self.registers.l = l;
            self.registers.h = h;
            add16(&mut self.registers.pc, 3);
        } else if opcode == 0b_0011_1010 {
            // LDA
            let addr = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
            self.registers.a = self.memory.get_u8(addr);
            add16(&mut self.registers.pc, 3);
        } else if let Some(rp) = ops!(opcode, ('0', '0', 'X', 'X', '1', '0', '1', '1')) {
            // DCX
            self.set_pair(rp, self.get_pair(rp).overflowing_sub(1).0);
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0x27 {
            // DAA
            if self.registers.a & 0x0F > 9 || self.registers.flag_ac {
                let (value, overflow) = self.registers.a.overflowing_add(0x06);
                self.registers.flag_ac = (self.registers.a & 0x0F) + 0x06 > 0x0F;
                self.registers.a = value;
                self.registers.flag_c = overflow;
                self.set_a_flags();
            }
            if self.registers.a & 0xF0 > 0x90 || self.registers.flag_c {
                let (value, overflow) = self.registers.a.overflowing_add(0x60);
                self.registers.a = value;
                if overflow {
                    self.registers.flag_c = true;
                }
                self.set_a_flags();
            }
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0x2F {
            // CMA
            self.registers.a = !self.registers.a;
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0x37 {
            // STC
            self.registers.flag_c = true;
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0x3F {
            // CMC
            self.registers.flag_c = !self.registers.flag_c;
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0b_01_110_110 {
            self.halted = true;
        } else if let Some((dst, src)) = ops2!(opcode, ('0', '1', 'X', 'X', 'X', 'Y', 'Y', 'Y')) {
            // mov
            let value = self.get_location(src);
            self.set_location(dst, value);
            add16(&mut self.registers.pc, 1);
        } else if let Some((operation, operand_code)) = ops2!(opcode, ('1', '0', 'X', 'X', 'X', 'Y', 'Y', 'Y')) {
            // arithmetic
            let operand = self.get_location(operand_code);
            match operation {
                0 => self.add(operand, false),
                1 => self.add(operand, self.registers.flag_c),
                2 => self.sub(operand, false),
                3 => self.sub(operand, self.registers.flag_c),
                4 => {
                    self.registers.a &= operand;
                    self.registers.flag_c = false;
                    self.set_a_flags();
                },
                5 => {
                    self.registers.a ^= operand;
                    self.registers.flag_c = false;
                    self.registers.flag_ac = false;
                    self.set_a_flags();
                },
                6 => {
                    self.registers.a |= operand;
                    self.registers.flag_c = false;
                    self.set_a_flags();
                },
                7 => {
                    let a = self.registers.a;
                    self.sub(operand, false);
                    self.registers.a = a; // restore A
                },
                _ => unreachable!()
            }
            add16(&mut self.registers.pc, 1);
        } else if let Some(cond) = ops!(opcode, ('1', '1', 'X', 'X', 'X', '0', '0', '0')) {
            // conditional return
            if self.check_cond(cond) {
                self.registers.pc = self.memory.get_u16(self.registers.sp);
                self.registers.sp = self.registers.sp.overflowing_add(2).0;
            } else {
                add16(&mut self.registers.pc, 1);
            }
        } else if let Some(cond) = ops!(opcode, ('1', '1', 'X', 'X', 'X', '0', '1', '0')) {
            // conditional jump
            if self.check_cond(cond) {
                self.registers.pc = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
            } else {
                add16(&mut self.registers.pc, 3);
            }
        } else if let Some(cond) = ops!(opcode, ('1', '1', 'X', 'X', 'X', '1', '0', '0')) {
            // conditional call
            if self.check_cond(cond) {
                sub16(&mut self.registers.sp, 2);
                self.memory.set_u16(self.registers.sp, self.registers.pc.overflowing_add(3).0);
                self.registers.pc = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
            } else {
                add16(&mut self.registers.pc, 3);
            }
        } else if let Some(rp) = ops!(opcode, ('1', '1', 'X', 'X', '0', '0', '0', '1')) {
            // POP
            let data16 = self.memory.get_u16(self.registers.sp);
            self.set_pair_flags(rp, data16);
            self.registers.sp = self.registers.sp.overflowing_add(2).0;
            add16(&mut self.registers.pc, 1);
        } else if let Some(rp) = ops!(opcode, ('1', '1', 'X', 'X', '0', '1', '0', '1')) {
            // PUSH
            let data16 = self.get_pair_flags(rp);
            sub16(&mut self.registers.sp, 2);
            self.memory.set_u16(self.registers.sp, data16);
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0xC3 {
            // JMP
            self.registers.pc = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
        } else if opcode == 0xC9 {
            // RET
            self.registers.pc = self.memory.get_u16(self.registers.sp);
            self.registers.sp = self.registers.sp.overflowing_add(2).0;
        } else if opcode == 0xCD {
            // CALL
            sub16(&mut self.registers.sp, 2);
            self.memory.set_u16(self.registers.sp, self.registers.pc.overflowing_add(3).0);
            self.registers.pc = self.memory.get_u16(self.registers.pc.overflowing_add(1).0);
        } else if let Some(operation) = ops!(opcode, ('1', '1', 'X', 'X', 'X', '1', '1', '0')) {
            // arithmetic
            let operand = self.memory.get_u8(self.registers.pc.overflowing_add(1).0);
            match operation {
                0 => self.add(operand, false),
                1 => self.add(operand, self.registers.flag_c),
                2 => self.sub(operand, false),
                3 => self.sub(operand, self.registers.flag_c),
                4 => {
                    self.registers.a &= operand;
                    self.registers.flag_c = false;
                    self.set_a_flags();
                },
                5 => {
                    self.registers.a ^= operand;
                    self.registers.flag_c = false;
                    self.registers.flag_ac = false;
                    self.set_a_flags();
                },
                6 => {
                    self.registers.a |= operand;
                    self.registers.flag_c = false;
                    self.set_a_flags();
                },
                7 => {
                    let a = self.registers.a;
                    self.sub(operand, false);
                    self.registers.a = a; // restore A
                },
                _ => unreachable!()
            }
            self.registers.pc = self.registers.pc.overflowing_add(2).0;
        } else if let Some(exp) = ops!(opcode, ('1', '1', 'X', 'X', 'X', '1', '1', '1')) {
            // RST
            sub16(&mut self.registers.sp, 2);
            self.memory.set_u16(self.registers.sp, self.registers.pc.overflowing_add(1).0);
            self.registers.pc = (exp as u16) << 3;
        } else if opcode == 0xD3 {
            // OUT
            let port = self.memory.get_u8(self.registers.pc.overflowing_add(1).0);
            self.out(port, self.registers.a);
            add16(&mut self.registers.pc, 2);
        } else if opcode == 0xDB {
            // IN
            let port = self.memory.get_u8(self.registers.pc.overflowing_add(1).0);
            self.registers.a = self.inp(port);
            add16(&mut self.registers.pc, 2);
        } else if opcode == 0xE3 {
            // XTHL
            self.memory.swap(self.registers.sp, &mut self.registers.l);
            self.memory.swap(self.registers.sp + 1, &mut self.registers.h);
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0xE9 {
            // PCHL
            self.registers.pc = self.registers.hl();
        } else if opcode == 0xF9 {
            // SPHL
            self.registers.sp = self.registers.hl();
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0xEB {
            // XCHG
            std::mem::swap(&mut self.registers.l, &mut self.registers.e);
            std::mem::swap(&mut self.registers.h, &mut self.registers.d);
            add16(&mut self.registers.pc, 1);
        } else if opcode == 0xF3 {
            // DI
            add16(&mut self.registers.pc, 1);
            self.interruption_enabled = false;
        } else if opcode == 0xFB {
            // EI
            add16(&mut self.registers.pc, 1);
            self.interruption_enabled = true;
        } else {
            panic!("Bad opcode 0x{:02X} at 0x{:04X}.", opcode, self.registers.pc);
        }
    }

    fn set_flags(&mut self, register: u8) {
        self.registers.flag_s = register >= 0b_1000_0000;
        self.registers.flag_z = register == 0;
        self.registers.flag_p = register.count_ones() % 2 == 0;
    }

    fn set_a_flags(&mut self) {
        self.set_flags(self.registers.a);
    }

    fn add(&mut self, operand: u8, carry: bool) {
        let (o, a) = to_pair((self.registers.a as u16) + (operand as u16) + if carry { 1 } else { 0 });
        self.registers.a = a;
        self.registers.flag_c = o > 0;
        self.registers.flag_ac = (self.registers.a & 0x0F) + (operand & 0x0F) + if carry { 1 } else { 0 } > 0x0F;
        self.set_a_flags();
    }

    fn sub(&mut self, operand: u8, carry: bool) {
        let operand = operand.overflowing_add(if carry { 1 } else { 0 }).0;
        let (o, a) = to_pair((self.registers.a as u16) + (neg(operand) as u16));
        self.registers.a = a;
        self.registers.flag_c = o <= 0;
        self.registers.flag_ac = (self.registers.a & 0x0F) + (neg(operand) & 0x0F) <= 0x0F;
        self.set_a_flags();
    }

    fn get_location(&mut self, reg: u8) -> u8 {
        match reg {
            0 => self.registers.b,
            1 => self.registers.c,
            2 => self.registers.d,
            3 => self.registers.e,
            4 => self.registers.h,
            5 => self.registers.l,
            6 => self.memory.get_u8(self.registers.hl()),
            7 => self.registers.a,
            _ => unreachable!(),
        }
    }

    fn set_location(&mut self, reg: u8, value: u8) {
        match reg {
            0 => self.registers.b = value,
            1 => self.registers.c = value,
            2 => self.registers.d = value,
            3 => self.registers.e = value,
            4 => self.registers.h = value,
            5 => self.registers.l = value,
            6 => self.memory.set_u8(self.registers.hl(), value),
            7 => self.registers.a = value,
            _ => unreachable!(),
        }
    }

    fn get_pair(&self, rp: u8) -> u16 {
        match rp {
            0 => from_pair(self.registers.b, self.registers.c),
            1 => from_pair(self.registers.d, self.registers.e),
            2 => from_pair(self.registers.h, self.registers.l),
            3 => self.registers.sp,
            _ => unreachable!(),
        }
    }

    fn set_pair(&mut self, rp: u8, data16: u16) {
        let (h, l) = to_pair(data16);
        match rp {
            0 => { self.registers.b = h; self.registers.c = l; },
            1 => { self.registers.d = h; self.registers.e = l; },
            2 => { self.registers.h = h; self.registers.l = l; },
            3 => self.registers.sp = data16,
            _ => unreachable!(),
        }
    }

    fn get_pair_flags(&self, rp: u8) -> u16 {
        match rp {
            0 => from_pair(self.registers.b, self.registers.c),
            1 => from_pair(self.registers.d, self.registers.e),
            2 => from_pair(self.registers.h, self.registers.l),
            3 => from_pair(self.registers.get_flags_byte(), self.registers.a),
            _ => unreachable!(),
        }
    }

    fn set_pair_flags(&mut self, rp: u8, data16: u16) {
        let (h, l) = to_pair(data16);
        match rp {
            0 => { self.registers.b = h; self.registers.c = l; },
            1 => { self.registers.d = h; self.registers.e = l; },
            2 => { self.registers.h = h; self.registers.l = l; },
            3 => { self.registers.set_flags_byte(h); self.registers.a = l; },
            _ => unreachable!(),
        }
    }

    fn check_cond(&self, cond: u8) -> bool {
        match cond {
            0 => !self.registers.flag_z,
            1 => self.registers.flag_z,
            2 => !self.registers.flag_c,
            3 => self.registers.flag_c,
            4 => !self.registers.flag_p,
            5 => self.registers.flag_p,
            6 => !self.registers.flag_s,
            7 => self.registers.flag_s,
            _ => unreachable!(),
        }
    }

    pub fn out(&self, port: u8, data: u8) {
        println!("OUT: port 0x{:02X} data 0x{:02X}", port, data);
    }

    pub fn inp(&self, port: u8) -> u8 {
        println!("IN: port 0x{:02X}", port);
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neg() {
        assert_eq!(neg(0), 0);
        assert_eq!(neg(1), 0xFF);
        assert_eq!(neg(127), 0x81);
        assert_eq!(neg(128), 128);
    }

    #[test]
    fn test_mask() {
        const E: OpMask = ('0', '1', 'X', 'X', 'X', '1', '0', '0');
        const MASK: u8 = mask(E);
        assert_eq!(MASK, 0b_11_000_111);
        const MASK1: u8 = bits(E, '1');
        assert_eq!(MASK1, 0b_01_000_100);
        const SHIFT: u8 = bits(E, 'X');
        assert_eq!(SHIFT, 0b_00_111_000);

        assert_eq!(
            ops!(0b01_010_100, ('0', '1', 'X', 'X', 'X', '1', '0', '0')),
            Some(0b_010)
        );

        assert_eq!(
            ops!(0b01_010_101, ('0', '1', 'X', 'X', 'X', '1', '0', '0')),
            None
        );
    }
}
