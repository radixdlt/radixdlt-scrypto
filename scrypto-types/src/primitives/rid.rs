use crate::primitives::*;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;

/// Reference to a bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RID {
    Immutable(u32),

    Mutable(u32),
}

/// Represents an error when decoding a reference.
#[derive(Debug, Clone)]
pub enum DecodeRIDError {
    InvalidHex(hex::FromHexError),
    InvalidLength,
    InvalidType(u8),
}

impl RID {
    /// Decode Reference from its hex representation.
    pub fn from_hex(hex: &str) -> Result<Self, DecodeRIDError> {
        let bytes = hex::decode(hex).map_err(|e| DecodeRIDError::InvalidHex(e))?;

        Self::from_slice(&bytes)
    }

    /// Decode Reference from a slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DecodeRIDError> {
        if bytes.len() == 1 + 4 {
            let kind = bytes[0];
            let data = &bytes[1..bytes.len()];
            match kind {
                0x00 => Ok(RID::Immutable(u32::from_le_bytes(
                    copy_u8_array(data).unwrap(),
                ))),
                0x01 => Ok(RID::Mutable(u32::from_le_bytes(
                    copy_u8_array(data).unwrap(),
                ))),
                _ => Err(DecodeRIDError::InvalidType(kind)),
            }
        } else {
            Err(DecodeRIDError::InvalidLength)
        }
    }

    pub fn is_immutable(&self) -> bool {
        !self.is_mutable()
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Self::Mutable(_) => true,
            _ => false,
        }
    }
}

impl<T: AsRef<str>> From<T> for RID {
    fn from(s: T) -> Self {
        Self::from_hex(s.as_ref()).unwrap()
    }
}

impl Into<Vec<u8>> for RID {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::Immutable(id) => {
                buf.push(0u8);
                buf.extend(id.to_le_bytes());
            }
            Self::Mutable(id) => {
                buf.push(1u8);
                buf.extend(id.to_le_bytes());
            }
        }
        buf
    }
}

impl ToString for RID {
    fn to_string(&self) -> String {
        let buf: Vec<u8> = self.clone().into();
        hex::encode(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "0011223344";
        let a: RID = s.into();
        assert_eq!(a.to_string(), s);
    }
}
