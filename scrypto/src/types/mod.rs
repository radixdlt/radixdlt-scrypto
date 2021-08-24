use sbor::{Decode, Describe, Encode};
use scrypto_types::rust::string::String;

// Re-export primitive types
pub use scrypto_types::*;

/// Information about a component.
#[derive(Debug, Clone, Describe, Encode, Decode)]
pub struct ComponentInfo {
    pub package: Address,
    pub blueprint: String,
}

/// Information about a resource.
#[derive(Debug, Clone, Describe, Encode, Decode)]
pub struct ResourceInfo {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
}

/// Represents a logging level.
#[derive(Debug, Clone, Describe, Encode, Decode)]
pub enum Level {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}
