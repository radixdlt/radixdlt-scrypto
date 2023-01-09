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
use crate::Describe;

/// Represents a key for a non-fungible resource
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonFungibleId {
    Number(u64),
    UUID(u128),
    Bytes(Vec<u8>),
    String(String),
}

impl From<u64> for NonFungibleId {
    fn from(value: u64) -> Self {
        Self::Number(value)
    }
}

impl NonFungibleIdType for u64 {
    fn id() -> NonFungibleIdTypeId {
        NonFungibleIdTypeId::Number
    }
}

impl From<u128> for NonFungibleId {
    fn from(value: u128) -> Self {
        Self::UUID(value)
    }
}

impl NonFungibleIdType for u128 {
    fn id() -> NonFungibleIdTypeId {
        NonFungibleIdTypeId::UUID
    }
}

pub trait NonFungibleIdType: Into<NonFungibleId> {
    fn id() -> NonFungibleIdTypeId;
}

/// Represents type of non-fungible id
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Describe)]
pub enum NonFungibleIdTypeId {
    String,
    Number,
    Bytes,
    UUID,
}

pub const NON_FUNGIBLE_ID_MAX_LENGTH: usize = 64;

impl NonFungibleId {
    /// Returns non-fungible ID type.
    pub fn id_type(&self) -> NonFungibleIdTypeId {
        match self {
            NonFungibleId::Bytes(..) => NonFungibleIdTypeId::Bytes,
            NonFungibleId::String(..) => NonFungibleIdTypeId::String,
            NonFungibleId::Number(..) => NonFungibleIdTypeId::Number,
            NonFungibleId::UUID(..) => NonFungibleIdTypeId::UUID,
        }
    }

    /// Returns simple string representation of non-fungible ID value.
    pub fn to_simple_string(&self) -> String {
        match self {
            NonFungibleId::Bytes(b) => hex::encode(b),
            NonFungibleId::String(s) => s.clone(),
            NonFungibleId::Number(n) => format!("{}", n),
            NonFungibleId::UUID(u) => format!("{}", u),
        }
    }

