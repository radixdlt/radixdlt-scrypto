use core::num::ParseIntError;

use hex::FromHexError;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::abi::*;
use crate::data::*;
use crate::*;

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

//====================================
// Rust types => NonFungibleLocalId
//====================================

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

/// Represents a key for a non-fungible resource
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonFungibleLocalId {
    Number(u64),
    UUID(u128),
    Bytes(Vec<u8>),
    String(String),
}

/// Represents type of non-fungible id
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, LegacyDescribe)]
pub enum NonFungibleIdType {
    String,
    Number,
    Bytes,
    UUID,
}

pub const NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH: usize = 64;

impl NonFungibleLocalId {
    /// Returns non-fungible ID type.
    pub fn id_type(&self) -> NonFungibleIdType {
        match self {
            NonFungibleLocalId::Bytes(..) => NonFungibleIdType::Bytes,
            NonFungibleLocalId::String(..) => NonFungibleIdType::String,
            NonFungibleLocalId::Number(..) => NonFungibleIdType::Number,
            NonFungibleLocalId::UUID(..) => NonFungibleIdType::UUID,
        }
    }

    /// Returns simple string representation of non-fungible ID value.
    pub fn to_simple_string(&self) -> String {
        match self {
            NonFungibleLocalId::Bytes(b) => hex::encode(b),
            NonFungibleLocalId::String(s) => s.clone(),
            NonFungibleLocalId::Number(n) => format!("{}", n),
            NonFungibleLocalId::UUID(u) => format!("{}", u),
        }
    }

    /// Converts simple string representation to non-fungible ID.
    pub fn try_from_simple_string(
        id_type: NonFungibleIdType,
        s: &str,
    ) -> Result<Self, ParseNonFungibleLocalIdError> {
        let non_fungible_local_id = match id_type {
            NonFungibleIdType::Bytes => NonFungibleLocalId::Bytes(hex::decode(s)?),
            NonFungibleIdType::Number => NonFungibleLocalId::Number(s.parse::<u64>()?),
            NonFungibleIdType::String => NonFungibleLocalId::String(s.to_string()),
            NonFungibleIdType::UUID => NonFungibleLocalId::UUID(s.parse::<u128>()?),
        };

        non_fungible_local_id.validate_contents()?;

        Ok(non_fungible_local_id)
    }

    /// Returns the simple string representation of non-fungible ID value.
    ///
    /// You should generally prefer the simple string representation without the type information,
    /// unless the type information cannot be located.
    ///
    /// This representation looks like:
    /// * `String#abc`
    /// * `Bytes#23ae33`
    /// * `Number#23`
    /// * `UUID#345`
    pub fn to_combined_simple_string(&self) -> String {
        format!("{}#{}", self.id_type(), self.to_simple_string())
    }

    /// Converts combined simple string representation to non-fungible ID.
    ///
    /// You should generally prefer the simple string representation without the type information,
    /// unless the type information cannot be located.
    ///
    /// This accepts the following:
    /// * `String#abc`
    /// * `Bytes#23ae33`
    /// * `Number#23`
    /// * `UUID#345` or `U128#567`
    pub fn try_from_combined_simple_string(s: &str) -> Result<Self, ParseNonFungibleLocalIdError> {
        let parts = s
            .splitn(2, '#')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>();

        if parts.len() != 2 {
            return Err(ParseNonFungibleLocalIdError::RequiresTwoPartsSeparatedByHash);
        }

        let id_type = parts[0].parse::<NonFungibleIdType>()?;
        Self::try_from_simple_string(id_type, parts[1])
    }

