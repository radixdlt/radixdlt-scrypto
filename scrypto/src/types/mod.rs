mod address;
mod amount;
mod bid;
mod h256;
mod rid;
mod sid;
mod vid;

pub use address::{Address, ParseAddressError};
pub use amount::Amount;
pub use bid::{ParseBIDError, BID};
pub use h256::{ParseH256Error, H256};
pub use rid::{ParseRIDError, RID};
pub use sid::{ParseSIDError, SID};
pub use vid::{ParseVIDError, VID};

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
