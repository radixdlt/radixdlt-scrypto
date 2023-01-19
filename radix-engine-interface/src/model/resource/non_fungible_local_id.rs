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

/// Trait for converting into a `NonFungibleLocalId` of any kind (i.e. Integer, String, Bytes and UUID).
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
        NonFungibleIdType::Integer
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

/// Trait for converting into a `NonFungibleLocalId` of non-auto-generated kind (i.e. Integer, String and Bytes).
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
        Self::Integer(value)
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
    /// String of `[_0-9a-zA-Z]{1,64}`.
    String(String),
    /// Unsigned integers, up to u64.
    Integer(u64),
    /// Bytes, of length between 1 and 64.
    Bytes(Vec<u8>),
    /// UUID, v4, variant 1, big endian. See https://www.rfc-editor.org/rfc/rfc4122
    UUID(u128),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentValidationError {
    TooLong,
    Empty,
    ContainsBadCharacter(char),
    NotUuidV4Variant1,
}

impl NonFungibleLocalId {
    pub fn id_type(&self) -> NonFungibleIdType {
        match self {
            NonFungibleLocalId::String(..) => NonFungibleIdType::String,
            NonFungibleLocalId::Integer(..) => NonFungibleIdType::Integer,
            NonFungibleLocalId::Bytes(..) => NonFungibleIdType::Bytes,
            NonFungibleLocalId::UUID(..) => NonFungibleIdType::UUID,
        }
    }

    pub fn validate_contents(&self) -> Result<(), ContentValidationError> {
        match self {
            NonFungibleLocalId::String(value) => {
                if value.len() == 0 {
                    return Err(ContentValidationError::Empty);
                }
                if value.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
                    return Err(ContentValidationError::TooLong);
                }
                for char in value.chars() {
                    if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
                        return Err(ContentValidationError::ContainsBadCharacter(char));
                    }
                }
                Ok(())
            }
            NonFungibleLocalId::Bytes(value) => {
                if value.len() == 0 {
                    return Err(ContentValidationError::Empty);
                }
                if value.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
                    return Err(ContentValidationError::TooLong);
                }
                Ok(())
            }
            NonFungibleLocalId::Integer(_) => Ok(()),
            NonFungibleLocalId::UUID(v) => {
                // 0100 - v4
                // 10 - variant 1
                if (v & 0x00000000_0000_f000_C000_000000000000u128)
                    != 0x00000000_0000_4000_8000_000000000000u128
                {
                    return Err(ContentValidationError::NotUuidV4Variant1);
                }

                Ok(())
            }
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
    InvalidInteger,
    InvalidBytes,
    InvalidUUID,
    ContentValidationError(ContentValidationError),
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
            NonFungibleLocalId::Integer(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_be_bytes())?; // TODO: variable length encoding?
            }
            NonFungibleLocalId::Bytes(v) => {
                encoder.write_byte(2)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_slice())?;
            }
            NonFungibleLocalId::UUID(v) => {
                encoder.write_byte(3)?;
                encoder.write_slice(&v.to_be_bytes())?;
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
            1 => Self::Integer(u64::from_be_bytes(copy_u8_array(decoder.read_slice(8)?))),
            2 => {
                let size = decoder.read_size()?;
                Self::Bytes(decoder.read_slice(size)?.to_vec())
            }
            3 => Self::UUID(u128::from_be_bytes(copy_u8_array(decoder.read_slice(16)?))),
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
            NonFungibleLocalId::Integer(
                u64::from_str_radix(&s[1..s.len() - 1], 10)
                    .map_err(|_| ParseNonFungibleLocalIdError::InvalidInteger)?,
            )
        } else if s.starts_with("[") && s.ends_with("]") {
            NonFungibleLocalId::Bytes(
                hex::decode(&s[1..s.len() - 1])
                    .map_err(|_| ParseNonFungibleLocalIdError::InvalidBytes)?,
            )
        } else if s.starts_with("{") && s.ends_with("}") {
            let chars: Vec<char> = s[1..s.len() - 1].chars().collect();
            if chars.len() == 32 + 4
                && chars[8] == '-'
                && chars[13] == '-'
                && chars[18] == '-'
                && chars[23] == '-'
            {
                let hyphen_stripped: String = chars.into_iter().filter(|c| *c != '-').collect();
                NonFungibleLocalId::UUID(
                    u128::from_str_radix(&hyphen_stripped, 16)
                        .map_err(|_| ParseNonFungibleLocalIdError::InvalidUUID)?,
                )
            } else {
                return Err(ParseNonFungibleLocalIdError::InvalidUUID);
            }
        } else {
            return Err(ParseNonFungibleLocalIdError::UnknownType);
        };

        local_id
            .validate_contents()
            .map_err(ParseNonFungibleLocalIdError::ContentValidationError)?;

        Ok(local_id)
    }
}

impl fmt::Display for NonFungibleLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NonFungibleLocalId::String(v) => write!(f, "<{}>", v),
            NonFungibleLocalId::Integer(v) => write!(f, "#{}#", v),
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
        assert!(matches!(
            validation_result,
            Err(ContentValidationError::TooLong)
        ));
        let validation_result = NonFungibleLocalId::Bytes(vec![]).validate_contents();
        assert!(matches!(
            validation_result,
            Err(ContentValidationError::Empty)
        ));

        // String length
        let validation_result =
            NonFungibleLocalId::String(string_of_length(NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH))
                .validate_contents();
        assert!(matches!(validation_result, Ok(_)));
        let validation_result =
            NonFungibleLocalId::String(string_of_length(1 + NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH))
                .validate_contents();
        assert!(matches!(
            validation_result,
            Err(ContentValidationError::TooLong)
        ));
        let validation_result = NonFungibleLocalId::String("".to_string()).validate_contents();
        assert!(matches!(
            validation_result,
            Err(ContentValidationError::Empty)
        ));

        // UUIDv1
        let validation_result =
            NonFungibleLocalId::from_str("{baaa4d3e-97f6-11ed-a8fc-0242ac120002}");
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleLocalIdError::ContentValidationError(
                ContentValidationError::NotUuidV4Variant1
            ))
        ));

        // UUIDv4 variant 2
        let validation_result =
            NonFungibleLocalId::from_str("{a5942110-956f-4b51-d517-79366f501d25}");
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleLocalIdError::ContentValidationError(
                ContentValidationError::NotUuidV4Variant1
            ))
        ));
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
            Err(ContentValidationError::ContainsBadCharacter(char))
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            NonFungibleLocalId::from_str("#1#").unwrap(),
            NonFungibleLocalId::Integer(1)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#10#").unwrap(),
            NonFungibleLocalId::Integer(10)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("{b36f5b3f-835b-406c-980f-7788d8f13c1b}").unwrap(),
            NonFungibleLocalId::UUID(0xb36f5b3f_835b_406c_980f_7788d8f13c1b)
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
        assert_eq!(NonFungibleLocalId::Integer(1).to_string(), "#1#",);
        assert_eq!(NonFungibleLocalId::Integer(10).to_string(), "#10#",);
        assert_eq!(
            NonFungibleLocalId::UUID(0x0236805c_56e9_4431_a2a3_7d339db305c4).to_string(),
            "{0236805c-56e9-4431-a2a3-7d339db305c4}",
        );
        assert_eq!(
            NonFungibleLocalId::String("test".to_owned()).to_string(),
            "<test>"
        );
        assert_eq!(NonFungibleLocalId::Bytes(vec![1, 10]).to_string(), "[010a]");
    }
}
