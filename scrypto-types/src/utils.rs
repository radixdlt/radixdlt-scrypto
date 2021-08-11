pub enum CopyArrayError {
    InvalidLength,
}

pub fn copy_u8_array<const N: usize>(slice: &[u8]) -> Result<[u8; N], CopyArrayError> {
    if slice.len() == N {
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&slice[0..N]);
        Ok(bytes)
    } else {
        Err(CopyArrayError::InvalidLength)
    }
}
