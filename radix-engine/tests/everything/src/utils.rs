use scrypto::resource::*;
use scrypto::types::*;

pub fn create_mutable(symbol: &str) -> (ResourceDef, Bucket) {
    let auth = ResourceBuilder::new()
        .metadata("name", "Mint Auth")
        .create_fixed(1);

    let resource_def = ResourceBuilder::new()
        .metadata("symbol", symbol)
        .create_mutable(auth.resource_def());

    (resource_def, auth)
}

pub fn create_fixed(symbol: &str, supply: Amount) -> Bucket {
    ResourceBuilder::new()
        .metadata("symbol", symbol)
        .create_fixed(supply)
}
