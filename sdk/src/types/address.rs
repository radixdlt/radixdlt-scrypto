extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use core::fmt;

use serde::{Deserialize, Serialize};

use crate::utils::*;

/// Represents a Radix Engine address:
/// * `0x00` - System
/// * `0x01` - Radix native token
/// * `0x03 + lower_26_bytes(sha_256_twice(33_byte_compressed_pubkey + nonce))` - A resource address
/// * `0x04 + 33_byte_compressed_pubkey` - An account address
/// * `0x05 + sha_256_twice(tx_id + nonce)` - A component address
/// * `0x06 + sha_256_twice(code)` - A blueprint address
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String")]
#[serde(into = "String")]
pub struct Address {
    raw: String,
}

impl Address {
    pub fn radix_native_token() -> Self {
        "0x01".into()
    }

    pub fn resource(hash: [u8; 26]) -> Self {
        let mut buf = String::from("0x03");
        buf.push_str(to_hex_string(&hash).as_str());
        buf.into()
    }

    pub fn component(hash: [u8; 32]) -> Self {
        let mut buf = String::from("0x05");
        buf.push_str(to_hex_string(&hash).as_str());
        buf.into()
    }

    pub fn blueprint(hash: [u8; 32]) -> Self {
        let mut buf = String::from("0x06");
        buf.push_str(to_hex_string(&hash).as_str());
        buf.into()
    }
}

impl From<&str> for Address {
    fn from(s: &str) -> Self {
        Self { raw: s.to_string() }
    }
}

impl From<String> for Address {
    fn from(s: String) -> Self {
        Self { raw: s }
    }
}

impl Into<String> for Address {
    fn into(self) -> String {
        self.raw
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        self.raw.clone()
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
