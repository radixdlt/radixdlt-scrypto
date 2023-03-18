use crate::rust::vec::Vec;

/// Copies a slice to a fixed-sized array.
pub fn copy_u8_array<const N: usize>(slice: &[u8]) -> [u8; N] {
    if slice.len() == N {
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&slice[0..N]);
        bytes
    } else {
        panic!("Invalid length");
    }
}

/// Combines a u8 with a u8 slice.
pub fn combine(ty: u8, bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(1 + bytes.len());
    v.push(ty);
    v.extend(bytes);
    v
}
