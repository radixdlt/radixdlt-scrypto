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

use crate::abi::*;
use crate::data::*;
use crate::scrypto_type;
use crate::Describe;

/// Represents a key for a non-fungible resource
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonFungibleId {
    String(String),
    U32(u32),
    U64(u64),
    Bytes(Vec<u8>),
    UUID(u128),
}

/// Represents type of non-fungible id
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Describe, Default)]
pub enum NonFungibleIdType {
    String,
    U32,
    U64,
    Bytes,
    #[default]
    UUID,
}

pub const NON_FUNGIBLE_ID_MAX_LENGTH: usize = 64;

impl NonFungibleId {
    /// Returns non-fungible ID type.
    pub fn id_type(&self) -> NonFungibleIdType {
        match self {
            NonFungibleId::Bytes(..) => NonFungibleIdType::Bytes,
            NonFungibleId::String(..) => NonFungibleIdType::String,
            NonFungibleId::U32(..) => NonFungibleIdType::U32,
            NonFungibleId::U64(..) => NonFungibleIdType::U64,
            NonFungibleId::UUID(..) => NonFungibleIdType::UUID,
        }
    }

    /// Returns string representation of non-fungible ID value for transaction manifest.
    pub fn to_manifest_string(&self) -> String {
        match self {
            NonFungibleId::Bytes(b) => format!("Bytes(\"{}\")", hex::encode(b)),
            NonFungibleId::String(s) => format!("\"{}\"", s),
            NonFungibleId::U32(n) => format!("{}u32", n),
            NonFungibleId::U64(n) => format!("{}u64", n),
            NonFungibleId::UUID(u) => format!("{}u128", u),
        }
    }

    /// Converts transaction manifest representation string to non-fungible ID.
    pub fn try_from_manifest_string(s: &str) -> Result<Self, ParseNonFungibleIdError> {
        // TODO: improve this parser for properly handing edge cases
        let s = s.trim();
        let non_fungible_id = if s.len() > 9 && s.starts_with("Bytes(\"") && s.ends_with("\")") {
            NonFungibleId::Bytes(
                hex::decode(&s[7..s.len() - 2])?,
            )
        } else if s.len() > 4 && s.ends_with("u128") {
            NonFungibleId::UUID(
                s[0..s.len() - 4].parse::<u128>()?,
            )
        } else if s.len() > 3 && s.ends_with("u64") {
            NonFungibleId::U64(
                s[0..s.len() - 3].parse::<u64>()?,
            )
        } else if s.len() > 3 && s.ends_with("u32") {
            NonFungibleId::U32(
                s[0..s.len() - 3].parse::<u32>()?,
            )
        } else if s.len() > 2 && s.starts_with("\"") && s.ends_with("\"") {
            NonFungibleId::String(s[1..s.len() - 1].to_string())
        } else {
            return Err(ParseNonFungibleIdError::CannotParseType);
        };

        non_fungible_id.validate_contents()?;

        Ok(non_fungible_id)
    }

    /// Returns simple string representation of non-fungible ID value.
    pub fn to_simple_string(&self) -> String {
        match self {
            NonFungibleId::Bytes(b) => hex::encode(b),
            NonFungibleId::String(s) => s.clone(),
            NonFungibleId::U32(n) => format!("{}", n),
            NonFungibleId::U64(n) => format!("{}", n),
            NonFungibleId::UUID(u) => format!("{}", u),
        }
    }

