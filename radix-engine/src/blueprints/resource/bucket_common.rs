use crate::blueprints::resource::*;
use crate::errors::{KernelError, RuntimeError};
use crate::kernel::heap::{DroppedFungibleBucket, DroppedNonFungibleBucket};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
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
    pub resource_address: ResourceAddress, // TODO: remove address in favour of parent
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

pub fn drop_fungible_bucket_of_address<Y>(
    expected_address: ResourceAddress,
    bucket_node_id: &NodeId,
    api: &mut Y,
) -> Result<DroppedFungibleBucket, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
{
    let node_substates = api.kernel_drop_node(bucket_node_id)?;

    // Note that we assume the input is indeed a bucket; we're just not sure if it's
    // fungible or non-fungible, because schema type allows either.
    let info: BucketInfoSubstate = node_substates
        .get(&SysModuleId::Object.into())
        .unwrap()
        .get(&BucketOffset::Info.into())
        .map(|x| x.as_typed().unwrap())
        .unwrap();

    if info.resource_address != expected_address {
        return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
            bucket_node_id.clone(),
        )));
    }

    let bucket: DroppedFungibleBucket = node_substates.into();
    if bucket.locked.is_locked() {
        return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
            bucket_node_id.clone(),
        )));
    }

    Ok(bucket)
}

pub fn drop_non_fungible_bucket_of_address<Y>(
    expected_address: ResourceAddress,
    bucket_node_id: &NodeId,
    api: &mut Y,
) -> Result<DroppedNonFungibleBucket, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
{
    let node_substates = api.kernel_drop_node(bucket_node_id)?;

    // Note that we assume the input is indeed a bucket; we're just not sure if it's
    // fungible or non-fungible, because schema type allows either.
    let info: BucketInfoSubstate = node_substates
        .get(&SysModuleId::Object.into())
        .unwrap()
        .get(&BucketOffset::Info.into())
        .map(|x| x.as_typed().unwrap())
        .unwrap();

    if info.resource_address != expected_address {
        return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
            bucket_node_id.clone(),
        )));
    }

    let bucket: DroppedNonFungibleBucket = node_substates.into();
    if bucket.locked.is_locked() {
        return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
            bucket_node_id.clone(),
        )));
    }

    Ok(bucket)
}
