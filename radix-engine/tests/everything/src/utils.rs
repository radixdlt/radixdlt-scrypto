use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;

pub fn create_tokens(symbol: &str, supply: u32) -> Address {
    let resource = Resource::new(
        symbol,
        "name",
        "description",
        "url",
        "icon_url",
        Some(Context::address()),
        Some(U256::from(supply)),
    );
    resource.into()
}

pub fn mint_tokens(address: Address, amount: u32) -> Tokens {
    let resource: Resource = address.into();
    resource.mint_tokens(U256::from(amount))
}
