use sbor::{describe::Type, *};

use crate::engine::{api::*, call_engine};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a resource definition.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceDefRef(pub [u8; 26]);

impl ResourceDefRef {
    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&mut self, amount: T, auth: BucketRef) -> Bucket {
        let input = MintResourceInput {
            resource_def_ref: *self,
            new_supply: Supply::Fungible {
                amount: amount.into(),
            },
            auth: auth.0,
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        Bucket(output.bucket_id)
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(
        &mut self,
        key: &NonFungibleKey,
        data: T,
        auth: BucketRef,
    ) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(key.clone(), (data.immutable_data(), data.mutable_data()));

        let input = MintResourceInput {
            resource_def_ref: *self,
            new_supply: Supply::NonFungible { entries },
            auth: auth.0,
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        Bucket(output.bucket_id)
    }

    /// Burns a bucket of resources.
    pub fn burn(&mut self, bucket: Bucket) {
        let input = BurnResourceInput {
            bucket_id: bucket.0,
            auth: None,
        };
        let _output: BurnResourceOutput = call_engine(BURN_RESOURCE, input);
    }

    /// Burns a bucket of resources.
    pub fn burn_with_auth(&mut self, bucket: Bucket, auth: BucketRef) {
        let input = BurnResourceInput {
            bucket_id: bucket.0,
            auth: Some(auth.0),
        };
        let _output: BurnResourceOutput = call_engine(BURN_RESOURCE, input);
    }

    /// Returns the resource type.
    pub fn resource_type(&self) -> ResourceType {
        let input = GetResourceTypeInput {
            resource_def_ref: *self,
        };
        let output: GetResourceTypeOutput = call_engine(GET_RESOURCE_TYPE, input);

        output.resource_type
    }

    /// Returns the metadata associated with this resource.
    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource_def_ref: *self,
        };
        let output: GetResourceMetadataOutput = call_engine(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    /// Returns the feature flags.
    pub fn flags(&self) -> u64 {
        let input = GetResourceFlagsInput {
            resource_def_ref: *self,
        };
        let output: GetResourceFlagsOutput = call_engine(GET_RESOURCE_FLAGS, input);

        output.flags
    }

    /// Returns the mutable feature flags.
    pub fn mutable_flags(&self) -> u64 {
        let input = GetResourceMutableFlagsInput {
            resource_def_ref: *self,
        };
        let output: GetResourceMutableFlagsOutput = call_engine(GET_RESOURCE_MUTABLE_FLAGS, input);

        output.mutable_flags
    }

    /// Returns the current supply of this resource.
    pub fn total_supply(&self) -> Decimal {
        let input = GetResourceTotalSupplyInput {
            resource_def_ref: *self,
        };
        let output: GetResourceTotalSupplyOutput = call_engine(GET_RESOURCE_TOTAL_SUPPLY, input);

        output.total_supply
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, key: &NonFungibleKey) -> T {
        let input = GetNonFungibleDataInput {
            resource_def_ref: *self,
            key: key.clone(),
        };
        let output: GetNonFungibleDataOutput = call_engine(GET_NON_FUNGIBLE_DATA, input);

        T::decode(&output.immutable_data, &output.mutable_data).unwrap()
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &mut self,
        key: &NonFungibleKey,
        new_data: T,
        auth: BucketRef,
    ) {
        let input = UpdateNonFungibleMutableDataInput {
            resource_def_ref: *self,
            key: key.clone(),
            new_mutable_data: new_data.mutable_data(),
            auth: auth.0,
        };
        let _: UpdateNonFungibleMutableDataOutput =
            call_engine(UPDATE_NON_FUNGIBLE_MUTABLE_DATA, input);
    }

    /// Turns on feature flags.
    pub fn enable_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceFlagsInput {
            resource_def_ref: *self,
            new_flags: self.flags() | flags,
            auth: auth.0,
        };
        let _output: UpdateResourceFlagsOutput = call_engine(UPDATE_RESOURCE_FLAGS, input);
    }

    /// Turns off feature flags.
    pub fn disable_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceFlagsInput {
            resource_def_ref: *self,
            new_flags: self.flags() & !flags,
            auth: auth.0,
        };
        let _output: UpdateResourceFlagsOutput = call_engine(UPDATE_RESOURCE_FLAGS, input);
    }

    /// Locks feature flag settings.
    pub fn lock_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceMutableFlagsInput {
            resource_def_ref: *self,
            new_mutable_flags: self.flags() & !flags,
            auth: auth.0,
        };
        let _output: UpdateResourceMutableFlagsOutput =
            call_engine(UPDATE_RESOURCE_MUTABLE_FLAGS, input);
    }

    pub fn update_metadata(&mut self, new_metadata: HashMap<String, String>, auth: BucketRef) {
        let input = UpdateResourceMetadataInput {
            resource_def_ref: *self,
            new_metadata,
            auth: auth.0,
        };
        let _output: UpdateResourceMetadataOutput = call_engine(UPDATE_RESOURCE_METADATA, input);
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParseResourceDefRefError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceDefRefError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseResourceDefRefError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceDefRef {
    type Error = ParseResourceDefRefError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseResourceDefRefError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceDefRef {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(ResourceDefRef, CustomType::ResourceDefRef, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for ResourceDefRef {
    type Err = ParseResourceDefRefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseResourceDefRefError::InvalidHex)?;
        if bytes.get(0) != Some(&3u8) {
            return Err(ParseResourceDefRefError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for ResourceDefRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(3, &self.0)))
    }
}

impl fmt::Debug for ResourceDefRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
