use sbor::*;
use scrypto::rust::string::String;
use scrypto::types::*;

/// Represents a resource created.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Resource {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
}
