use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;

pub fn create_mutable_tokens(symbol: &str, minter: Address) -> Address {
    let resource = Resource::new_mutable(symbol, "name", "description", "url", "icon_url", minter);
    resource.into()
}

pub fn create_fixed_tokens(symbol: &str, supply: U256) -> Tokens {
    Resource::new_fixed(symbol, "name", "description", "url", "icon_url", supply).1
}

pub fn mint_tokens(resource: Address, amount: u32) -> Tokens {
    let resource = Resource::from(resource);
    resource.mint_tokens(U256::from(amount))
}