    /// Converts simple string representation to non-fungible ID.
    pub fn try_from_simple_string(
        id_type: NonFungibleIdType,
        s: &str,
    ) -> Result<Self, ParseNonFungibleIdError> {
        let non_fungible_id = match id_type {
            NonFungibleIdType::Bytes => NonFungibleId::Bytes(hex::decode(s)?),
            NonFungibleIdType::U32 => NonFungibleId::U32(s.parse::<u32>()?),
            NonFungibleIdType::U64 => NonFungibleId::U64(s.parse::<u64>()?),
            NonFungibleIdType::String => NonFungibleId::String(s.to_string()),
            NonFungibleIdType::UUID => NonFungibleId::UUID(s.parse::<u128>()?),
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
    /// * `U32#122`
    /// * `U64#23`
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
    /// * `U32#122`
    /// * `U64#23`
    /// * `UUID#345` or `U128#567`
    pub fn try_from_combined_simple_string(
        s: &str,
    ) -> Result<Self, ParseNonFungibleIdError> {
        let parts = s
            .splitn(2, '#')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>();

        if parts.len() != 2 {
            return Err(ParseNonFungibleIdError::RequiresTwoPartsSeparatedByHash);
        }

        let id_type = parts[0].parse::<NonFungibleIdType>()?;
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

// Extract internal type id to optimize decoding process.
fn validate_id(slice: &[u8]) -> Result<SborTypeId<ScryptoCustomTypeId>, DecodeError> {
    let mut decoder = ScryptoDecoder::new(slice);
    decoder.read_and_check_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
    decoder.read_type_id()
}

impl TryFrom<&[u8]> for NonFungibleId {
    type Error = ParseNonFungibleIdError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let non_fungible_id = match validate_id(slice) {
            Ok(type_id) => match type_id {
                ScryptoSborTypeId::Array => NonFungibleId::Bytes(
                    scrypto_decode::<Vec<u8>>(slice)?,
                ),
                ScryptoSborTypeId::String => NonFungibleId::String(
                    scrypto_decode::<String>(slice)?
                ),
                ScryptoSborTypeId::U32 => NonFungibleId::U32(
                    scrypto_decode::<u32>(slice)?
                ),
                ScryptoSborTypeId::U64 => NonFungibleId::U64(
                    scrypto_decode::<u64>(slice)?
                ),
                ScryptoSborTypeId::U128 => NonFungibleId::UUID(
                    scrypto_decode::<u128>(slice)?
                ),
                _ => return Err(ParseNonFungibleIdError::UnexpectedTypeId),
            },
            Err(err) => return Err(err.into()),
        };

        non_fungible_id.validate_contents()?;

        Ok(non_fungible_id)
    }
}

impl NonFungibleId {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            NonFungibleId::Bytes(b) => scrypto_encode(&b).expect("Error encoding Byte array"),
            NonFungibleId::String(s) => scrypto_encode(&s).expect("Error encoding String"),
            NonFungibleId::U32(n) => scrypto_encode(&n).expect("Error encoding Number 32-bits"),
            NonFungibleId::U64(n) => scrypto_encode(&n).expect("Error encoding Number 64-bits"),
            NonFungibleId::UUID(u) => scrypto_encode(&u).expect("Error encoding UUID"),
        }
    }
}

scrypto_type!(
    NonFungibleId,
    ScryptoCustomTypeId::NonFungibleId,
    Type::NonFungibleId
);

//======
// text
//======

impl fmt::Display for NonFungibleIdType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NonFungibleIdType::Bytes => write!(f, "Bytes"),
            NonFungibleIdType::U32 => write!(f, "U32"),
            NonFungibleIdType::U64 => write!(f, "U64"),
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
            "U32" => Self::U32,
            "U64" => Self::U64,
            "Bytes" => Self::Bytes,
            "UUID" => Self::UUID,
            "U128" => Self::UUID, // Add this in as an alias
            _ => return Err(ParseNonFungibleIdTypeError::UnknownType)
        };
        Ok(id_type)
    }
}

impl fmt::Display for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NonFungibleId::Bytes(b) => write!(f, "Bytes(\"{}\")", hex::encode(b)),
            NonFungibleId::String(s) => write!(f, "\"{}\"", s),
            NonFungibleId::U32(n) => write!(f, "{}u32", n),
            NonFungibleId::U64(n) => write!(f, "{}u64", n),
            NonFungibleId::UUID(u) => write!(f, "{}u128", u),
        }
    }
}

impl fmt::Debug for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_fungible_id_type_and_display() {
        let nfid = NonFungibleId::U32(1);
        assert_eq!(nfid.id_type(), NonFungibleIdType::U32);
        assert_eq!(format!("{}", nfid), "1u32");

        let nfid = NonFungibleId::U64(100);
        assert_eq!(nfid.id_type(), NonFungibleIdType::U64);
        assert_eq!(format!("{}", nfid), "100u64");

        let nfid = NonFungibleId::String(String::from("test"));
        assert_eq!(nfid.id_type(), NonFungibleIdType::String);
        assert_eq!(format!("{}", nfid), "\"test\"");

        let nfid = NonFungibleId::UUID(1_u128);
        assert_eq!(nfid.id_type(), NonFungibleIdType::UUID);
        assert_eq!(format!("{}", nfid), "1u128");

        let nfid = NonFungibleId::Bytes(vec![1, 2, 3, 255]);
        assert_eq!(nfid.id_type(), NonFungibleIdType::Bytes);
        assert_eq!(format!("{}", nfid), "Bytes(\"010203ff\")");
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
    fn test_non_fungible_id_type_default() {
        assert_eq!(NonFungibleIdType::default(), NonFungibleIdType::UUID);
    }

