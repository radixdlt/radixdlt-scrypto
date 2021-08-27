use crate::primitives::*;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;

/// Represents a 32-byte hash digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct H256(pub [u8; 32]);

/// Represents an error when parsing H256.
#[derive(Debug, Clone)]
pub enum ParseH256Error {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl H256 {
    /// Returns the lower 26 bytes.
    pub fn lower_26_bytes(&self) -> [u8; 26] {
        let mut result = [0u8; 26];
        result.copy_from_slice(&self.0[6..32]);
        result
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl FromStr for H256 {
    type Err = ParseH256Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|e| ParseH256Error::InvalidHex(e))?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for H256 {
    type Error = ParseH256Error;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            Err(ParseH256Error::InvalidLength(slice.len()))
        } else {
            Ok(H256(copy_u8_array(&slice)))
        }
    }
}

impl From<&str> for H256 {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}

impl From<String> for H256 {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

impl Into<Vec<u8>> for H256 {
    fn into(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl fmt::Display for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "b177968c9c68877dc8d33e25759183c556379daa45a4d78a2b91c70133c873ca";
        let h = H256::from_str(s).unwrap();
        assert_eq!(h.to_string(), s);
    }
}
