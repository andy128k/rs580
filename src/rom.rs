use crate::memory::Memory;

pub struct ROM {
    data: Vec<u8>,
}

impl ROM {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_owned()
        }
    }
}

impl Memory for ROM {
    #[inline]
    fn get_u8(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    #[inline]
    fn set_u8(&mut self, _addr: u16, _value: u8) {
        // DO NOTHING
    }
}
