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

const fn masked_value((b7, b6, b5, b4, b3, b2, b1, b0): OpMask) -> u8 {
    ((b7 == '1') as u8) << 7 |
    ((b6 == '1') as u8) << 6 |
    ((b5 == '1') as u8) << 5 |
    ((b4 == '1') as u8) << 4 |
    ((b3 == '1') as u8) << 3 |
    ((b2 == '1') as u8) << 2 |
    ((b1 == '1') as u8) << 1 |
    ((b0 == '1') as u8)
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
            const VAR_MASK: u8 = bits($e, 'D');
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

#[derive(Default)]
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
}

pub struct Memory {
    data: [u8; 65536],
}

impl std::default::Default for Memory {
    fn default() -> Self {
        Self {
            data: [0_u8; 65536],
        }
    }
}

#[derive(Default)]
pub struct Machine {
    pub registers: Registers,
    pub halted: bool,
    pub memory: Memory,
}

impl Machine {
    pub fn reset(&mut self) {
        self.registers.pc = 0;
    }

    pub fn step(&mut self) {
        let opcode = self.memory.data[self.registers.pc as usize];

        if opcode == 0 {
            // NOP
            self.registers.pc += 1;
            return;
        }
        if opcode == 0b_01_110_110 {
            self.halted = true;
            return;
        }

        if let Some(rp) = ops!(opcode, ('0', '0', 'D', 'D', '0', '0', '0', '1')) {
            // LXI
            let data16 = self.mem_get_u16(self.registers.pc + 1);
            self.set_pair(rp, data16);
            self.registers.pc += 3;
            return;
        }

        if opcode & 0b_1110_1111 == 0b_0000_0010 {
            // STAX
            self.memory.data[self.get_pair((opcode >> 4) & 1) as usize] = self.registers.a;
            self.registers.pc += 1;
            return;
        }

        if opcode == 0b_0010_0010 {
            // SHLD
            let addr = self.mem_get_u16(self.registers.pc + 1) as usize;
            self.memory.data[addr] = self.registers.l;
            self.memory.data[addr + 1] = self.registers.h;
            self.registers.pc += 3;
            return;
        }
        if opcode == 0b_0011_0010 {
            // STA
            let addr = self.mem_get_u16(self.registers.pc + 1) as usize;
            self.memory.data[addr] = self.registers.a;
            self.registers.pc += 3;
            return;
        }
        if opcode & 0b_1100_1111 == 0b_0000_0011 {
            // INX
            let rp = (opcode >> 4) & 3;
            self.set_pair(rp, self.get_pair(rp) + 1);
            self.registers.pc += 1;
            return;
        }
        if opcode & 0b_11_000_111 == 0b_00_000_100 {
            // INR
            let reg = (opcode >> 3) & 7;
            let value = *self.get_location(reg);
            *self.get_location(reg) = value + 1;
            self.registers.flag_ac = (value & 0x0F) + 1 > 0x0F;
            self.set_a_flags();
            self.registers.pc += 1;
            return;
        }
        if opcode & 0b_11_000_111 == 0b_00_000_101 {
            // DCR
            let reg = (opcode >> 3) & 7;
            let value = *self.get_location(reg);
            *self.get_location(reg) = value + 0xFF;
            self.registers.flag_ac = (value & 0x0F) + 0x0F <= 0x0F;
            self.set_a_flags();
            self.registers.pc += 1;
            return;
        }
        if opcode & 0b_11_000_111 == 0b_00_000_110 {
            // MVI
            let data = self.mem_get_u8(self.registers.pc + 1);
            *self.get_location(opcode >> 3 & 7) = data;
            self.registers.pc += 2;
            return;
        }

        if opcode & 0b_1100_0000 == 0b_0100_0000 {
            // mov
            *self.get_location(opcode >> 3 & 7) = *self.get_location(opcode & 7);
            self.registers.pc += 1;
            return;
        }

        if opcode & 0b_11_00_0000 == 0b_10_00_0000 {
            // arithmetic
            let operand = *self.get_location(opcode & 7);
            match (opcode >> 3) & 7 {
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
            self.registers.pc += 1;
            return;
        }

        if opcode == 0xC9 {
            // RET
            self.registers.pc = self.mem_get_u16(self.registers.sp);
            self.registers.sp += 2;
            return;
        }

        panic!("Bad opcode");
    }

    fn set_a_flags(&mut self) {
        self.registers.flag_s = self.registers.a >= 0b_1000_0000;
        self.registers.flag_z = self.registers.a == 0;
        self.registers.flag_p = self.registers.a.count_ones() % 2 == 0;
    }

    fn add(&mut self, operand: u8, carry: bool) {
        let (o, a) = to_pair((self.registers.a as u16) + (operand as u16) + if carry { 1 } else { 0 });
        self.registers.a = a;
        self.registers.flag_c = o > 0;
        self.registers.flag_ac = (self.registers.a & 0x0F) + (operand & 0x0F) + if carry { 1 } else { 0 } > 0x0F;
        self.set_a_flags();
    }

    fn sub(&mut self, operand: u8, carry: bool) {
        let (o, a) = to_pair((self.registers.a as u16) + (neg(operand + if carry { 1 } else { 0 }) as u16));
        self.registers.a = a;
        self.registers.flag_c = o <= 0;
        self.registers.flag_ac = (self.registers.a & 0x0F) + (neg(operand + if carry { 1 } else { 0 }) & 0x0F) <= 0x0F;
        self.set_a_flags();
    }

    fn get_location(&mut self, reg: u8) -> &mut u8 {
        match reg {
            0 => &mut self.registers.b,
            1 => &mut self.registers.c,
            2 => &mut self.registers.d,
            3 => &mut self.registers.e,
            4 => &mut self.registers.h,
            5 => &mut self.registers.l,
            6 => &mut self.memory.data[self.registers.hl() as usize],
            7 => &mut self.registers.a,
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
            3 => from_pair(0, self.registers.a), // TODO PSW
            _ => unreachable!(),
        }
    }

    #[inline]
    fn mem_get_u8(&self, addr: u16) -> u8 {
        self.memory.data[addr as usize]
    }

    #[inline]
    fn mem_get_u16(&self, addr: u16) -> u16 {
        from_pair(self.mem_get_u8(addr + 1), self.mem_get_u8(addr))
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
        const E: OpMask = ('0', '1', 'D', 'D', 'D', '1', '0', '0');
        const MASK: u8 = mask(E);
        assert_eq!(MASK, 0b_11_000_111);
        const MASK1: u8 = masked_value(E);
        assert_eq!(MASK1, 0b_01_000_100);
        const SHIFT: u8 = bits(E, 'D');
        assert_eq!(SHIFT, 0b_00_111_000);

        assert_eq!(
            ops!(0b01_010_100, ('0', '1', 'D', 'D', 'D', '1', '0', '0')),
            Some(0b_010)
        );

        assert_eq!(
            ops!(0b01_010_101, ('0', '1', 'D', 'D', 'D', '1', '0', '0')),
            None
        );
    }

    #[test]
    fn it_works() {
        let mut m: Machine = Default::default();
        m.reset();
        // m.step();
        assert_eq!(2 + 2, 4);
    }
}
