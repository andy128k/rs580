use crate::memory::Memory;

pub struct SegmentedMemory {
    segments: Vec<(usize, usize, Box<dyn Memory>)>,
}

impl SegmentedMemory {
    pub fn new() -> Self {
        Self {
            segments: Vec::new()
        }
    }

    pub fn add(mut self, from: usize, to: usize, memory: Box<dyn Memory>) -> Self {
        self.segments.push((from, to, memory));
        self
    }
}

impl Memory for SegmentedMemory {
    #[inline]
    fn get_u8(&self, addr: u16) -> u8 {
        for (from, to, memory) in &self.segments {
            if *from <= (addr as usize) && (addr as usize) < *to {
                return memory.get_u8(addr - (*from as u16));
            }
        }
        0xFF
    }

    #[inline]
    fn set_u8(&mut self, addr: u16, value: u8) {
        for (from, to, ref mut memory) in self.segments.iter_mut() {
            if *from <= (addr as usize) && (addr as usize) < *to {
                memory.set_u8(addr - (*from as u16), value);
            }
        }
    }
}
