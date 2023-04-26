use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::{ClientApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NonFungibleVaultError {
    MissingId(NonFungibleLocalId),
    NotEnoughAmount,
}

pub struct NonFungibleVaultBlueprint;

impl NonFungibleVaultBlueprint {
    fn check_amount(amount: &Decimal) -> bool {
        !amount.is_negative() && amount.0 % BnumI256::from(10i128.pow(18)) == BnumI256::from(0)
    }

    fn get_id_type<Y>(api: &mut Y) -> Result<NonFungibleIdType, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_parent_field(
            NonFungibleResourceManagerOffset::IdType.into(),
            LockFlags::read_only(),
        )?;
        let id_type: NonFungibleIdType = api.sys_read_substate_typed(handle)?;
        Ok(id_type)
    }

    pub fn take<Y>(amount: &Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        // Check amount
        if !Self::check_amount(amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Take
        let taken = NonFungibleVault::take(*amount, api)?;

        // Create node
        NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)
    }

    pub fn take_non_fungibles<Y>(
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Take
        let taken = NonFungibleVault::take_non_fungibles(&non_fungible_local_ids, api)?;

        // Create node
        NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)
    }

    pub fn put<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Drop other bucket
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Put
        NonFungibleVault::put(other_bucket.liquid, api)?;

        Ok(())
    }

    pub fn get_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let amount = NonFungibleVault::liquid_amount(api)? + NonFungibleVault::locked_amount(api)?;

        Ok(amount)
    }

    pub fn get_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut ids = NonFungibleVault::liquid_non_fungible_local_ids(api)?;
        ids.extend(NonFungibleVault::locked_non_fungible_local_ids(api)?);
        Ok(ids)
    }

    pub fn recall<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        if !Self::check_amount(&amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let taken = NonFungibleVault::take(amount, api)?;

        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)?;

        Runtime::emit_event(api, RecallResourceEvent::Amount(amount))?;

        Ok(bucket)
    }

    pub fn recall_non_fungibles<Y>(
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let taken = NonFungibleVault::take_non_fungibles(&non_fungible_local_ids, api)?;

        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(taken.into_ids(), api)?;

        Runtime::emit_event(api, RecallResourceEvent::Ids(non_fungible_local_ids))?;

        Ok(bucket)
    }

    pub fn create_proof<Y>(receiver: &NodeId, api: &mut Y) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let resource_address =
            ResourceAddress::new_unchecked(api.get_info()?.blueprint_parent.unwrap().into());
        let id_type = Self::get_id_type(api)?;
        let amount = NonFungibleVault::liquid_amount(api)? + NonFungibleVault::locked_amount(api)?;

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
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        if !Self::check_amount(&amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let id_type = Self::get_id_type(api)?;
        let resource_address =
            ResourceAddress::new_unchecked(api.get_info()?.blueprint_parent.unwrap().into());

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
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let resource_address =
            ResourceAddress::new_unchecked(api.get_info()?.blueprint_parent.unwrap().into());
        let id_type = Self::get_id_type(api)?;

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
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        NonFungibleVault::lock_non_fungibles(receiver, local_ids, api)?;
        Ok(())
    }

    pub fn unlock_non_fungibles<Y>(
        local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        NonFungibleVault::unlock_non_fungibles(local_ids, api)?;

        Ok(())
    }
}

pub struct NonFungibleVault;

impl NonFungibleVault {
    pub fn liquid_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LiquidNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidNonFungibleVault = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount;
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn locked_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LockedNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub fn liquid_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LiquidNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LiquidNonFungibleVault = api.sys_read_substate_typed(handle)?;

        let items: Vec<NonFungibleLocalId> = api.scan_typed_index(&substate_ref.ids.0, u32::MAX)?;
        let ids = items.into_iter().collect();
        api.sys_drop_lock(handle)?;
        Ok(ids)
    }

    pub fn locked_non_fungible_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LockedNonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let ids = substate_ref.ids();
        api.sys_drop_lock(handle)?;
        Ok(ids)
    }

    pub fn take<Y>(amount: Decimal, api: &mut Y) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidNonFungibleVault = api.sys_read_substate_typed(handle)?;

        // deduct from liquidity pool
        if substate_ref.amount < amount {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleVaultError(NonFungibleVaultError::NotEnoughAmount),
            ));
        }

        // TODO: Fix/Cleanup
        if substate_ref.amount > Decimal::from(u32::MAX) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::ResourceError(
                    ResourceError::InvalidTakeAmount,
                )),
            ));
        }

        substate_ref.amount -= amount;

        let amount_to_take: u32 = amount
            .to_string()
            .parse()
            .expect("Failed to convert amount to u32");

        let taken = {
            let ids: Vec<NonFungibleLocalId> =
                api.take_typed(substate_ref.ids.as_node_id(), amount_to_take)?;
            LiquidNonFungibleResource {
                ids: ids.into_iter().collect(),
            }
        };

        api.sys_write_substate_typed(handle, &substate_ref)?;
        api.sys_drop_lock(handle)?;

        Runtime::emit_event(api, WithdrawResourceEvent::Amount(amount))?;

        Ok(taken)
    }

    pub fn take_non_fungibles<Y>(
        ids: &BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref: LiquidNonFungibleVault = api.sys_read_substate_typed(handle)?;

        substate_ref.amount -= Decimal::from(ids.len());

        // TODO: Batch remove
        for id in ids {
            let removed =
                api.remove_from_index(substate_ref.ids.as_node_id(), scrypto_encode(id).unwrap())?;

            if removed.is_none() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleVaultError(NonFungibleVaultError::MissingId(
                        id.clone(),
                    )),
                ));
            }
        }

        Runtime::emit_event(api, WithdrawResourceEvent::Ids(ids.clone()))?;
        api.sys_drop_lock(handle)?;

        Ok(LiquidNonFungibleResource::new(ids.clone()))
    }

    pub fn put<Y>(resource: LiquidNonFungibleResource, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let event = DepositResourceEvent::Ids(resource.ids().clone());

        let handle = api.lock_field(
            NonFungibleVaultOffset::LiquidNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut vault: LiquidNonFungibleVault = api.sys_read_substate_typed(handle)?;

        vault.amount += Decimal::from(resource.ids.len());

        // update liquidity
        // TODO: Batch update
        // TODO: Rather than insert, use create_unique?
        for id in resource.ids {
            api.insert_typed_into_index(vault.ids.as_node_id(), scrypto_encode(&id).unwrap(), id)?;
        }

        api.sys_write_substate_typed(handle, &vault)?;
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
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LockedNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;
        let max_locked: Decimal = locked.ids.len().into();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount - max_locked;
            let resource = NonFungibleVault::take(delta, api)?;

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
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LockedNonFungible.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked: LockedNonFungibleResource = api.sys_read_substate_typed(handle)?;

        // Take from liquid if needed
        let delta: BTreeSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        NonFungibleVault::take_non_fungibles(&delta, api)?;

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
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleVaultOffset::LockedNonFungible.into(),
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

        NonFungibleVault::put(LiquidNonFungibleResource::new(liquid_non_fungibles), api)
    }
}
