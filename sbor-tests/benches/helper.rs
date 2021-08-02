use bincode_core::{deserialize, serialize, BufferWriterError, CoreWrite, DefaultOptions};
use serde::{Deserialize, Serialize};

struct GrowableBufferWriter {
    buffer: Vec<u8>,
}

impl GrowableBufferWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::<u8>::with_capacity(256),
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

pub fn bincode_encode<T: Serialize>(v: &T) -> Vec<u8> {
    let mut writer = GrowableBufferWriter::new();
    let options = DefaultOptions::new();
    serialize(v, &mut writer, options).unwrap();
    writer.buffer
}

pub fn bincode_decode<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> Result<T, String> {
    let options = DefaultOptions::new();
    deserialize(buf, options).map_err(|e| e.to_string())
}

pub fn json_encode<T: Serialize>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}

pub fn json_decode<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> Result<T, String> {
    serde_json::from_slice(buf).map_err(|e| e.to_string())
}
