use sbor::*;

use crate::engine::{api::*, call_engine};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceAddress(pub [u8; 26]);

impl ResourceAddress {}

#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        let input = MintResourceInput {
            resource_address: self.0,
            mint_params: MintParams::Fungible {
                amount: amount.into(),
            },
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        Bucket(output.bucket_id)
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(&self, id: &NonFungibleId, data: T) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(id.clone(), (data.immutable_data(), data.mutable_data()));

        let input = MintResourceInput {
            resource_address: self.0,
            mint_params: MintParams::NonFungible { entries },
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        Bucket(output.bucket_id)
    }

    /// Burns a bucket of resources.
    pub fn burn(&self, bucket: Bucket) {
        let input = BurnResourceInput {
            bucket_id: bucket.0,
        };
        let _output: BurnResourceOutput = call_engine(BURN_RESOURCE, input);
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

    /// Returns the feature flags.
    pub fn flags(&self) -> u64 {
        let input = GetResourceFlagsInput {
            resource_address: self.0,
        };
        let output: GetResourceFlagsOutput = call_engine(GET_RESOURCE_FLAGS, input);

        output.flags
    }

    /// Returns the mutable feature flags.
    pub fn mutable_flags(&self) -> u64 {
        let input = GetResourceMutableFlagsInput {
            resource_address: self.0,
        };
        let output: GetResourceMutableFlagsOutput = call_engine(GET_RESOURCE_MUTABLE_FLAGS, input);

        output.mutable_flags
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
        let input = UpdateNonFungibleMutableDataInput {
            non_fungible_address: NonFungibleAddress::new(self.0, id.clone()),
            new_mutable_data: new_data.mutable_data(),
        };
        let _: UpdateNonFungibleMutableDataOutput =
            call_engine(UPDATE_NON_FUNGIBLE_MUTABLE_DATA, input);
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

    /// Turns on feature flags.
    pub fn enable_flags(&self, flags: u64) {
        let input = EnableFlagsInput {
            resource_address: self.0,
            flags,
        };
        let _output: EnableFlagsOutput = call_engine(ENABLE_FLAGS, input);
    }

    /// Turns off feature flags.
    pub fn disable_flags(&self, flags: u64) {
        let input = DisableFlagsInput {
            resource_address: self.0,
            flags,
        };
        let _output: DisableFlagsOutput = call_engine(DISABLE_FLAGS, input);
    }

    /// Locks feature flag settings.
    pub fn lock_flags(&self, flags: u64) {
        let input = LockFlagsInput {
            resource_address: self.0,
            flags,
        };
        let _output: LockFlagsOutput = call_engine(LOCK_FLAGS, input);
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

custom_type!(ResourceAddress, CustomType::ResourceAddress, Vec::new());

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
