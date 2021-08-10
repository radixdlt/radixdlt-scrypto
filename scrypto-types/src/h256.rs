use crate::dependencies::*;

/// Represents a 32-byte hash digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct H256 {
    raw: [u8; 32],
}

#[derive(Debug)]
pub enum DecodeH256Error {
    InvalidHex(hex::FromHexError),
    InvalidLength,
}

impl H256 {
    pub fn new(raw: [u8; 32]) -> Self {
        Self { raw }
    }

    /// Decode a hash from its hex representation.
    pub fn from_hex(hex: &str) -> Result<Self, DecodeH256Error> {
        let data = hex::decode(hex).map_err(|e| DecodeH256Error::InvalidHex(e))?;
        Self::from_slice(&data)
    }

    /// Decode a hash from a slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DecodeH256Error> {
        Ok(Self {
            raw: bytes
                .try_into()
                .map_err(|_| DecodeH256Error::InvalidLength)?,
        })
    }

    /// Returns the lower 26 bytes.
    pub fn lower_26_bytes(&self) -> [u8; 26] {
        let mut result = [0u8; 26];
        result.copy_from_slice(&self.raw[6..32]);
        result
    }

    /// Obtain a slice of this hash.
    pub fn slice(&self) -> &[u8] {
        &self.raw
    }
}

impl<T: AsRef<str>> From<T> for H256 {
    fn from(s: T) -> Self {
        H256::from_hex(s.as_ref()).unwrap()
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

impl ToString for H256 {
    fn to_string(&self) -> String {
        hex::encode(self.as_ref())
    }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::dependencies::*;
    use crate::*;

    #[test]
    fn test_from_to_string() {
        let s = "b177968c9c68877dc8d33e25759183c556379daa45a4d78a2b91c70133c873ca";
        let h: H256 = s.into();
        assert_eq!(h.to_string(), s);
    }
}
