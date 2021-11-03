use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::format;
use crate::rust::string::String;
use crate::rust::vec;
use crate::types::*;
use crate::utils::*;

/// Represents the definition of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDef {
    address: Address,
}

impl From<Address> for ResourceDef {
    fn from(address: Address) -> Self {
        if !address.is_resource_def() {
            scrypto_abort(format!(
                "Unable to downcast Address to ResourceDef: {}",
                address
            ));
        }

        Self { address }
    }
}

impl From<ResourceDef> for Address {
    fn from(a: ResourceDef) -> Address {
        a.address
    }
}

impl ResourceDef {
    /// Creates a resource with mutable supply. The resource definition is returned.
    pub fn new_mutable<A: Into<ResourceDef>>(
        granularity: u8,
        metadata: HashMap<String, String>,
        minter: A,
    ) -> Self {
        let input = CreateResourceMutableInput {
            granularity,
            metadata,
            minter: minter.into().address(),
        };
        let output: CreateResourceMutableOutput = call_kernel(CREATE_RESOURCE_MUTABLE, input);

        output.resource_def.into()
    }

    /// Creates a resource with fixed supply. The created resource is immediately returned.
    pub fn new_fixed<T: Into<Decimal>>(
        granularity: u8,
        metadata: HashMap<String, String>,
        supply: T,
    ) -> (Self, Bucket) {
        let input = CreateResourceFixedInput {
            granularity,
            metadata,
            supply: supply.into(),
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        (output.resource_def.into(), output.bucket.into())
    }

    /// Mints resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T, minter: BucketRef) -> Bucket {
        let input = MintResourceInput {
            resource_def: self.address,
            amount: amount.into(),
            minter: minter.into(),
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    /// Burns a bucket of resources.
    pub fn burn(&self, bucket: Bucket, minter: BucketRef) {
        let input = BurnResourceInput {
            bucket: bucket.into(),
            minter: minter.into(),
        };
        let _output: BurnResourceOutput = call_kernel(BURN_RESOURCE, input);
    }

    /// Returns the metadata associated with this resource.
    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource_def: self.address,
        };
        let output: GetResourceMetadataOutput = call_kernel(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    /// Returns the minter address.
    pub fn minter(&self) -> Option<Address> {
        let input = GetResourceMinterInput {
            resource_def: self.address,
        };
        let output: GetResourceMinterOutput = call_kernel(GET_RESOURCE_MINTER, input);

        output.minter
    }

    /// Returns the granularity.
    pub fn granularity(&self) -> u8 {
        let input = GetResourceGranularityInput {
            resource_def: self.address,
        };
        let output: GetResourceGranularityOutput = call_kernel(GET_RESOURCE_MINTER, input);

        output.granularity
    }

    /// Returns the current supply of this resource.
    pub fn supply(&self) -> Decimal {
        let input = GetResourceSupplyInput {
            resource_def: self.address,
        };
        let output: GetResourceSupplyOutput = call_kernel(GET_RESOURCE_SUPPLY, input);

        output.supply
    }

    /// Returns the address of this resource.
    pub fn address(&self) -> Address {
        self.address
    }
}

//========
// SBOR
//========

impl TypeId for ResourceDef {
    fn type_id() -> u8 {
        Address::type_id()
    }
}

impl Encode for ResourceDef {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.address.encode_value(encoder);
    }
}

impl Decode for ResourceDef {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Address::decode_value(decoder).map(Into::into)
    }
}

impl Describe for ResourceDef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_RESOURCE_DEF.to_owned(),
            generics: vec![],
        }
    }
}
