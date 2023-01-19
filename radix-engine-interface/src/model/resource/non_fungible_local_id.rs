use crate::abi::*;
use crate::data::*;
use crate::model::NonFungibleIdType;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

pub const NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH: usize = 64;

/// Trait for converting into a `NonFungibleLocalId` of any kind (i.e. Number, String, Bytes and UUID).
pub trait IntoNonFungibleLocalId: Into<NonFungibleLocalId> {
    fn id_type() -> NonFungibleIdType;
}

impl IntoNonFungibleLocalId for String {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::String
    }
}
impl IntoNonFungibleLocalId for u64 {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::Number
    }
}
impl IntoNonFungibleLocalId for Vec<u8> {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::Bytes
    }
}
impl IntoNonFungibleLocalId for u128 {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::UUID
    }
}

/// Trait for converting into a `NonFungibleLocalId` of non-auto-generated kind (i.e. Number, String and Bytes).
pub trait IntoManualNonFungibleLocalId: IntoNonFungibleLocalId {}

impl IntoManualNonFungibleLocalId for String {}
impl IntoManualNonFungibleLocalId for u64 {}
impl IntoManualNonFungibleLocalId for Vec<u8> {}

impl From<String> for NonFungibleLocalId {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<u64> for NonFungibleLocalId {
    fn from(value: u64) -> Self {
        Self::Number(value)
    }
}
impl From<Vec<u8>> for NonFungibleLocalId {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}
impl From<u128> for NonFungibleLocalId {
    fn from(value: u128) -> Self {
        Self::UUID(value)
    }
}

/// Represents the local id of a non-fungible.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonFungibleLocalId {
    String(String),
    Number(u64),
    Bytes(Vec<u8>),
    UUID(u128),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdValidationError {
    TooLong,
    Empty,
    ContainsBadCharacter(char),
}

impl NonFungibleLocalId {
    pub fn id_type(&self) -> NonFungibleIdType {
        match self {
            NonFungibleLocalId::String(..) => NonFungibleIdType::String,
            NonFungibleLocalId::Number(..) => NonFungibleIdType::Number,
            NonFungibleLocalId::Bytes(..) => NonFungibleIdType::Bytes,
            NonFungibleLocalId::UUID(..) => NonFungibleIdType::UUID,
        }
    }

    pub fn validate_contents(&self) -> Result<(), IdValidationError> {
        match self {
            NonFungibleLocalId::String(value) => {
                if value.len() == 0 {
                    return Err(IdValidationError::Empty);
                }
                if value.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
                    return Err(IdValidationError::TooLong);
                }
                for char in value.chars() {
                    if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
                        return Err(IdValidationError::ContainsBadCharacter(char));
                    }
                }
                Ok(())
            }
            NonFungibleLocalId::Bytes(value) => {
                if value.len() == 0 {
                    return Err(IdValidationError::Empty);
                }
                if value.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
                    return Err(IdValidationError::TooLong);
                }
                Ok(())
            }
            NonFungibleLocalId::Number(_) | NonFungibleLocalId::UUID(_) => Ok(()),
        }
    }
}

//========
// error
//========

/// Represents an error when decoding non-fungible id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleLocalIdError {
    UnknownType,
    InvalidNumber,
    InvalidBytes,
    InvalidUUID,
    IdValidationError(IdValidationError),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleLocalIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleLocalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for NonFungibleLocalId {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for NonFungibleLocalId {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            NonFungibleLocalId::String(v) => {
                encoder.write_byte(0)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_bytes())?;
            }
            NonFungibleLocalId::Number(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            NonFungibleLocalId::Bytes(v) => {
                encoder.write_byte(2)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_slice())?;
            }
            NonFungibleLocalId::UUID(v) => {
                encoder.write_byte(3)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for NonFungibleLocalId {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let local_id = match decoder.read_byte()? {
            0 => {
                let size = decoder.read_size()?;
                Self::String(
                    String::from_utf8(decoder.read_slice(size)?.to_vec())
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                )
            }
            1 => Self::Number(u64::from_le_bytes(copy_u8_array(decoder.read_slice(8)?))),
            2 => {
                let size = decoder.read_size()?;
                Self::Bytes(decoder.read_slice(size)?.to_vec())
            }
            3 => Self::UUID(u128::from_le_bytes(copy_u8_array(decoder.read_slice(16)?))),
            _ => return Err(DecodeError::InvalidCustomValue),
        };

        local_id
            .validate_contents()
            .map_err(|_| DecodeError::InvalidCustomValue)?;

        Ok(local_id)
    }
}

impl scrypto_abi::LegacyDescribe for NonFungibleLocalId {
    fn describe() -> scrypto_abi::Type {
        Type::NonFungibleLocalId
    }
}

//======
// text
//======

impl FromStr for NonFungibleLocalId {
    type Err = ParseNonFungibleLocalIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let local_id = if s.starts_with("<") && s.ends_with(">") {
            NonFungibleLocalId::String(s[1..s.len() - 1].to_string())
        } else if s.starts_with("#") && s.ends_with("#") {
            NonFungibleLocalId::Number(
                u64::from_str_radix(&s[1..s.len() - 1], 10)
                    .map_err(|_| ParseNonFungibleLocalIdError::InvalidNumber)?,
            )
        } else if s.starts_with("[") && s.ends_with("]") {
            NonFungibleLocalId::Bytes(
                hex::decode(&s[1..s.len() - 1])
                    .map_err(|_| ParseNonFungibleLocalIdError::InvalidBytes)?,
            )
        } else if s.starts_with("{") && s.ends_with("}") {
            let hex: String = s[1..s.len() - 1]
                .chars()
                .into_iter()
                .filter(|c| *c != '-')
                .collect();
            NonFungibleLocalId::UUID(
                u128::from_str_radix(&hex, 16)
                    .map_err(|_| ParseNonFungibleLocalIdError::InvalidUUID)?,
            )
        } else {
            return Err(ParseNonFungibleLocalIdError::UnknownType);
        };

        local_id
            .validate_contents()
            .map_err(ParseNonFungibleLocalIdError::IdValidationError)?;

        Ok(local_id)
    }
}

