use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::utils::*;
use crate::H256;

/// Resource bucket id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BID {
    Transient(u32),

    Persisted(H256, u32),
}

/// Represents an error when decoding a BID.
#[derive(Debug, Clone)]
pub enum DecodeBIDError {
    InvalidHex(hex::FromHexError),
    InvalidLength,
    InvalidType(u8),
}

impl BID {
    /// Decode BID from its hex representation.
    pub fn from_hex(hex: &str) -> Result<Self, DecodeBIDError> {
        let bytes = hex::decode(hex).map_err(|e| DecodeBIDError::InvalidHex(e))?;

        Self::from_slice(&bytes)
    }

    /// Decode BID from a slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DecodeBIDError> {
        let invalid_len = DecodeBIDError::InvalidLength;

        if bytes.len() >= 1 {
            let kind = bytes[0];
            let data = &bytes[1..bytes.len()];
            match kind {
                0x00 => Ok(BID::Transient(u32::from_le_bytes(
                    copy_u8_array(data).map_err(|_| invalid_len)?,
                ))),
                0x01 => {
                    if data.len() == 32 + 4 {
                        Ok(BID::Persisted(
                            H256::from_slice(&data[0..32]).unwrap(),
                            u32::from_le_bytes(
                                copy_u8_array(&data[32..]).map_err(|_| invalid_len)?,
                            ),
                        ))
                    } else {
                        Err(invalid_len)
                    }
                }
                _ => Err(DecodeBIDError::InvalidType(kind)),
            }
        } else {
            Err(invalid_len)
        }
    }

    pub fn is_transient(&self) -> bool {
        match self {
            Self::Transient(_) => true,
            _ => false,
        }
    }

    pub fn is_persisted(&self) -> bool {
        !self.is_transient()
    }
}

impl<T: AsRef<str>> From<T> for BID {
    fn from(s: T) -> Self {
        Self::from_hex(s.as_ref()).unwrap()
    }
}

impl Into<Vec<u8>> for BID {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::Transient(index) => {
                buf.push(0u8);
                buf.extend(index.to_le_bytes());
            }
            Self::Persisted(hash, index) => {
                buf.push(1u8);
                buf.extend(hash.slice());
                buf.extend(index.to_le_bytes());
            }
        }
        buf
    }
}

impl ToString for BID {
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
        let s = "01f4cb57e4c4cd9d6564823eee427779d022d4f5f601791484a97837e6ffcf4cba01000000";
        let a: BID = s.into();
        assert_eq!(a.to_string(), s);
    }
}
