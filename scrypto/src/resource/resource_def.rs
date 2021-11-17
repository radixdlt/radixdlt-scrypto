use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::BTreeMap;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::rust::vec;
use crate::types::*;

/// Represents the definition of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDef {
    address: Address,
}

impl From<Address> for ResourceDef {
    fn from(address: Address) -> Self {
        if !address.is_resource_def() {
            panic!("Unable to downcast Address to ResourceDef: {}", address);
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
    pub fn new_mutable(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        minter: Address,
    ) -> Self {
        let input = CreateResourceMutableInput {
            resource_type,
            metadata,
            minter,
        };
        let output: CreateResourceMutableOutput = call_kernel(CREATE_RESOURCE_MUTABLE, input);

        output.resource_def.into()
    }

    /// Creates a resource with fixed supply. The created resource is immediately returned.
    pub fn new_fixed(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        supply: ResourceSupply,
    ) -> (Self, Bucket) {
        let input = CreateResourceFixedInput {
            resource_type,
            metadata,
            supply,
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        (output.resource_def.into(), output.bucket.into())
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T, auth: BucketRef) -> Bucket {
        let input = MintResourceInput {
            resource_def: self.address,
            supply: ResourceSupply::Fungible {
                amount: amount.into(),
            },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    /// Mints non-fungible resources
    pub fn mint_nft<T: Encode>(&self, id: u64, value: T, auth: BucketRef) -> Bucket {
        let mut entries = BTreeMap::new();
        entries.insert(id, scrypto_encode(&value));

        let input = MintResourceInput {
            resource_def: self.address,
            supply: ResourceSupply::NonFungible { entries },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    /// Burns a bucket of resources.
    pub fn burn(&self, bucket: Bucket, auth: BucketRef) {
        let input = BurnResourceInput {
            bucket: bucket.into(),
            auth: auth.into(),
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

    /// Returns the resource type.
    pub fn resource_type(&self) -> ResourceType {
        let input = GetResourceTypeInput {
            resource_def: self.address,
        };
        let output: GetResourceTypeOutput = call_kernel(GET_RESOURCE_MINTER, input);

        output.resource_type
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
