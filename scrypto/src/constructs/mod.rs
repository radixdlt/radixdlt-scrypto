mod blueprint;
mod component;
mod context;
mod logger;
mod package;
mod resource;

pub use blueprint::Blueprint;
pub use component::Component;
pub use context::Context;
pub use logger::Logger;
pub use package::Package;
pub use resource::Resource;

use crate::rust::string::String;
use crate::types::*;
use sbor::*;

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
