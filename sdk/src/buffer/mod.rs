extern crate alloc;
use alloc::str;
use alloc::vec::Vec;

use bincode_core::{deserialize, serialize, BufferWriterError, CoreWrite, DefaultOptions};
use serde::{Deserialize, Serialize};

struct GrowableBufferWriter {
    buffer: Vec<u8>,
}

impl GrowableBufferWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::<u8>::new(),
        }
    }
}

impl CoreWrite for &'_ mut GrowableBufferWriter {
    type Error = BufferWriterError;

    fn write(&mut self, val: u8) -> Result<(), Self::Error> {
        self.buffer.push(val);
        Ok(())
    }
}

impl CoreWrite for GrowableBufferWriter {
    type Error = BufferWriterError;
    fn write(&mut self, val: u8) -> Result<(), Self::Error> {
        self.buffer.push(val);
        Ok(())
    }
}

pub fn radix_encode<T: Serialize>(v: &T) -> Vec<u8> {
    let mut writer = GrowableBufferWriter::new();
    let options = DefaultOptions::new();
    serialize(v, &mut writer, options).unwrap();
    writer.buffer
}

pub fn radix_decode<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> T {
    let options = DefaultOptions::new();
    deserialize(buf, options).unwrap()
}

pub fn radix_encode_value<T: Serialize>(v: &T) -> Vec<u8> {
    let buf = serde_json::to_string(v).unwrap();
    radix_encode(&buf.into_bytes())
}

pub fn radix_decode_value<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> T {
    let buf = str::from_utf8(radix_decode(buf)).unwrap();
    serde_json::from_str(&buf).unwrap()
}

#[cfg(test)]
mod tests {

    extern crate alloc;
    use alloc::string::ToString;

    #[test]
    fn test_serialization() {
        let obj = crate::abi::EmitLogInput {
            level: "TRACE".to_string(),
            message: "Hello, world!".to_string(),
        };
        let bin = crate::buffer::radix_encode(&obj);
        let obj2 = crate::buffer::radix_decode::<crate::abi::EmitLogInput>(&bin);
        let bin2 = crate::buffer::radix_encode(&obj2);
        assert_eq!(bin, bin2);
    }
}
