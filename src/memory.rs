pub trait Memory {
    #[inline]
    fn get_u8(&self, addr: u16) -> u8;

    #[inline]
    fn set_u8(&mut self, addr: u16, value: u8);

    fn get_range(&self, mut from: u16, to: u16) -> Vec<u8> {
        let mut result = Vec::new();
        while from < to {
            result.push(self.get_u8(from));
            from = from.overflowing_add(1).0;
        }
        result
    }

    fn set_range(&mut self, mut addr: u16, data: &[u8]) {
        for b in data {
            self.set_u8(addr, *b);
            addr = addr.overflowing_add(1).0;
        }
    }

    #[inline]
    fn get_u8_u8(&self, addr: u16) -> (u8, u8) {
        (
            self.get_u8(addr),
            self.get_u8(addr.overflowing_add(1).0),
        )
    }

    #[inline]
    fn get_u16(&self, addr: u16) -> u16 {
        let (l, h) = self.get_u8_u8(addr);
        (h as u16) << 8 | (l as u16)
    }

    #[inline]
    fn set_u8_u8(&mut self, addr: u16, (l, h): (u8, u8)) {
        self.set_u8(addr, l);
        self.set_u8(addr.overflowing_add(1).0, h);
    }

    #[inline]
    fn set_u16(&mut self, addr: u16, value: u16) {
        let h = (value >> 8) as u8;
        let l = (value & 0xFF) as u8;
        self.set_u8_u8(addr, (l, h));
    }

    #[inline]
    fn swap(&mut self, addr: u16, value: &mut u8) {
        let m = self.get_u8(addr);
        self.set_u8(addr, *value);
        *value = m;
    }
}
