mod address;
mod bid;
mod h256;
mod rid;
mod u256;

pub const SCRYPTO_TYPE_U256: u8 = 0x80;
pub const SCRYPTO_TYPE_ADDRESS: u8 = 0x81;
pub const SCRYPTO_TYPE_H256: u8 = 0x82;
pub const SCRYPTO_TYPE_BID: u8 = 0x83;
pub const SCRYPTO_TYPE_RID: u8 = 0x84;

pub use address::{Address, ParseAddressError};
pub use bid::BID;
pub use h256::{ParseH256Error, H256};
pub use rid::RID;
pub use u256::U256;

use crate::rust::vec::Vec;

fn copy_u8_array<const N: usize>(slice: &[u8]) -> [u8; N] {
    if slice.len() == N {
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&slice[0..N]);
        bytes
    } else {
        panic!("Invalid length");
    }
}

fn combine2(ty: u8, bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(ty);
    v.extend(bytes);
    v
}

fn combine3(ty: u8, bytes: &[u8], bytes2: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(ty);
    v.extend(bytes);
    v.extend(bytes2);
    v
}