impl fmt::Display for NonFungibleLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NonFungibleLocalId::String(v) => write!(f, "<{}>", v),
            NonFungibleLocalId::Number(v) => write!(f, "#{}#", v),
            NonFungibleLocalId::Bytes(v) => write!(f, "[{}]", hex::encode(&v)),
            NonFungibleLocalId::UUID(v) => {
                let hex = format!("{:032x}", v);
                write!(
                    f,
                    "{{{}-{}-{}-{}-{}}}",
                    &hex[0..8],
                    &hex[8..12],
                    &hex[12..16],
                    &hex[16..20],
                    &hex[20..32]
                )
            }
        }
    }
}

impl fmt::Debug for NonFungibleLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_fungible_length_validation() {
        // Bytes length
        let validation_result =
            NonFungibleLocalId::Bytes([0; NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH].to_vec())
                .validate_contents();
        assert!(matches!(validation_result, Ok(_)));
        let validation_result =
            NonFungibleLocalId::Bytes([0; 1 + NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH].to_vec())
                .validate_contents();
        assert!(matches!(validation_result, Err(IdValidationError::TooLong)));
        let validation_result = NonFungibleLocalId::Bytes(vec![]).validate_contents();
        assert!(matches!(validation_result, Err(IdValidationError::Empty)));

        // String length
        let validation_result =
            NonFungibleLocalId::String(string_of_length(NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH))
                .validate_contents();
        assert!(matches!(validation_result, Ok(_)));
        let validation_result =
            NonFungibleLocalId::String(string_of_length(1 + NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH))
                .validate_contents();
        assert!(matches!(validation_result, Err(IdValidationError::TooLong)));
        let validation_result = NonFungibleLocalId::String("".to_string()).validate_contents();
        assert!(matches!(validation_result, Err(IdValidationError::Empty)));
    }

    fn string_of_length(size: usize) -> String {
        let mut str_buf = String::new();
        for _ in 0..size {
            str_buf.push('a');
        }
        str_buf
    }

    #[test]
    fn test_non_fungible_string_validation() {
        let valid_id_string = "abcdefghijklmnopqrstuvwxyz_ABCDEFGHIJKLMNOPQRSTUVWZYZ_0123456789";
        let validation_result =
            NonFungibleLocalId::String(valid_id_string.to_owned()).validate_contents();
        assert!(matches!(validation_result, Ok(_)));

        test_invalid_char('.');
        test_invalid_char('`');
        test_invalid_char('\\');
        test_invalid_char('"');
        test_invalid_char(' ');
        test_invalid_char('\r');
        test_invalid_char('\n');
        test_invalid_char('\t');
        test_invalid_char('\u{0000}'); // Null
        test_invalid_char('\u{0301}'); // Combining acute accent
        test_invalid_char('\u{2764}'); // ❤
        test_invalid_char('\u{000C}'); // Form feed
        test_invalid_char('\u{202D}'); // LTR override
        test_invalid_char('\u{202E}'); // RTL override
        test_invalid_char('\u{1F600}'); // :-) emoji
    }

    fn test_invalid_char(char: char) {
        let validation_result =
            NonFungibleLocalId::String(format!("valid_{}", char)).validate_contents();
        assert_eq!(
            validation_result,
            Err(IdValidationError::ContainsBadCharacter(char))
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            NonFungibleLocalId::from_str("#1#").unwrap(),
            NonFungibleLocalId::Number(1)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#10#").unwrap(),
            NonFungibleLocalId::Number(10)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("{1234567890}").unwrap(),
            NonFungibleLocalId::UUID(0x1234567890)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("<test>").unwrap(),
            NonFungibleLocalId::String("test".to_owned())
        );
        assert_eq!(
            NonFungibleLocalId::from_str("[010a]").unwrap(),
            NonFungibleLocalId::Bytes(vec![1, 10])
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(NonFungibleLocalId::Number(1).to_string(), "#1#",);
        assert_eq!(NonFungibleLocalId::Number(10).to_string(), "#10#",);
        assert_eq!(
            NonFungibleLocalId::UUID(0x1234567890).to_string(),
            "{00000000-0000-0000-0000-001234567890}",
        );
        assert_eq!(
            NonFungibleLocalId::String("test".to_owned()).to_string(),
            "<test>"
        );
        assert_eq!(NonFungibleLocalId::Bytes(vec![1, 10]).to_string(), "[010a]");
    }
}
