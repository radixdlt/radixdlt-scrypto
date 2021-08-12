use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::utils::*;

/// Reference to a bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reference {
    Immutable(u32),

    Mutable(u32),
}

/// Represents an error when decoding a reference.
#[derive(Debug, Clone)]
pub enum DecodeReferenceError {
    InvalidHex(hex::FromHexError),
    InvalidLength,
    InvalidType(u8),
}

impl Reference {
    /// Decode Reference from its hex representation.
    pub fn from_hex(hex: &str) -> Result<Self, DecodeReferenceError> {
        let bytes = hex::decode(hex).map_err(|e| DecodeReferenceError::InvalidHex(e))?;

        Self::from_slice(&bytes)
    }

    /// Decode Reference from a slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DecodeReferenceError> {
        if bytes.len() == 1 + 4 {
            let kind = bytes[0];
            let data = &bytes[1..bytes.len()];
            match kind {
                0x00 => Ok(Reference::Immutable(u32::from_le_bytes(
                    copy_u8_array(data).unwrap(),
                ))),
                0x01 => Ok(Reference::Mutable(u32::from_le_bytes(
                    copy_u8_array(data).unwrap(),
                ))),
                _ => Err(DecodeReferenceError::InvalidType(kind)),
            }
        } else {
            Err(DecodeReferenceError::InvalidLength)
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

impl<T: AsRef<str>> From<T> for Reference {
    fn from(s: T) -> Self {
        Self::from_hex(s.as_ref()).unwrap()
    }
}

impl Into<Vec<u8>> for Reference {
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

impl ToString for Reference {
    fn to_string(&self) -> String {
        let buf: Vec<u8> = self.clone().into();
        hex::encode(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::string::ToString;
    use crate::*;

    #[test]
    fn test_from_to_string() {
        let s = "0011223344";
        let a: Reference = s.into();
        assert_eq!(a.to_string(), s);
    }
}
