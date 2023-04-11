use crate::crypto::blake2b_256_hash;
use sbor::rust::borrow::ToOwned;
use sbor::rust::convert::TryFrom;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

/// Represents a 32-byte hash digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Hash(pub [u8; Self::LENGTH]);

impl Hash {
    pub const LENGTH: usize = 32;

    /// Returns the lower 27 bytes.
    pub fn lower_27_bytes(&self) -> [u8; 27] {
        let mut result = [0u8; 27];
        result.copy_from_slice(&self.0[5..32]);
        result
    }

    /// Returns the lower 26 bytes.
    pub fn lower_26_bytes(&self) -> [u8; 26] {
        let mut result = [0u8; 26];
        result.copy_from_slice(&self.0[6..32]);
        result
    }

    /// Returns the lower 16 bytes.
    pub fn lower_16_bytes(&self) -> [u8; 16] {
        let mut result = [0u8; 16];
        result.copy_from_slice(&self.0[16..32]);
        result
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Computes the hash digest of a message.
pub fn hash<T: AsRef<[u8]>>(data: T) -> Hash {
    blake2b_256_hash(data)
}

//========
// error
//========

/// Represents an error when parsing hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseHashError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseHashError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseHashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Hash {
    type Error = ParseHashError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Hash::LENGTH {
            return Err(ParseHashError::InvalidLength(slice.len()));
        }
        Ok(Self(copy_u8_array(slice)))
    }
}

impl From<Hash> for Vec<u8> {
    fn from(value: Hash) -> Self {
        value.to_vec()
    }
}

impl Hash {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

//======
// text
//======

impl FromStr for Hash {
    type Err = ParseHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseHashError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "b177968c9c68877dc8d33e25759183c556379daa45a4d78a2b91c70133c873ca";
        let h = Hash::from_str(s).unwrap();
        assert_eq!(h.to_string(), s);
    }
}
