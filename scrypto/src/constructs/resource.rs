use sbor::*;

use crate::constructs::*;
use crate::kernel::*;
use crate::rust::string::ToString;
use crate::types::*;

/// A primitive piece of state which has a single owner, and behaves like a physical object.
#[derive(Debug, Encode, Decode, Describe)]
pub struct Resource {
    address: Address,
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
