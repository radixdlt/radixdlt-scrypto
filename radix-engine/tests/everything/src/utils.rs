use scrypto::resource::*;
use scrypto::types::*;

pub fn create_mutable_tokens(symbol: &str, minter: Address) -> Address {
    ResourceBuilder::new()
        .symbol(symbol)
        .create_tokens_mutable(minter)
        .into()
}

pub fn create_fixed_tokens(symbol: &str, supply: U256) -> Tokens {
    ResourceBuilder::new()
        .symbol(symbol)
        .create_tokens_fixed(supply)
}

pub fn mint_tokens(resource: Address, amount: u32) -> Tokens {
    let resource = Resource::from(resource);
    resource.mint(U256::from(amount))
}
