use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::heap::{DroppedBucket, DroppedBucketResource};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct NonFungibleVaultIdTypeSubstate {
    pub id_type: NonFungibleIdType,
}

pub struct NonFungibleVaultBlueprint;

impl NonFungibleVaultBlueprint {
    fn check_amount(amount: &Decimal) -> bool {
        !amount.is_negative() && amount.0 % BnumI256::from(10i128.pow(18)) == BnumI256::from(0)
    }

    fn get_id_type<Y>(receiver: &NodeId, api: &mut Y) -> Result<NonFungibleIdType, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle =
            api.sys_lock_substate(receiver, &VaultOffset::Info.into(), LockFlags::read_only())?;
        let info: NonFungibleVaultIdTypeSubstate = api.sys_read_substate_typed(handle)?;
        let id_type = info.id_type;
        api.sys_drop_lock(handle)?;
        Ok(id_type)
    }

    pub fn take<Y>(receiver: &NodeId, amount: &Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Check amount
        if !Self::check_amount(amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Take
        let taken = NonFungibleVault::take(receiver, *amount, api)?;
        let id_type = Self::get_id_type(receiver, api)?;
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );

        // Create node
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address,
                    resource_type: ResourceType::NonFungible { id_type },
                })
                .unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub fn take_non_fungibles<Y>(
        receiver: &NodeId,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Take
        let taken = NonFungibleVault::take_non_fungibles(receiver, &non_fungible_local_ids, api)?;

        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let id_type = Self::get_id_type(receiver, api)?;

        // Create node
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address,
                    resource_type: ResourceType::NonFungible { id_type },
                })
                .unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub fn put<Y>(receiver: &NodeId, bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Drop other bucket
        let other_bucket: DroppedBucket = api.kernel_drop_node(bucket.0.as_node_id())?.into();

        // Check resource address
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        if resource_address != other_bucket.info.resource_address {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::MismatchingResource),
            ));
        }

        // Put
        if let DroppedBucketResource::NonFungible(r) = other_bucket.resource {
            NonFungibleVault::put(receiver, r, api)?;
        } else {
            panic!("Expected non fungible bucket");
        }

        Ok(())
    }

    pub fn get_amount<Y>(receiver: &NodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let amount = NonFungibleVault::liquid_amount(receiver, api)?
            + NonFungibleVault::locked_amount(receiver, api)?;

        Ok(amount)
    }

    pub fn get_non_fungible_local_ids<Y>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let mut ids = NonFungibleVault::liquid_non_fungible_local_ids(receiver, api)?;
        ids.extend(NonFungibleVault::locked_non_fungible_local_ids(
            receiver, api,
        )?);
        Ok(ids)
    }

    pub fn recall<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        if !Self::check_amount(&amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let id_type = Self::get_id_type(receiver, api)?;
        let taken = NonFungibleVault::take(receiver, amount, api)?;
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address,
                    resource_type: ResourceType::NonFungible { id_type },
                })
                .unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, RecallResourceEvent::Amount(amount))?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub fn recall_non_fungibles<Y>(
        receiver: &NodeId,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let taken = NonFungibleVault::take_non_fungibles(receiver, &non_fungible_local_ids, api)?;

        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let id_type = Self::get_id_type(receiver, api)?;

        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address,
                    resource_type: ResourceType::NonFungible { id_type },
                })
                .unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&taken).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, RecallResourceEvent::Ids(non_fungible_local_ids))?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub fn create_proof<Y>(receiver: &NodeId, api: &mut Y) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let id_type = Self::get_id_type(receiver, api)?;
        let amount = NonFungibleVault::liquid_amount(receiver, api)?
            + NonFungibleVault::locked_amount(receiver, api)?;

        let proof_info = ProofInfoSubstate {
            resource_address,
            resource_type: ResourceType::NonFungible { id_type },
            restricted: false,
        };
        let proof = NonFungibleVault::lock_amount(receiver, amount, api)?;

        let proof_id = api.new_object(
            PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&FungibleProof::default()).unwrap(),
                scrypto_encode(&proof).unwrap(),
            ],
        )?;

        Ok(Proof(Own(proof_id)))
    }

    pub fn create_proof_by_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        if !Self::check_amount(&amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let id_type = Self::get_id_type(receiver, api)?;
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );

        let proof_info = ProofInfoSubstate {
            resource_address,
            resource_type: ResourceType::NonFungible { id_type },
            restricted: false,
        };
        let proof = NonFungibleVault::lock_amount(receiver, amount, api)?;
        let proof_id = api.new_object(
            PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&FungibleProof::default()).unwrap(),
                scrypto_encode(&proof).unwrap(),
            ],
        )?;

        Ok(Proof(Own(proof_id)))
    }

    pub fn create_proof_by_ids<Y>(
        receiver: &NodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(
            api.get_object_info(receiver)?.type_parent.unwrap().into(),
        );
        let id_type = Self::get_id_type(receiver, api)?;

        let proof_info = ProofInfoSubstate {
            resource_address,
            resource_type: ResourceType::NonFungible { id_type },
            restricted: false,
        };
        let proof = NonFungibleVault::lock_non_fungibles(receiver, ids, api)?;
        let proof_id = api.new_object(
            PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&FungibleProof::default()).unwrap(),
                scrypto_encode(&proof).unwrap(),
            ],
        )?;
        Ok(Proof(Own(proof_id)))
    }

    //===================
    // Protected method
    //===================

    // FIXME: set up auth

    pub fn lock_non_fungibles<Y>(
        receiver: &NodeId,
        local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        NonFungibleVault::lock_non_fungibles(receiver, local_ids, api)?;
        Ok(())
    }

    pub fn unlock_non_fungibles<Y>(
        receiver: &NodeId,
        local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        NonFungibleVault::unlock_non_fungibles(receiver, local_ids, api)?;

        Ok(())
    }
}

