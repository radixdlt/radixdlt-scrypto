use sbor::any::*;
use scrypto::rust::fmt;
use scrypto::rust::vec::Vec;

use crate::utils::*;

#[derive(Clone)]
pub struct ValidatedData {
    pub raw: Vec<u8>,
    pub value: Value,
}

impl fmt::Debug for ValidatedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: format the value based on the tiny lang introduced by transaction manifest.
        if self.raw.len() <= 1024 {
            write!(f, "{}", format_data(&self.raw).unwrap())
        } else {
            write!(f, "LargeValue(len: {})", self.raw.len())
        }
    }
}
