use sbor::*;
use crate::args;
use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;

use crate::engine::{api::*, call_engine};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe)]
pub enum ResourceMethod {
    Mint,
    Burn,
    TakeFromVault,
    UpdateMetadata,
    UpdateNonFungibleData,
}

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceAddress(pub [u8; 26]);

impl ResourceAddress {}

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::Resource(self.0),
            function: "mint".to_string(),
            args: args![MintParams::Fungible { amount: amount.into() }]
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        let bucket: Bucket = scrypto_decode(&output.rtn).unwrap();
        bucket
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(&self, id: &NonFungibleId, data: T) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(id.clone(), (data.immutable_data(), data.mutable_data()));

        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::Resource(self.0),
            function: "mint".to_string(),
            args: args![MintParams::NonFungible { entries }]
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        let bucket: Bucket = scrypto_decode(&output.rtn).unwrap();
        bucket
    }

    /// Burns a bucket of resources.
    pub fn burn(&self, bucket: Bucket) {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::Bucket(bucket.0),
            function: "burn".to_string(),
            args: args![]
        };
        let _: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
    }

    /// Returns the resource type.
    pub fn resource_type(&self) -> ResourceType {
        let input = GetResourceTypeInput {
            resource_address: self.0,
        };
        let output: GetResourceTypeOutput = call_engine(GET_RESOURCE_TYPE, input);

        output.resource_type
    }

    /// Returns the metadata associated with this resource.
    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource_address: self.0,
        };
        let output: GetResourceMetadataOutput = call_engine(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    /// Returns the current supply of this resource.
    pub fn total_supply(&self) -> Decimal {
        let input = GetResourceTotalSupplyInput {
            resource_address: self.0,
        };
        let output: GetResourceTotalSupplyOutput = call_engine(GET_RESOURCE_TOTAL_SUPPLY, input);

        output.total_supply
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId) -> T {
        let input = GetNonFungibleDataInput {
            non_fungible_address: NonFungibleAddress::new(self.0, id.clone()),
        };
        let output: GetNonFungibleDataOutput = call_engine(GET_NON_FUNGIBLE_DATA, input);

        T::decode(&output.immutable_data, &output.mutable_data).unwrap()
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId, new_data: T) {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::Resource(self.0),
            function: "update_non_fungible_mutable_data".to_string(),
            args: args![id.clone(), new_data.mutable_data()]
        };
        let _: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
    }

    /// Checks if non-fungible unit, with certain key exists or not.
    ///
    pub fn non_fungible_exists(&self, id: &NonFungibleId) -> bool {
        let input = NonFungibleExistsInput {
            non_fungible_address: NonFungibleAddress::new(self.0, id.clone()),
        };
        let output: NonFungibleExistsOutput = call_engine(NON_FUNGIBLE_EXISTS, input);

        output.non_fungible_exists
    }

    /// Updates the resource metadata
    pub fn update_metadata(&self, new_metadata: HashMap<String, String>) {
        let input = UpdateResourceMetadataInput {
            resource_address: self.0,
            new_metadata,
        };
        let _output: UpdateResourceMetadataOutput = call_engine(UPDATE_RESOURCE_METADATA, input);
    }
}

//========
// error
//========

/// Represents an error when decoding resource address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResourceAddressError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseResourceAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseResourceAddressError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(ResourceAddress, ScryptoType::ResourceAddress, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for ResourceAddress {
    type Err = ParseResourceAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseResourceAddressError::InvalidHex(s.to_owned()))?;
        if bytes.get(0) != Some(&3u8) {
            return Err(ParseResourceAddressError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(3, &self.0)))
    }
}

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
