use crate::crypto::blake2b_256_hash;
use radix_rust::copy_u8_array;
use sbor::rust::borrow::ToOwned;
use sbor::rust::convert::TryFrom;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

/// Represents a 32-byte hash digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Hash(pub [u8; Self::LENGTH]);

impl Hash {
    pub const LENGTH: usize = 32;

    pub fn lower_bytes<const N: usize>(&self) -> [u8; N] {
        self.0[(Self::LENGTH - N)..Self::LENGTH].try_into().unwrap()
    }
}

pub trait IsHash: AsRef<[u8]> + Sized + From<Hash> + Into<Hash> + AsRef<Hash> {
    fn as_bytes(&self) -> &[u8; Hash::LENGTH] {
        &<Self as AsRef<Hash>>::as_ref(self).0
    }

    fn as_slice(&self) -> &[u8] {
        &<Self as AsRef<Hash>>::as_ref(self).0
    }

    fn as_hash(&self) -> &Hash {
        &<Self as AsRef<Hash>>::as_ref(self)
    }

    fn into_bytes(self) -> [u8; Hash::LENGTH] {
        self.into_hash().0
    }

    fn into_hash(self) -> Hash {
        self.into()
    }

    fn from_bytes(bytes: [u8; Hash::LENGTH]) -> Self {
        Hash(bytes).into()
    }

    fn from_hash(hash: Hash) -> Self {
        hash.into()
    }
}

impl IsHash for Hash {}

impl AsRef<Hash> for Hash {
    fn as_ref(&self) -> &Hash {
        self
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
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ParseHashError {
    InvalidHex(String),
    InvalidLength { actual: usize, expected: usize },
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
            return Err(ParseHashError::InvalidLength {
                actual: slice.len(),
                expected: Hash::LENGTH,
            });
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

#[macro_export]
macro_rules! define_wrapped_hash {
    ($(#[$docs:meta])* $name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
        #[sbor(transparent)]
        $(#[$docs])*
        pub struct $name(pub Hash);

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl AsRef<Hash> for $name {
            fn as_ref(&self) -> &Hash {
                &self.0
            }
        }

        impl From<Hash> for $name {
            fn from(value: Hash) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Hash {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl IsHash for $name {}
    };
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
