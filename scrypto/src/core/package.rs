use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::engine::{api::*, call_engine, types::PackageId};
use crate::misc::*;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::ToString;
use crate::types::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackageRef(pub PackageId);

impl PackageRef {
    /// Creates a new package.
    pub fn new(code: &[u8]) -> Self {
        let input = PublishPackageInput {
            code: code.to_vec(),
        };
        let output: PublishPackageOutput = call_engine(PUBLISH_PACKAGE, input);

        Self(output.package_id)
    }

    /// Invokes a function on this blueprint.
    pub fn call<T: Decode, S: AsRef<str>>(
        &self,
        blueprint: S,
        function: S,
        args: Vec<Vec<u8>>,
    ) -> T {
        let output = Context::call_function((*self, blueprint), function, args);

        scrypto_decode(&output).unwrap()
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParsePackageRefError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParsePackageRefError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParsePackageRefError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for PackageRef {
    type Error = ParsePackageRefError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParsePackageRefError::InvalidLength(slice.len())),
        }
    }
}

impl PackageRef {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(PackageRef, CustomType::PackageRef, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for PackageRef {
    type Err = ParsePackageRefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParsePackageRefError::InvalidHex)?;
        Self::try_from(&bytes[1..])
    }
}

impl ToString for PackageRef {
    fn to_string(&self) -> String {
        hex::encode(combine(1, &self.0))
    }
}
