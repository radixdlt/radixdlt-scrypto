use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
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
        if !address.is_package() {
            scrypto_abort("Unable to downcast Address to ResourceDef");
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
    pub fn new_mutable<A: Into<ResourceDef>>(metadata: HashMap<String, String>, minter: A) -> Self {
        let input = CreateResourceMutableInput {
            metadata,
            minter: minter.into().address(),
        };
        let output: CreateResourceMutableOutput = call_kernel(CREATE_RESOURCE_MUTABLE, input);

        output.resource_def.into()
    }

    /// Creates a resource with fixed supply. The created resource is immediately returned.
    pub fn new_fixed<T: Into<Amount>>(
        metadata: HashMap<String, String>,
        supply: T,
    ) -> (Self, Bucket) {
        let input = CreateResourceFixedInput {
            metadata,
            supply: supply.into(),
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        (output.resource_def.into(), output.bucket.into())
    }

    /// Mints resources
    pub fn mint<T: Into<Amount>>(&self, amount: T) -> Bucket {
        let input = MintResourceInput {
            resource_def: self.address,
            amount: amount.into(),
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    /// Burns a bucket of resources.
    pub fn burn(bucket: Bucket) {
        let input = BurnResourceInput {
            bucket: bucket.into(),
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

    /// Returns the current supply of this resource.
    pub fn supply(&self) -> Amount {
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
