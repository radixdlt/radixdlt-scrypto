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

pub struct NonFungibleVaultBlueprint;

impl NonFungibleVaultBlueprint {
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
        let taken = NonFungibleVault::take(receiver, *amount, api)?;

        // Create node
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address: info.resource_address,
                    resource_type: info.resource_type,
                })
                    .unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(bucket_id))
    }
}
