use crate::primitives::*;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;

/// Reference to a bucket.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum RID {
    Immutable(u32),

    Mutable(u32),
}

/// Represents an error when parsing RID.
#[derive(Debug, Clone)]
pub enum ParseRIDError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl RID {
    pub fn is_mutable(&self) -> bool {
        match self {
            Self::Mutable(_) => true,
            _ => false,
        }
    }

    pub fn is_immutable(&self) -> bool {
        !self.is_mutable()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Immutable(id) => combine2(0, &id.to_le_bytes()),
            Self::Mutable(id) => combine2(1, &id.to_le_bytes()),
        }
    }
}

impl FromStr for RID {
    type Err = ParseRIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|e| ParseRIDError::InvalidHex(e))?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for RID {
    type Error = ParseRIDError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match (slice.get(0), slice.len()) {
            (Some(0), 5) => Ok(RID::Immutable(u32::from_le_bytes(copy_u8_array(
                &slice[1..],
            )))),
            (Some(1), 5) => Ok(RID::Mutable(u32::from_le_bytes(copy_u8_array(&slice[1..])))),
            (_, len) => Err(ParseRIDError::InvalidLength(len)),
        }
    }
}

impl From<&str> for RID {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}

impl From<String> for RID {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

impl fmt::Debug for RID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Display for RID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "0100000001";
        let a = RID::from_str(s).unwrap();
        assert_eq!(a.to_string(), s);
    }
}
