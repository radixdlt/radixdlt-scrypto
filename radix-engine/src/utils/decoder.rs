use scrypto::buffer::*;
use scrypto::rust::vec::Vec;

use crate::engine::*;

/// Decodes data into an instance of `T`.
pub fn decode_data<T: sbor::Decode>(data: Vec<u8>) -> Result<T, RuntimeError> {
    scrypto_decode(&data).map_err(RuntimeError::InvalidData)
}
