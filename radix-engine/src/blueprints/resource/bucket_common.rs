use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug)]
pub struct DroppedFungibleBucket {
    pub liquid: LiquidFungibleResource,
    pub locked: LockedFungibleResource,
}

#[derive(Debug)]
pub struct DroppedNonFungibleBucket {
    pub liquid: LiquidNonFungibleResource,
    pub locked: LockedNonFungibleResource,
}

impl From<Vec<Vec<u8>>> for DroppedFungibleBucket {
    fn from(val: Vec<Vec<u8>>) -> Self {
        let liquid: LiquidFungibleResource =
            scrypto_decode(&val[FungibleBucketField::Liquid as usize]).unwrap();
        let locked: LockedFungibleResource =
            scrypto_decode(&val[FungibleBucketField::Locked as usize]).unwrap();

        DroppedFungibleBucket { liquid, locked }
    }
}

impl From<Vec<Vec<u8>>> for DroppedNonFungibleBucket {
    fn from(val: Vec<Vec<u8>>) -> Self {
        let liquid: LiquidNonFungibleResource =
            scrypto_decode(&val[NonFungibleBucketField::Liquid as usize]).unwrap();
        let locked: LockedNonFungibleResource =
            scrypto_decode(&val[NonFungibleBucketField::Locked as usize]).unwrap();

        DroppedNonFungibleBucket { liquid, locked }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BucketError {
    ResourceError(ResourceError),
    ProofError(ProofError),
    Locked(error_models::OwnedNodeId),
    InvalidAmount(Decimal),
    DecimalOverflow,
}

impl From<BucketError> for RuntimeError {
    fn from(bucket_error: BucketError) -> Self {
        RuntimeError::ApplicationError(ApplicationError::BucketError(bucket_error))
    }
}

pub fn drop_fungible_bucket<Y: SystemApi<RuntimeError>>(
    bucket_node_id: &NodeId,
    api: &mut Y,
) -> Result<DroppedFungibleBucket, RuntimeError> {
    let fields = api.drop_object(bucket_node_id)?;
    let bucket: DroppedFungibleBucket = fields.into();
    if bucket.locked.is_locked() {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::BucketError(BucketError::Locked((*bucket_node_id).into())),
        ));
    }

    Ok(bucket)
}

pub fn drop_non_fungible_bucket<Y: SystemApi<RuntimeError>>(
    bucket_node_id: &NodeId,
    api: &mut Y,
) -> Result<DroppedNonFungibleBucket, RuntimeError> {
    let fields = api.drop_object(bucket_node_id)?;
    let bucket: DroppedNonFungibleBucket = fields.into();
    if bucket.locked.is_locked() {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::BucketError(BucketError::Locked((*bucket_node_id).into())),
        ));
    }

    Ok(bucket)
}
