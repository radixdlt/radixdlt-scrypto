use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::types::*;

/// Represents the definition of a resource.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceDef {
    address: Address,
}

impl From<Address> for ResourceDef {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl From<ResourceDef> for Address {
    fn from(a: ResourceDef) -> Address {
        a.address
    }
}

impl ResourceDef {
    pub fn new_mutable(metadata: HashMap<String, String>, minter: Address) -> Self {
        let input = CreateResourceMutableInput { metadata, minter };
        let output: CreateResourceMutableOutput = call_kernel(CREATE_RESOURCE_MUTABLE, input);

        output.resource_address.into()
    }

    pub fn new_fixed<T: Into<Amount>>(
        metadata: HashMap<String, String>,
        supply: T,
    ) -> (Self, Bucket) {
        let input = CreateResourceFixedInput {
            metadata,
            supply: supply.into(),
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        (output.resource_address.into(), output.bucket.into())
    }

    pub fn mint<T: Into<Amount>>(&self, amount: T) -> Bucket {
        let amt = amount.into();
        assert!(amt >= Amount::one());

        let input = MintResourceInput {
            resource_address: self.address,
            amount: amt,
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    pub fn burn(bucket: Bucket) {
        let input = BurnResourceInput {
            bucket: bucket.into(),
        };
        let _output: BurnResourceOutput = call_kernel(BURN_RESOURCE, input);
    }

    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource_address: self.address,
        };
        let output: GetResourceMetadataOutput = call_kernel(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    pub fn minter(&self) -> Option<Address> {
        let input = GetResourceMinterInput {
            resource_address: self.address,
        };
        let output: GetResourceMinterOutput = call_kernel(GET_RESOURCE_MINTER, input);

        output.minter
    }

    pub fn supply(&self) -> Amount {
        let input = GetResourceSupplyInput {
            resource_address: self.address,
        };
        let output: GetResourceSupplyOutput = call_kernel(GET_RESOURCE_SUPPLY, input);

        output.supply
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

impl Describe for ResourceDef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_RESOURCE_DEF.to_owned(),
        }
    }
}
