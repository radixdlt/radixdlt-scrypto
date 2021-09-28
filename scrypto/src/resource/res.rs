use sbor::{describe::Type, *};

use crate::constants::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::types::*;

/// The definition of a particular class of resources.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceDef {
    address: Address,
}

/// Utility for creating resources
pub struct ResourceBuilder {
    metadata: HashMap<String, String>,
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

        output.resource.into()
    }

    pub fn new_fixed<T: Into<Amount>>(metadata: HashMap<String, String>, supply: T) -> Bucket {
        let input = CreateResourceFixedInput {
            metadata,
            supply: supply.into(),
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        output.bucket.into()
    }

    pub fn mint<T: Into<Amount>>(&self, amount: T) -> Bucket {
        let amt = amount.into();
        assert!(amt >= Amount::one());

        let input = MintResourceInput {
            resource: self.address,
            amount: amt,
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    pub fn burn(bucket: Bucket) {
        let input = BurnResourceInput { bucket: bucket.into() };
        let _output: BurnResourceOutput = call_kernel(BURN_RESOURCE, input);
    }

    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource: self.address,
        };
        let output: GetResourceMetadataOutput = call_kernel(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    pub fn minter(&self) -> Option<Address> {
        let input = GetResourceMinterInput {
            resource: self.address,
        };
        let output: GetResourceMinterOutput = call_kernel(GET_RESOURCE_MINTER, input);

        output.minter
    }

    pub fn supply(&self) -> Amount {
        let input = GetResourceSupplyInput {
            resource: self.address,
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
            name: SCRYPTO_NAME_RESOURCE.to_owned(),
        }
    }
}

impl ResourceBuilder {
    /// New resource builder.
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    /// Add metadata attribute.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        self.metadata.insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    /// Create resource with mutable supply; the resource can be minted using `Resource::mint()` afterwards.
    pub fn create_mutable(&self, minter: Address) -> ResourceDef {
        ResourceDef::new_mutable(self.metadata.clone(), minter)
    }

    /// Create resource with fixed supply.
    pub fn create_fixed<T: Into<Amount>>(&self, supply: T) -> Bucket {
        ResourceDef::new_fixed(self.metadata.clone(), supply.into())
    }
}

impl Default for ResourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}