    /// Converts simple string representation to non-fungible ID.
    pub fn try_from_simple_string(
        id_type: NonFungibleIdTypeId,
        s: &str,
    ) -> Result<Self, ParseNonFungibleIdError> {
        let non_fungible_id = match id_type {
            NonFungibleIdTypeId::Bytes => NonFungibleId::Bytes(hex::decode(s)?),
            NonFungibleIdTypeId::Number => NonFungibleId::Number(s.parse::<u64>()?),
            NonFungibleIdTypeId::String => NonFungibleId::String(s.to_string()),
            NonFungibleIdTypeId::UUID => NonFungibleId::UUID(s.parse::<u128>()?),
        };

        non_fungible_id.validate_contents()?;

        Ok(non_fungible_id)
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
    pub fn try_from_combined_simple_string(s: &str) -> Result<Self, ParseNonFungibleIdError> {
        let parts = s
            .splitn(2, '#')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>();

        if parts.len() != 2 {
            return Err(ParseNonFungibleIdError::RequiresTwoPartsSeparatedByHash);
        }

        let id_type = parts[0].parse::<NonFungibleIdTypeId>()?;
        Self::try_from_simple_string(id_type, parts[1])
    }

    pub fn validate_contents(&self) -> Result<(), ParseNonFungibleIdError> {
        match self {
            NonFungibleId::String(value) => {
                if value.len() == 0 {
                    return Err(ParseNonFungibleIdError::Empty);
                }
                if value.len() > NON_FUNGIBLE_ID_MAX_LENGTH {
                    return Err(ParseNonFungibleIdError::TooLong);
                }
                validate_non_fungible_id_string(value)?;
            }
            NonFungibleId::Bytes(value) => {
                if value.len() == 0 {
                    return Err(ParseNonFungibleIdError::Empty);
                }
                if value.len() > NON_FUNGIBLE_ID_MAX_LENGTH {
                    return Err(ParseNonFungibleIdError::TooLong);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn validate_non_fungible_id_string(string: &str) -> Result<(), ParseNonFungibleIdError> {
    for char in string.chars() {
        if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
            return Err(ParseNonFungibleIdError::InvalidCharacter(char));
        }
    }
    Ok(())
}

//========
// error
//========

/// Represents an error when decoding non-fungible id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleIdError {
    InvalidSbor(DecodeError),
    InvalidHex,
    InvalidInt(ParseIntError),
    InvalidIdType(ParseNonFungibleIdTypeError),
    CannotParseType,
    UnexpectedTypeId,
    TooLong,
    Empty,
    InvalidCharacter(char),
    RequiresTwoPartsSeparatedByHash,
}

impl From<DecodeError> for ParseNonFungibleIdError {
    fn from(err: DecodeError) -> Self {
        ParseNonFungibleIdError::InvalidSbor(err)
    }
}

impl From<FromHexError> for ParseNonFungibleIdError {
    fn from(_: FromHexError) -> Self {
        ParseNonFungibleIdError::InvalidHex
    }
}

impl From<ParseIntError> for ParseNonFungibleIdError {
    fn from(err: ParseIntError) -> Self {
        ParseNonFungibleIdError::InvalidInt(err)
    }
}

impl From<ParseNonFungibleIdTypeError> for ParseNonFungibleIdError {
    fn from(err: ParseNonFungibleIdTypeError) -> Self {
        ParseNonFungibleIdError::InvalidIdType(err)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TypeId<ScryptoCustomTypeId> for NonFungibleId {
    #[inline]
    fn type_id() -> SborTypeId<ScryptoCustomTypeId> {
        SborTypeId::Custom(ScryptoCustomTypeId::NonFungibleId)
    }
}

impl<E: Encoder<ScryptoCustomTypeId>> Encode<ScryptoCustomTypeId, E> for NonFungibleId {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            NonFungibleId::Number(v) => {
                encoder.write_byte(0)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            NonFungibleId::UUID(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            NonFungibleId::Bytes(v) => {
                encoder.write_byte(2)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_slice())?;
            }
            NonFungibleId::String(v) => {
                encoder.write_byte(3)?;
                encoder.write_size(v.len())?;
                encoder.write_slice(v.as_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ScryptoCustomTypeId>> Decode<ScryptoCustomTypeId, D> for NonFungibleId {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<ScryptoCustomTypeId>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let non_fungible_id = match decoder.read_byte()? {
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

        non_fungible_id
            .validate_contents()
            .map_err(|_| DecodeError::InvalidCustomValue)?;

        Ok(non_fungible_id)
    }
}

impl scrypto_abi::Describe for NonFungibleId {
    fn describe() -> scrypto_abi::Type {
        Type::NonFungibleId
    }
}

//======
// text
//======

impl fmt::Display for NonFungibleIdTypeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NonFungibleIdTypeId::Bytes => write!(f, "Bytes"),
            NonFungibleIdTypeId::Number => write!(f, "U64"),
            NonFungibleIdTypeId::String => write!(f, "String"),
            NonFungibleIdTypeId::UUID => write!(f, "UUID"),
        }
    }
}

impl fmt::Debug for NonFungibleIdTypeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleIdTypeError {
    UnknownType,
}

impl FromStr for NonFungibleIdTypeId {
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

impl fmt::Display for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_simple_string())
    }
}

impl fmt::Debug for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_combined_simple_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_fungible_id_type_and_display() {
        let nfid = NonFungibleId::Number(100);
        assert_eq!(nfid.id_type(), NonFungibleIdTypeId::Number);
        assert_eq!(nfid.to_string(), "100");

        let nfid = NonFungibleId::String(String::from("test"));
        assert_eq!(nfid.id_type(), NonFungibleIdTypeId::String);
        assert_eq!(nfid.to_string(), "test");

        let nfid = NonFungibleId::UUID(1_u128);
        assert_eq!(nfid.id_type(), NonFungibleIdTypeId::UUID);
        assert_eq!(nfid.to_string(), "1");

        let nfid = NonFungibleId::Bytes(vec![1, 2, 3, 255]);
        assert_eq!(nfid.id_type(), NonFungibleIdTypeId::Bytes);
        assert_eq!(nfid.to_string(), "010203ff");
    }

    #[test]
    fn test_non_fungible_length_validation() {
        // Bytes length
        let validation_result =
            NonFungibleId::Bytes([0; NON_FUNGIBLE_ID_MAX_LENGTH].to_vec()).validate_contents();
        assert!(matches!(validation_result, Ok(_)));
        let validation_result =
            NonFungibleId::Bytes([0; 1 + NON_FUNGIBLE_ID_MAX_LENGTH].to_vec()).validate_contents();
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleIdError::TooLong)
        ));
        let validation_result = NonFungibleId::Bytes(vec![]).validate_contents();
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleIdError::Empty)
        ));

        // String length
        let validation_result =
            NonFungibleId::String(string_of_length(NON_FUNGIBLE_ID_MAX_LENGTH)).validate_contents();
        assert!(matches!(validation_result, Ok(_)));
        let validation_result =
            NonFungibleId::String(string_of_length(1 + NON_FUNGIBLE_ID_MAX_LENGTH))
                .validate_contents();
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleIdError::TooLong)
        ));
        let validation_result = NonFungibleId::String("".to_string()).validate_contents();
        assert!(matches!(
            validation_result,
            Err(ParseNonFungibleIdError::Empty)
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
            NonFungibleId::String(valid_id_string.to_owned()).validate_contents();
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
            NonFungibleId::String(format!("valid_{}", char)).validate_contents();
        assert_eq!(
            validation_result,
            Err(ParseNonFungibleIdError::InvalidCharacter(char))
        );
    }

    #[test]
    fn test_non_fungible_id_simple_conversion() {
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdTypeId::Number, "1").unwrap(),
            NonFungibleId::Number(1)
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdTypeId::Number, "10").unwrap(),
            NonFungibleId::Number(10)
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdTypeId::UUID, "1234567890").unwrap(),
            NonFungibleId::UUID(1234567890)
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdTypeId::String, "test").unwrap(),
            NonFungibleId::String(String::from("test"))
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdTypeId::Bytes, "010a").unwrap(),
            NonFungibleId::Bytes(vec![1, 10])
        );
    }
}
