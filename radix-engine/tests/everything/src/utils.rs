use scrypto::resource::*;
use scrypto::types::*;

pub fn create_mutable(symbol: &str, minter: Address) -> ResourceDef {
    ResourceBuilder::new()
        .metadata("symbol", symbol)
        .create_mutable(minter)
}

pub fn create_fixed(symbol: &str, supply: Amount) -> Bucket {
    ResourceBuilder::new()
        .metadata("symbol", symbol)
        .create_fixed(supply)
}
