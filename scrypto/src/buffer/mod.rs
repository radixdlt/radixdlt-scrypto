extern crate alloc;
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

/// Encodes a value into byte array, using Bincode.
pub fn bincode_encode<T: Serialize>(v: &T) -> Vec<u8> {
    let mut writer = GrowableBufferWriter::new();
    let options = DefaultOptions::new();
    serialize(v, &mut writer, options).unwrap();
    writer.buffer
}

/// Decodes a value from a byte buffer, using Bincode.
pub fn bincode_decode<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> T {
    let options = DefaultOptions::new();
    deserialize(buf, options).unwrap()
}

/// Encodes a value into byte array, using Radix data format.
pub fn radix_encode<T: Serialize>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}

/// Decodes a value from a byte buffer, using Radix data format.
pub fn radix_decode<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> T {
    serde_json::from_slice(buf).unwrap()
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::String;
    use alloc::string::ToString;

    use crate::buffer::*;
    use crate::kernel::*;
    use crate::types::*;

    #[test]
    fn test_serialization() {
        let obj = PutComponentStateInput {
            component: Address::System,
            state: radix_encode(&"test".to_string()),
        };
        let encoded = crate::buffer::bincode_encode(&obj);
        let decoded = crate::buffer::bincode_decode::<PutComponentStateInput>(&encoded);
        assert_eq!(decoded.component, Address::System);
        assert_eq!(radix_decode::<String>(&decoded.state), "test");
    }
}
