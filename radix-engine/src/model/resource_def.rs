use sbor::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::types::*;

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    pub metadata: HashMap<String, String>,
    pub minter: Option<Address>,
    pub supply: Amount,
    pub auth: Option<Address>,
}
