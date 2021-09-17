use sbor::{describe::Type, *};

use crate::constants::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::types::*;

/// An abstraction of digital assets, e.g. token, badge and NFT.
#[derive(Debug, Encode, Decode)]
pub struct Resource {
    address: Address,
}

/// Information about a resource.
#[derive(Debug, Clone, Describe, Encode, Decode)]
pub struct ResourceInfo {
    pub metadata: HashMap<String, String>,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
}

/// Utility for creating new resource
pub struct ResourceBuilder {
    metadata: HashMap<String, String>,
}

impl From<Address> for Resource {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl From<Resource> for Address {
    fn from(a: Resource) -> Address {
        a.address
    }
}

impl Resource {
    pub fn new_mutable(metadata: HashMap<String, String>, minter: Address) -> Self {
        let input = CreateResourceMutableInput { metadata, minter };
        let output: CreateResourceMutableOutput = call_kernel(CREATE_RESOURCE_MUTABLE, input);

        output.resource.into()
    }

    pub fn new_fixed<T: Into<U256>>(metadata: HashMap<String, String>, supply: T) -> Bucket {
        let input = CreateResourceFixedInput {
            metadata,
            supply: supply.into(),
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        output.bucket.into()
    }

    pub fn info(&self) -> ResourceInfo {
        let input = GetResourceInfoInput {
            resource: self.address,
        };
        let output: GetResourceInfoOutput = call_kernel(GET_RESOURCE_INFO, input);

        ResourceInfo {
            metadata: output.metadata,
            minter: output.minter,
            supply: output.supply,
        }
    }

    pub fn mint<T: Into<U256>>(&self, amount: T) -> Bucket {
        let amt = amount.into();
        assert!(amt >= U256::one());

        let input = MintResourceInput {
            resource: self.address,
            amount: amt,
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

impl Describe for Resource {
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
    pub fn metadata(&mut self, name: &str, value: &str) -> &mut Self {
        self.metadata.insert(name.to_owned(), value.to_owned());
        self
    }

    /// Create resource with mutable supply; the resource can be minted using `Resource::mint()` afterwards.
    pub fn create_mutable(&self, minter: Address) -> Resource {
        Resource::new_mutable(self.metadata.clone(), minter)
    }

    /// Create resource with fixed supply.
    pub fn create_fixed<T: Into<U256>>(&self, supply: T) -> Bucket {
        Resource::new_fixed(self.metadata.clone(), supply.into())
    }
}