    #[test]
    fn test_non_fungible_id_encode_decode() {
        let n = NonFungibleId::U32(1);
        let buf = n.to_vec();
        let val = NonFungibleId::try_from(buf.as_slice()).unwrap();
        assert_eq!(n.id_type(), val.id_type());
        assert!(matches!(val, NonFungibleId::U32(v) if v == 1));

        let n = NonFungibleId::U64(u64::MAX);
        let buf = n.to_vec();
        let val = NonFungibleId::try_from(buf.as_slice()).unwrap();
        assert_eq!(n.id_type(), val.id_type());
        assert!(matches!(val, NonFungibleId::U64(v) if v == u64::MAX));

        let n = NonFungibleId::UUID(u128::MAX);
        let buf = n.to_vec();
        let val = NonFungibleId::try_from(buf.as_slice()).unwrap();
        assert_eq!(n.id_type(), val.id_type());
        assert!(matches!(val, NonFungibleId::UUID(v) if v == u128::MAX));

        const TEST_STR: &str = "test_string_0123";
        let n = NonFungibleId::String(TEST_STR.to_string());
        let buf = n.to_vec();
        let val = NonFungibleId::try_from(buf.as_slice()).unwrap();
        assert_eq!(n.id_type(), val.id_type());
        assert!(matches!(val, NonFungibleId::String(s) if s == TEST_STR));

        let array: [u8; 5] = [1, 2, 3, 4, 5];
        let n = NonFungibleId::Bytes(array.to_vec());
        let buf = n.to_vec();
        let val = NonFungibleId::try_from(buf.as_slice()).unwrap();
        assert_eq!(n.id_type(), val.id_type());
        assert!(matches!(val, NonFungibleId::Bytes(b) if b == array));
    }

    #[test]
    fn test_non_fungible_id_bytes_rep() {
        // internal buffer representation:
        //   <sbor-v1 prefix: 5c><sbor type id><optional element id><optional size><bytes>
        assert_eq!(
            NonFungibleId::try_from(hex::decode("5c2007023575").unwrap().as_slice()).unwrap(),
            NonFungibleId::Bytes(vec![53u8, 117u8]),
        );
        assert_eq!(
            NonFungibleId::try_from(hex::decode("5c0905000000").unwrap().as_slice()).unwrap(),
            NonFungibleId::U32(5)
        );
        assert_eq!(
            NonFungibleId::try_from(hex::decode("5c0a0500000000000000").unwrap().as_slice()).unwrap(),
            NonFungibleId::U64(5)
        );
        assert_eq!(
            NonFungibleId::try_from(hex::decode("5c0b05000000000000000000000000000000").unwrap().as_slice()).unwrap(),
            NonFungibleId::UUID(5)
        );
        assert_eq!(
            NonFungibleId::try_from(hex::decode("5c0c0474657374").unwrap().as_slice()).unwrap(),
            NonFungibleId::String(String::from("test"))
        );
    }

    #[test]
    fn test_non_fungible_id_manifest_conversion() {
        assert_eq!(
            NonFungibleId::try_from_manifest_string("1u32").unwrap(),
            NonFungibleId::U32(1)
        );
        assert_eq!(
            NonFungibleId::try_from_manifest_string("10u64").unwrap(),
            NonFungibleId::U64(10)
        );
        assert_eq!(
            NonFungibleId::try_from_manifest_string("1234567890u128").unwrap(),
            NonFungibleId::UUID(1234567890)
        );
        assert_eq!(
            NonFungibleId::try_from_manifest_string("\"test\"").unwrap(),
            NonFungibleId::String(String::from("test"))
        );
        assert_eq!(
            NonFungibleId::try_from_manifest_string("Bytes(\"010a\")").unwrap(),
            NonFungibleId::Bytes(vec![1, 10])
        );
    }

    #[test]
    fn test_non_fungible_id_simple_conversion() {
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdType::U32, "1").unwrap(),
            NonFungibleId::U32(1)
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdType::U64, "10").unwrap(),
            NonFungibleId::U64(10)
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdType::UUID, "1234567890").unwrap(),
            NonFungibleId::UUID(1234567890)
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdType::String, "test").unwrap(),
            NonFungibleId::String(String::from("test"))
        );
        assert_eq!(
            NonFungibleId::try_from_simple_string(NonFungibleIdType::Bytes, "010a").unwrap(),
            NonFungibleId::Bytes(vec![1, 10])
        );
    }
}
