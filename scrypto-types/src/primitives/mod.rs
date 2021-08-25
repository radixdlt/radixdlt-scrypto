mod address;
mod bid;
mod h256;
mod rid;
mod u256;

pub use address::{Address, DecodeAddressError};
pub use bid::BID;
pub use h256::{DecodeH256Error, H256};
pub use rid::RID;
pub use u256::U256;

#[derive(Debug, Clone)]
enum CopyArrayError {
    InvalidLength,
}

fn copy_u8_array<const N: usize>(slice: &[u8]) -> Result<[u8; N], CopyArrayError> {
    if slice.len() == N {
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&slice[0..N]);
        Ok(bytes)
    } else {
        Err(CopyArrayError::InvalidLength)
    }
}
