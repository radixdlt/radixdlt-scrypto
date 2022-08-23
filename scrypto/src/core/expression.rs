use sbor::rust::borrow::ToOwned;
use sbor::rust::convert::TryFrom;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Expression(pub String);

//========
// error
//========

/// Represents an error when parsing Expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseExpressionError {
    InvalidUtf8,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseExpressionError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Expression {
    type Error = ParseExpressionError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(
            String::from_utf8(slice.to_vec()).map_err(|_| Self::Error::InvalidUtf8)?,
        ))
    }
}

impl Expression {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

scrypto_type!(Expression, ScryptoType::Expression, Vec::new());

//======
// text
//======

impl FromStr for Expression {
    type Err = ParseExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::format;
    use sbor::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let name = "hello";
        let s = Expression::from_str(name).unwrap();
        assert_eq!(s.to_string(), name);
        assert_eq!(format!("{:?}", s), name);
        assert_eq!(format!("{}", s), name);
    }

    #[test]
    fn test_from_to_bytes() {
        let s = Expression("hello".to_owned());
        let s2 = Expression::try_from(s.to_vec().as_slice()).unwrap();
        assert_eq!(s2, s);
    }
}
