use crate::*;

pub trait Decode<'de>: Sized {
    fn decode(decoder: Decoder<'de>) -> Result<Self, ()>;
}

pub struct Decoder<'de> {
    data: &'de [u8],
    offset: usize,
}

impl<'de> Decoder<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.offset
    }

    pub fn decode_unit(&mut self) {
        let t = self.read(1);
        assert!(t[0] == TYPE_UNIT);
    }

    pub fn decode_bool(&mut self) -> bool {
        let t = self.read(1);
        assert!(t[0] == TYPE_BOOL && (t[1] == 0 || t[1] == 1));
        t[1] == 1
    }

    fn read(&mut self, n: usize) -> &'de [u8] {
        assert!(self.remaining() >= n);
        let slice = &self.data[self.offset..self.offset + n];
        self.offset += n;
        slice
    }
}
