use sbor::{describe::Type, *};

use crate::engine::*;
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::types::*;

/// Represents a resource definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceDef([u8; 26]);

impl ResourceDef {
    fn this(&self) -> Self {
        Self(self.0)
    }

    /// Creates a resource with the given parameters.
    ///
    /// A bucket is returned iif an initial supply is provided.
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        authorities: HashMap<ResourceDef, u64>,
        initial_supply: Option<Supply>,
    ) -> (ResourceDef, Option<Bucket>) {
        let input = CreateResourceInput {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorities,
            initial_supply,
        };
        let output: CreateResourceOutput = call_engine(CREATE_RESOURCE, input);

        (output.resource_def, output.bucket.map(Into::into))
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&mut self, amount: T, auth: BucketRef) -> Bucket {
        let input = MintResourceInput {
            resource_def: self.this(),
            new_supply: Supply::Fungible {
                amount: amount.into(),
            },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        output.bucket
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
            resource_def: self.this(),
            new_supply: Supply::NonFungible { entries },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        output.bucket
    }

    /// Burns a bucket of resources.
    pub fn burn(&mut self, bucket: Bucket) {
        let input = BurnResourceInput { bucket, auth: None };
        let _output: BurnResourceOutput = call_engine(BURN_RESOURCE, input);
    }

    /// Burns a bucket of resources.
    pub fn burn_with_auth(&mut self, bucket: Bucket, auth: BucketRef) {
        let input = BurnResourceInput {
            bucket,
            auth: Some(auth.into()),
        };
        let _output: BurnResourceOutput = call_engine(BURN_RESOURCE, input);
    }

    /// Returns the resource type.
    pub fn resource_type(&self) -> ResourceType {
        let input = GetResourceTypeInput {
            resource_def: self.this(),
        };
        let output: GetResourceTypeOutput = call_engine(GET_RESOURCE_TYPE, input);

        output.resource_type
    }

    /// Returns the metadata associated with this resource.
    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource_def: self.this(),
        };
        let output: GetResourceMetadataOutput = call_engine(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    /// Returns the feature flags.
    pub fn flags(&self) -> u64 {
        let input = GetResourceFlagsInput {
            resource_def: self.this(),
        };
        let output: GetResourceFlagsOutput = call_engine(GET_RESOURCE_FLAGS, input);

        output.flags
    }

    /// Returns the mutable feature flags.
    pub fn mutable_flags(&self) -> u64 {
        let input = GetResourceMutableFlagsInput {
            resource_def: self.this(),
        };
        let output: GetResourceMutableFlagsOutput = call_engine(GET_RESOURCE_MUTABLE_FLAGS, input);

        output.mutable_flags
    }

    /// Returns the current supply of this resource.
    pub fn total_supply(&self) -> Decimal {
        let input = GetResourceTotalSupplyInput {
            resource_def: self.this(),
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
            resource_def: self.this(),
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
            resource_def: self.this(),
            key: key.clone(),
            new_mutable_data: new_data.mutable_data(),
            auth: auth.into(),
        };
        let _: UpdateNonFungibleMutableDataOutput =
            call_engine(UPDATE_NON_FUNGIBLE_MUTABLE_DATA, input);
    }

    /// Turns on feature flags.
    pub fn enable_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceFlagsInput {
            resource_def: self.this(),
            new_flags: self.flags() | flags,
            auth: auth.into(),
        };
        let _output: UpdateResourceFlagsOutput = call_engine(UPDATE_RESOURCE_FLAGS, input);
    }

    /// Turns off feature flags.
    pub fn disable_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceFlagsInput {
            resource_def: self.this(),
            new_flags: self.flags() & !flags,
            auth: auth.into(),
        };
        let _output: UpdateResourceFlagsOutput = call_engine(UPDATE_RESOURCE_FLAGS, input);
    }

    /// Locks feature flag settings.
    pub fn lock_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceMutableFlagsInput {
            resource_def: self.this(),
            new_mutable_flags: self.flags() & !flags,
            auth: auth.into(),
        };
        let _output: UpdateResourceMutableFlagsOutput =
            call_engine(UPDATE_RESOURCE_MUTABLE_FLAGS, input);
    }

    pub fn update_metadata(&mut self, new_metadata: HashMap<String, String>, auth: BucketRef) {
        let input = UpdateResourceMetadataInput {
            resource_def: self.this(),
            new_metadata,
            auth: auth.into(),
        };
        let _output: UpdateResourceMetadataOutput = call_engine(UPDATE_RESOURCE_METADATA, input);
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParseResourceDefError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceDefError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseResourceDefError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceDef {
    type Error = ParseResourceDefError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseResourceDefError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceDef {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(ResourceDef, CustomType::ResourceDef, Vec::new());

//======
// text
//======

impl FromStr for ResourceDef {
    type Err = ParseResourceDefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseResourceDefError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl ToString for ResourceDef {
    fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}
