use scrypto::resource::*;
use scrypto::types::*;

pub fn create_mutable(symbol: &str, minter: Address) -> Address {
    ResourceBuilder::new()
        .metadata("symbol", symbol)
        .create_mutable(minter)
        .into()
}

pub fn create_fixed(symbol: &str, supply: Amount) -> Bucket {
    ResourceBuilder::new()
        .metadata("symbol", symbol)
        .create_fixed(supply)
}

pub fn mint_resource(resource: Address, amount: u32) -> Bucket {
    let resource = Resource::from(resource);
    resource.mint(Amount::from(amount))
}
