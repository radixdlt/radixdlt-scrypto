mod address;
mod amount;
mod bid;
mod h256;
mod mid;
mod rid;
mod vid;

pub use address::{Address, ParseAddressError, ACCOUNT_PACKAGE, RADIX_TOKEN, SYSTEM_PACKAGE};
pub use amount::{Amount, ParseAmountError};
pub use bid::{Bid, ParseBidError};
pub use h256::{ParseH256Error, H256};
pub use mid::{Mid, ParseMidError};
pub use rid::{ParseRidError, Rid};
pub use vid::{ParseVidError, Vid};

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

fn combine(ty: u8, bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(1 + bytes.len());
    v.push(ty);
    v.extend(bytes);
    v
}
