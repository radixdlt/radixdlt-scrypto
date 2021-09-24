mod bucket;
mod component;
mod package;
mod resource;
mod storage;

pub use bucket::{Bucket, BucketError, BucketRef, LockedBucket, Vault};
pub use component::Component;
pub use package::Package;
pub use resource::ResourceDef;
pub use storage::Storage;

/// Represents a log severity.
#[derive(Debug, Clone)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
