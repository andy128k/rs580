pub mod memory;
pub mod ram;
pub mod rom;
pub mod segmented_memory;
pub mod cpu;

pub use cpu::Machine;
pub use memory::Memory;
pub use ram::RAM;
pub use rom::ROM;
pub use segmented_memory::SegmentedMemory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::RAM;

    #[test]
    fn test_opcodes_implemented() {
        let mut m: Machine = Machine::new(Box::new(RAM::default()));
        for i in 0..=255 {
            match i {
                0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xCB | 0xD9 | 0xDD | 0xED | 0xFD => continue,
                _ => {}
            }
            for a in &[0xFFFC, 0xFFFD, 0xFFFE, 0xFFFF, 0x0000, 0x0001, 0x0002, 0x0003] {
                println!("Try instruction {:02x} at {:04x}.", i, a);
                m.memory.set_u8(*a, i);
                m.registers.pc = *a;
                m.step();
            }
        }
    }

    #[test]
    fn test_daa() {
        let mut m: Machine = Machine::new(Box::new(RAM::default()));
        m.memory.set_u8(0, 0x27);
        m.reset();
        m.registers.a = 0x9B;
        m.registers.flag_c = false;
        m.registers.flag_ac = false;
        m.step();
        assert_eq!(m.registers.a, 1);
        assert!(m.registers.flag_c);
        assert!(m.registers.flag_ac);
    }

    #[test]
    fn it_works() {
        let mut m: Machine = Machine::new(Box::new(RAM::default()));
        m.reset();
        // m.step();
        assert_eq!(2 + 2, 4);
    }
}
