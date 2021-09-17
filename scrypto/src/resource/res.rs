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

/// Utility for creating new resources
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

    pub fn new_fixed<T: From<BID>>(metadata: HashMap<String, String>, supply: U256) -> T {
        let input = CreateResourceFixedInput { metadata, supply };
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

    pub fn mint<T: From<BID>>(&self, amount: U256) -> T {
        assert!(amount >= U256::one());

        let input = MintResourceInput {
            resource: self.address,
            amount,
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

    /// Create tokens with mutable supply; the resource can be minted using `Resource::mint()` afterwards.
    pub fn create_tokens_mutable(&self, minter: Address) -> Resource {
        Resource::new_mutable(self.metadata.clone(), minter)
    }

    /// Create tokens with fixed supply.
    pub fn create_tokens_fixed<T: Into<U256>>(&self, supply: T) -> Tokens {
        Resource::new_fixed(self.metadata.clone(), supply.into())
    }

    /// Create badges with mutable supply; the resource can be minted using `Resource::mint()` afterwards.
    pub fn create_badges_mutable(&self, minter: Address) -> Resource {
        Resource::new_mutable(self.metadata.clone(), minter)
    }

    /// Create badges with fixed supply.
    pub fn create_badges_fixed(&self, supply: U256) -> Badges {
        Resource::new_fixed(self.metadata.clone(), supply)
    }

    /// Add metadata attribute.
    pub fn metadata(&mut self, name: &str, value: &str) -> &mut Self {
        self.metadata.insert(name.to_owned(), value.to_owned());
        self
    }

    pub fn symbol(&mut self, symbol: &str) -> &mut Self {
        self.metadata("symbol", symbol)
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.metadata("name", name)
    }

    pub fn description(&mut self, description: &str) -> &mut Self {
        self.metadata("description", description)
    }

    pub fn url(&mut self, url: &str) -> &mut Self {
        self.metadata("url", url)
    }

    pub fn icon_url(&mut self, icon_url: &str) -> &mut Self {
        self.metadata("icon_url", icon_url)
    }
}
