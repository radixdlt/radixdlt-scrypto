use sbor::model::*;
use sbor::{Decode, Describe, Encode};

use crate::kernel::*;
use crate::resource::*;
use crate::types::rust::borrow::ToOwned;
use crate::types::rust::string::ToString;
use crate::types::*;

/// A primitive piece of state which has a single owner, and behaves like a physical object.
#[derive(Debug, Encode, Decode)]
pub struct Resource {
    address: Address,
}

impl Describe for Resource {
    fn describe() -> Type {
        Type::SystemType {
            name: "::scrypto::constructs::Resource".to_owned(),
        }
    }
}

impl From<Address> for Resource {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Into<Address> for Resource {
    fn into(self) -> Address {
        self.address
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
            info: ResourceInfo {
                symbol: symbol.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                url: url.to_string(),
                icon_url: icon_url.to_string(),
                minter: Some(minter),
                supply: None,
            },
        };
        let output: CreateResourceMutableOutput = call_kernel(CREATE_RESOURCE_MUTABLE, input);

        output.resource.into()
    }

    pub fn new_fixed_tokens(
        symbol: &str,
        name: &str,
        description: &str,
        url: &str,
        icon_url: &str,
        supply: U256,
    ) -> Tokens {
        let input = CreateResourceFixedInput {
            info: ResourceInfo {
                symbol: symbol.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                url: url.to_string(),
                icon_url: icon_url.to_string(),
                minter: None,
                supply: Some(supply),
            },
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        output.bucket.into()
    }

    pub fn new_fixed_badges(
        symbol: &str,
        name: &str,
        description: &str,
        url: &str,
        icon_url: &str,
        supply: U256,
    ) -> Badges {
        let input = CreateResourceFixedInput {
            info: ResourceInfo {
                symbol: symbol.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                url: url.to_string(),
                icon_url: icon_url.to_string(),
                minter: None,
                supply: Some(supply),
            },
        };
        let output: CreateResourceFixedOutput = call_kernel(CREATE_RESOURCE_FIXED, input);

        output.bucket.into()
    }

    pub fn get_info(&self) -> ResourceInfo {
        let input = GetResourceInfoInput {
            resource: self.address,
        };
        let output: GetResourceInfoOutput = call_kernel(GET_RESOURCE_INFO, input);

        output.result.unwrap()
    }

    fn mint(&self, amount: U256) -> BID {
        assert!(amount >= U256::one());

        let input = MintResourceInput {
            resource: self.address,
            amount,
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket
    }

    pub fn mint_tokens(&self, amount: U256) -> Tokens {
        self.mint(amount).into()
    }

    pub fn mint_badges(&self, amount: U256) -> Badges {
        self.mint(amount).into()
    }

    pub fn address(&self) -> Address {
        self.address
    }
}
