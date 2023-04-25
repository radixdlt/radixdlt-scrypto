use crate::blueprints::resource::*;
use crate::errors::{KernelError, RuntimeError};
use crate::kernel::heap::{DroppedFungibleBucket, DroppedNonFungibleBucket};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node_modules::type_info::TypeInfoBlueprint;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
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
    pub resource_type: ResourceType,
}

pub fn drop_fungible_bucket_of_address<Y>(
    expected_address: ResourceAddress,
    bucket_node_id: &NodeId,
    api: &mut Y,
) -> Result<DroppedFungibleBucket, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
{
    // Note that we assume the input is indeed a bucket, checked by schema
    let resource_address = ResourceAddress::new_unchecked(
        TypeInfoBlueprint::get_type(bucket_node_id, api)?
            .parent()
            .expect("Missing parent for fungible bucket")
            .into(),
    );
    let node_substates = api.kernel_drop_node(bucket_node_id)?;

    if resource_address != expected_address {
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
    // Note that we assume the input is indeed a bucket, checked by schema
    let resource_address = ResourceAddress::new_unchecked(
        TypeInfoBlueprint::get_type(bucket_node_id, api)?
            .parent()
            .expect("Missing parent for fungible bucket")
            .into(),
    );
    let node_substates = api.kernel_drop_node(bucket_node_id)?;

    if resource_address != expected_address {
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
