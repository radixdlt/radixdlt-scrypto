use blake2::Digest as Blake2Digest;

use crate::prelude::*;

/// Represents a 32-byte hash accumulator.
#[derive(Default)]
pub struct HashAccumulator {
    inner: Blake2b256,
    input_size: usize,
}

impl HashAccumulator {
    pub fn new() -> Self {
        Default::default()
    }

    /// Effectively concatenates `data` to the payload-to-be-hashed
    pub fn concat(self, data: impl AsRef<[u8]>) -> Self {
        let bytes = data.as_ref();

        Self {
            inner: self.inner.chain_update(bytes),
            input_size: self
                .input_size
                .checked_add(bytes.len())
                .expect("Input to digest somehow larger than usize"),
        }
    }

    pub fn concat_mut(&mut self, data: impl AsRef<[u8]>) {
        let bytes = data.as_ref();
        self.inner.update(bytes);
        self.input_size = self
            .input_size
            .checked_add(bytes.len())
            .expect("Input to digest somehow larger than usize");
    }

    pub fn input_length(&self) -> usize {
        self.input_size
    }

    pub fn finalize(self) -> Hash {
        Hash(self.inner.finalize().into())
    }
}
