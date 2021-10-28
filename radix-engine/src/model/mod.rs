mod auth;
mod bucket;
mod component;
mod lazy_map;
mod package;
mod resource_def;
mod vault;

pub use auth::Auth;
pub use bucket::{Bucket, BucketError, BucketRef, LockedBucket};
pub use component::{Component, ComponentError};
pub use lazy_map::{LazyMap, LazyMapError};
pub use package::Package;
pub use resource_def::{ResourceDef, ResourceDefError};
pub use vault::{Vault, VaultError};

/// Represents a log severity.
#[derive(Debug, Clone)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
