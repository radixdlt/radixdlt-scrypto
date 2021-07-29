extern crate alloc;
use alloc::vec::Vec;

use crate::*;

pub trait Encode {
    fn encode(&self, encoder: Encoder);
}

pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn encode_unit(&mut self) {
        self.buf.push(TYPE_UNIT);
    }

    pub fn encode_bool(&mut self, value: bool) {
        self.buf.push(TYPE_BOOL);
        self.buf.push(if value { 1u8 } else { 0u8 });
    }

    pub fn encode_i8(&mut self, value: i8) {
        self.buf.push(TYPE_I8);
        self.buf.push(value as u8);
    }
}

impl Into<Vec<u8>> for Encoder {
    fn into(self) -> Vec<u8> {
        self.buf
    }
}