    pub fn validate_contents(&self) -> Result<(), ParseNonFungibleLocalIdError> {
        match self {
            NonFungibleLocalId::String(value) => {
                if value.len() == 0 {
                    return Err(ParseNonFungibleLocalIdError::Empty);
                }
                if value.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
                    return Err(ParseNonFungibleLocalIdError::TooLong);
                }
                validate_non_fungible_local_id_string(value)?;
            }
            NonFungibleLocalId::Bytes(value) => {
                if value.len() == 0 {
                    return Err(ParseNonFungibleLocalIdError::Empty);
                }
                if value.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
                    return Err(ParseNonFungibleLocalIdError::TooLong);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn validate_non_fungible_local_id_string(string: &str) -> Result<(), ParseNonFungibleLocalIdError> {
    for char in string.chars() {
        if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
            return Err(ParseNonFungibleLocalIdError::InvalidCharacter(char));
        }
    }
    Ok(())
}

//========
// error
//========

/// Represents an error when decoding non-fungible id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleLocalIdError {
    InvalidSbor(DecodeError),
    InvalidHex,
    InvalidInt(ParseIntError),
    InvalidIdType(ParseNonFungibleIdTypeError),
    CannotParseType,
    UnexpectedValueKind,
    TooLong,
    Empty,
    InvalidCharacter(char),
    RequiresTwoPartsSeparatedByHash,
}

impl From<DecodeError> for ParseNonFungibleLocalIdError {
    fn from(err: DecodeError) -> Self {
        ParseNonFungibleLocalIdError::InvalidSbor(err)
    }
}

impl From<FromHexError> for ParseNonFungibleLocalIdError {
    fn from(_: FromHexError) -> Self {
        ParseNonFungibleLocalIdError::InvalidHex
    }
}

impl From<ParseIntError> for ParseNonFungibleLocalIdError {
    fn from(err: ParseIntError) -> Self {
        ParseNonFungibleLocalIdError::InvalidInt(err)
    }
}

impl From<ParseNonFungibleIdTypeError> for ParseNonFungibleLocalIdError {
    fn from(err: ParseNonFungibleIdTypeError) -> Self {
        ParseNonFungibleLocalIdError::InvalidIdType(err)
    }
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
            NonFungibleLocalId::Number(v) => {
                encoder.write_byte(0)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            NonFungibleLocalId::UUID(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            NonFungibleLocalId::Bytes(v) => {
                encoder.write_byte(2)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_slice())?;
            }
            NonFungibleLocalId::String(v) => {
                encoder.write_byte(3)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_bytes())?;
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
        let non_fungible_local_id = match decoder.read_byte()? {
            0 => Self::Number(u64::from_le_bytes(copy_u8_array(decoder.read_slice(8)?))),
            1 => Self::UUID(u128::from_le_bytes(copy_u8_array(decoder.read_slice(16)?))),
            2 => {
                let size = decoder.read_size()?;
                Self::Bytes(decoder.read_slice(size)?.to_vec())
            }
            3 => {
                let size = decoder.read_size()?;
                Self::String(
                    String::from_utf8(decoder.read_slice(size)?.to_vec())
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                )
            }
            _ => return Err(DecodeError::InvalidCustomValue),
        };

        non_fungible_local_id
            .validate_contents()
            .map_err(|_| DecodeError::InvalidCustomValue)?;

        Ok(non_fungible_local_id)
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

impl fmt::Display for NonFungibleIdType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NonFungibleIdType::Bytes => write!(f, "Bytes"),
            NonFungibleIdType::Number => write!(f, "U64"),
            NonFungibleIdType::String => write!(f, "String"),
            NonFungibleIdType::UUID => write!(f, "UUID"),
        }
    }
}

impl fmt::Debug for NonFungibleIdType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleIdTypeError {
    UnknownType,
}

impl FromStr for NonFungibleIdType {
    type Err = ParseNonFungibleIdTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id_type = match s {
            "String" => Self::String,
            "U64" => Self::Number,
            "Bytes" => Self::Bytes,
            "UUID" => Self::UUID,
            "U128" => Self::UUID, // Add this in as an alias
            _ => return Err(ParseNonFungibleIdTypeError::UnknownType),
        };
        Ok(id_type)
    }
}

impl fmt::Display for NonFungibleLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_simple_string())
    }
}

impl fmt::Debug for NonFungibleLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_combined_simple_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_fungible_local_id_type_and_display() {
        let non_fungible_local_id = NonFungibleLocalId::Number(100);
        assert_eq!(non_fungible_local_id.id_type(), NonFungibleIdType::Number);
        assert_eq!(non_fungible_local_id.to_string(), "100");

        let non_fungible_local_id = NonFungibleLocalId::String(String::from("test"));
        assert_eq!(non_fungible_local_id.id_type(), NonFungibleIdType::String);
        assert_eq!(non_fungible_local_id.to_string(), "test");

        let non_fungible_local_id = NonFungibleLocalId::UUID(1_u128);
        assert_eq!(non_fungible_local_id.id_type(), NonFungibleIdType::UUID);
        assert_eq!(non_fungible_local_id.to_string(), "1");

        let non_fungible_local_id = NonFungibleLocalId::Bytes(vec![1, 2, 3, 255]);
        assert_eq!(non_fungible_local_id.id_type(), NonFungibleIdType::Bytes);
        assert_eq!(non_fungible_local_id.to_string(), "010203ff");
    }

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
            Err(ParseNonFungibleLocalIdError::TooLong)
        ));
        let validation_result = NonFungibleLocalId::Bytes(vec![]).validate_contents();
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleLocalIdError::Empty)
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
            Err(ParseNonFungibleLocalIdError::TooLong)
        ));
        let validation_result = NonFungibleLocalId::String("".to_string()).validate_contents();
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleLocalIdError::Empty)
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
            Err(ParseNonFungibleLocalIdError::InvalidCharacter(char))
        );
    }

    #[test]
    fn test_non_fungible_local_id_simple_conversion() {
        assert_eq!(
            NonFungibleLocalId::try_from_simple_string(NonFungibleIdType::Number, "1").unwrap(),
            NonFungibleLocalId::Number(1)
        );
        assert_eq!(
            NonFungibleLocalId::try_from_simple_string(NonFungibleIdType::Number, "10").unwrap(),
            NonFungibleLocalId::Number(10)
        );
        assert_eq!(
            NonFungibleLocalId::try_from_simple_string(NonFungibleIdType::UUID, "1234567890")
                .unwrap(),
            NonFungibleLocalId::UUID(1234567890)
        );
        assert_eq!(
            NonFungibleLocalId::try_from_simple_string(NonFungibleIdType::String, "test").unwrap(),
            NonFungibleLocalId::String(String::from("test"))
        );
        assert_eq!(
            NonFungibleLocalId::try_from_simple_string(NonFungibleIdType::Bytes, "010a").unwrap(),
            NonFungibleLocalId::Bytes(vec![1, 10])
        );
    }
}
