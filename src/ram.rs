pub use crate::memory::Memory;

pub struct RAM {
    data: Vec<u8>,
}

impl RAM {
    pub fn new(size: usize) -> Self {
        Self { data: vec![0; size] }
    }
}

impl std::default::Default for RAM {
    fn default() -> Self {
        Self::new(65536)
    }
}

impl Memory for RAM {
    #[inline]
    fn get_u8(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    #[inline]
    fn set_u8(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}
