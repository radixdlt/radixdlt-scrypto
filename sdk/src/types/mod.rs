extern crate alloc;
use alloc::vec::Vec;

mod address;
mod rid;
mod u256;

pub use address::Address;
pub use rid::*;
pub use u256::U256;
pub type Value = serde_json::Value;
pub type SerializedValue = Vec<u8>;