pub struct NonFungibleVault;

impl NonFungibleVault {
    pub fn liquid_amount<Y>(receiver: &NodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(receiver: &NodeId, api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn is_locked<Y>(receiver: &NodeId, api: &mut Y) -> Result<bool, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        Ok(!Self::locked_amount(receiver, api)?.is_zero())
    }

    pub fn liquid_non_fungible_local_ids<Y>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let ids = substate_ref.ids().clone();
        api.sys_drop_lock(handle)?;
        Ok(ids)
    }

    pub fn locked_non_fungible_local_ids<Y>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let ids = substate_ref.ids();
        api.sys_drop_lock(handle)?;
        Ok(ids)
    }

    pub fn take<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let taken = substate_ref.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_write_substate_typed(handle, &substate_ref)?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Amount(amount))?;

        Ok(taken)
    }

    pub fn take_non_fungibles<Y>(
        receiver: &NodeId,
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let taken = substate_ref
            .take_by_ids(ids)
            .map_err(VaultError::ResourceError)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::VaultError(e)))?;
        api.sys_write_substate_typed(handle, &substate_ref)?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Ids(ids.clone()))?;

        Ok(taken)
    }

    pub fn put<Y>(
        receiver: &NodeId,
        resource: LiquidNonFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let event = DepositResourceEvent::Ids(resource.ids().clone());

        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidNonFungibleResource = api.sys_read_substate_typed(handle)?;
        substate_ref.put(resource).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.sys_write_substate_typed(handle, &substate_ref)?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(())
    }

    // protected method
    pub fn lock_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<NonFungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let max_locked: Decimal = locked.ids.len().into();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            let resource = NonFungibleVault::take(receiver, delta, api)?;

            for nf in resource.into_ids() {
                locked.ids.insert(nf, 0);
            }
        }

        // Increase lock count
        let n: usize = amount
            .to_string()
            .parse()
            .expect("Failed to convert amount to usize");
        let ids_for_proof: BTreeSet<NonFungibleLocalId> =
            locked.ids.keys().cloned().into_iter().take(n).collect();
        for id in &ids_for_proof {
            locked.ids.entry(id.clone()).or_default().add_assign(1);
        }

        api.sys_write_substate_typed(handle, &locked)?;

        // Issue proof
        Ok(NonFungibleProof::new(
            ids_for_proof.clone(),
            btreemap!(
                LocalRef::Vault(Reference(receiver.clone().into())) => ids_for_proof
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?)
    }

    // protected method
    pub fn lock_non_fungibles<Y>(
        receiver: &NodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<NonFungibleProof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;

        // Take from liquid if needed
        let delta: BTreeSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        NonFungibleVault::take_non_fungibles(receiver, &delta, api)?;

        // Increase lock count
        for id in &ids {
            locked.ids.entry(id.clone()).or_default().add_assign(1);
        }

        api.sys_write_substate_typed(handle, &locked)?;

        // Issue proof
        Ok(NonFungibleProof::new(
            ids.clone(),
            btreemap!(
                LocalRef::Vault(Reference(receiver.clone().into()))=> ids
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?)
    }

    // protected method
    pub fn unlock_non_fungibles<Y>(
        receiver: &NodeId,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &VaultOffset::LockedNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;

        let mut liquid_non_fungibles = BTreeSet::<NonFungibleLocalId>::new();
        for id in ids {
            let cnt = locked
                .ids
                .remove(&id)
                .expect("Attempted to unlock non-fungible that was not locked");
            if cnt > 1 {
                locked.ids.insert(id, cnt - 1);
            } else {
                liquid_non_fungibles.insert(id);
            }
        }

        api.sys_write_substate_typed(handle, &locked)?;

        NonFungibleVault::put(
            receiver,
            LiquidNonFungibleResource::new(liquid_non_fungibles),
            api,
        )
    }
}
