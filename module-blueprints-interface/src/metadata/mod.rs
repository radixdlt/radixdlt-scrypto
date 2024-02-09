mod invocations;
mod roles;

pub use invocations::*;
pub use roles::*;

pub const EMPTY: Option<radix_engine_common::prelude::MetadataValue> = None;
