use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BucketError {
    ResourceError(ResourceError),
    ProofError(ProofError),
    MismatchingResource,
    InvalidAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct BucketInfoSubstate {
    pub resource_address: ResourceAddress, // TODO: remove
    pub resource_type: ResourceType,
}

impl BucketInfoSubstate {
    pub fn of<Y>(receiver: &NodeId, api: &mut Y) -> Result<Self, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle =
            api.sys_lock_substate(receiver, &BucketOffset::Info.into(), LockFlags::read_only())?;
        let substate_ref: BucketInfoSubstate = api.sys_read_substate_typed(handle)?;
        let info = substate_ref.clone();
        api.sys_drop_lock(handle)?;
        Ok(info)
    }
}
