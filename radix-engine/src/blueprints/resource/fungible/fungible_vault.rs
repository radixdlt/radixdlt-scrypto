use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::heap::{DroppedBucket, DroppedBucketResource};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::CostingError;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::{types::*, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use crate::blueprints::resource;

pub struct FungibleVaultBlueprint;

impl FungibleVaultBlueprint {
    pub fn take<Y>(
        receiver: &RENodeId,
        amount: &Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Check amount
        let info = VaultInfoSubstate::of(receiver, api)?;
        if !info.resource_type.check_amount(*amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Take
        let taken = FungibleVault::take(receiver, *amount, api)?;

        // Create node
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address: info.resource_address,
                    resource_type: info.resource_type,
                })
                    .unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(bucket_id))
    }

    pub fn put<Y>(
        receiver: &RENodeId,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Drop other bucket
        let other_bucket: DroppedBucket = api
            .kernel_drop_node(&RENodeId::Object(bucket.0))?
            .into();

        // Check resource address
        let info = VaultInfoSubstate::of(receiver, api)?;
        if info.resource_address != other_bucket.info.resource_address {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::MismatchingResource),
            ));
        }

        // Put
        if let DroppedBucketResource::Fungible(r) = other_bucket.resource {
            FungibleVault::put(receiver, r, api)?;
        } else {
            panic!("expecting fungible bucket")
        }

        Ok(())
    }
}
