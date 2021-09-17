use sbor::{describe::Type, *};

use crate::constants::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::types::*;

/// A primitive piece of state which has a single owner, and behaves like a physical object.
#[derive(Debug, Encode, Decode)]
pub struct Resource {
    address: Address,
}

/// Information about a resource.
#[derive(Debug, Clone, Describe, Encode, Decode)]
pub struct ResourceInfo {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
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
    pub fn new_mutable(
        symbol: &str,
        name: &str,
        description: &str,
        url: &str,
        icon_url: &str,
        minter: Address,
    ) -> Self {
        let input = CreateResourceMutableInput {
            symbol: symbol.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            url: url.to_string(),
            icon_url: icon_url.to_string(),
            minter,
        };
        let output: CreateResourceMutableOutput = call_kernel(CREATE_RESOURCE_MUTABLE, input);

        output.resource.into()
    }

    pub fn new_fixed<T: From<BID>>(
        symbol: &str,
        name: &str,
        description: &str,
        url: &str,
        icon_url: &str,
        supply: U256,
    ) -> T {
        let input = CreateResourceFixedInput {
            symbol: symbol.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            url: url.to_string(),
            icon_url: icon_url.to_string(),
            supply,
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        output.bucket.into()
    }

    pub fn get_info(&self) -> ResourceInfo {
        let input = GetResourceInfoInput {
            resource: self.address,
        };
        let output: GetResourceInfoOutput = call_kernel(GET_RESOURCE_INFO, input);

        ResourceInfo {
            symbol: output.symbol,
            name: output.name,
            description: output.description,
            url: output.url,
            icon_url: output.icon_url,
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
