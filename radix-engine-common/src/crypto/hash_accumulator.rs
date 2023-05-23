use blake2::Digest as Blake2Digest;

use crate::prelude::*;

/// Represents a 32-byte hash accumulator.
pub struct HashAccumulator {
    inner: Blake2b256,
    input_size: usize,
}

impl HashAccumulator {
    pub fn new() -> Self {
        Self {
            inner: Blake2b256::new(),
            input_size: 0,
        }
    }

    /// Effectively concatenates `data` to the payload-to-be-hashed
    pub fn update(self, data: impl AsRef<[u8]>) -> Self {
        let bytes = data.as_ref();

        Self {
            inner: self.inner.chain_update(bytes),
            input_size: self
                .input_size
                .checked_add(bytes.len())
                .expect("Input to digest somehow larger than usize"),
        }
    }

    pub fn input_length(&self) -> usize {
        self.input_size
    }

    pub fn finalize(self) -> Hash {
        Hash(self.inner.finalize().into())
    }
}
