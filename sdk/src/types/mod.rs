extern crate alloc;
use alloc::vec::Vec;

mod address;
mod hash;
mod rid;
mod u256;

pub use address::*;
pub use hash::*;
pub use rid::*;
pub use u256::*;

pub type Value = serde_json::Value;
pub type SerializedValue = Vec<u8>;
