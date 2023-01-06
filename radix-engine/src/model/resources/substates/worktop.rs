use native_sdk::resource::{ResourceManager, SysBucket};
use radix_engine_interface::api::api::{EngineApi, Invokable, InvokableModel};
use crate::engine::{ApplicationError, RuntimeError, SystemApi};
use crate::model::{
    BucketSubstate, LockableResource, Resource, ResourceOperationError, WorktopError,
};
use crate::types::*;

/// Worktop collects resources from function or method returns.
#[derive(Debug)]
pub struct WorktopSubstate {
    pub resources: BTreeMap<ResourceAddress, Own>,
}

impl WorktopSubstate {
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }
}
