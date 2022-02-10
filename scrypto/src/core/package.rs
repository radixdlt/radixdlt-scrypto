use sbor::{describe::Type, *};

use crate::engine::*;
use crate::misc::*;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::ToString;
use crate::types::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Package(pub [u8; 26]);

impl Package {
    /// Creates a new package.
    pub fn new(code: &[u8]) -> Self {
        let input = PublishPackageInput {
            code: code.to_vec(),
        };
        let output: PublishPackageOutput = call_engine(PUBLISH_PACKAGE, input);

        output.package
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParsePackageError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParsePackageError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParsePackageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Package {
    type Error = ParsePackageError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParsePackageError::InvalidLength(slice.len())),
        }
    }
}

impl Package {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(Package, CustomType::Package, Vec::new());

//======
// text
//======

impl FromStr for Package {
    type Err = ParsePackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParsePackageError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl ToString for Package {
    fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}
