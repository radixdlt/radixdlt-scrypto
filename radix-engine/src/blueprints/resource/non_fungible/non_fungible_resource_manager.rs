use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::heap::DroppedNonFungibleProof;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::schema::InstanceSchema;
use radix_engine_interface::types::NodeId;
use radix_engine_interface::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NonFungibleResourceManagerError {
    NonFungibleAlreadyExists(Box<NonFungibleGlobalId>),
    NonFungibleNotFound(Box<NonFungibleGlobalId>),
    InvalidField(String),
    FieldNotMutable(String),
    NonFungibleIdTypeDoesNotMatch(NonFungibleIdType, NonFungibleIdType),
    InvalidNonFungibleIdType,
    NonFungibleLocalIdProvidedForUUIDType,
    DropNonEmptyBucket,
}

pub type NonFungibleResourceManagerIdTypeSubstate = NonFungibleIdType;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMutableFieldsSubstate {
    pub mutable_fields: BTreeSet<String>, // TODO: Integrate with KeyValueStore schema check?
}

pub type NonFungibleResourceManagerTotalSupplySubstate = Decimal;

fn create_non_fungibles<Y>(
    resource_address: ResourceAddress,
    id_type: NonFungibleIdType,
    entries: BTreeMap<NonFungibleLocalId, ScryptoValue>,
    check_non_existence: bool,
    api: &mut Y,
) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let mut ids = BTreeSet::new();
    for (non_fungible_local_id, value) in entries {
        if non_fungible_local_id.id_type() != id_type {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleIdTypeDoesNotMatch(
                        non_fungible_local_id.id_type(),
                        id_type,
                    ),
                ),
            ));
        }

        let non_fungible_handle =
            api.actor_lock_key_value_entry(&non_fungible_local_id.to_key(), LockFlags::MUTABLE)?;

        if check_non_existence {
            let cur_non_fungible: Option<ScryptoValue> =
                api.key_value_entry_get_typed(non_fungible_handle)?;

            if let Some(..) = cur_non_fungible {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::NonFungibleAlreadyExists(Box::new(
                            NonFungibleGlobalId::new(resource_address, non_fungible_local_id),
                        )),
                    ),
                ));
            }
        }

        api.key_value_entry_set_typed(non_fungible_handle, value)?;
        api.key_value_entry_lock_release(non_fungible_handle)?;
        ids.insert(non_fungible_local_id);
    }

    Ok(())
}

pub struct NonFungibleResourceManagerBlueprint;

impl NonFungibleResourceManagerBlueprint {
    pub(crate) fn create<Y>(
        id_type: NonFungibleIdType,
        non_fungible_schema: NonFungibleDataSchema,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalNonFungibleResource)?;
        let resource_address = ResourceAddress::new_or_panic(global_node_id.into());
        Self::create_with_address(
            id_type,
            non_fungible_schema,
            metadata,
            access_rules,
            resource_address.into(),
            api,
        )
    }

    pub(crate) fn create_with_address<Y>(
        id_type: NonFungibleIdType,
        non_fungible_schema: NonFungibleDataSchema,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        resource_address: [u8; NodeId::LENGTH], // TODO: Clean this up
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mutable_fields = NonFungibleResourceManagerMutableFieldsSubstate {
            mutable_fields: non_fungible_schema.mutable_fields,
        };

        let instance_schema = InstanceSchema {
            schema: non_fungible_schema.schema,
            type_index: vec![non_fungible_schema.non_fungible],
        };

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            Some(instance_schema),
            vec![
                scrypto_encode(&id_type).unwrap(),
                scrypto_encode(&mutable_fields).unwrap(),
                scrypto_encode(&Decimal::zero()).unwrap(),
            ],
            vec![vec![]],
        )?;

        let resource_address = ResourceAddress::new_or_panic(resource_address);
        globalize_resource_manager(object_id, resource_address, access_rules, metadata, api)?;

        Ok(resource_address)
    }

    pub(crate) fn create_with_initial_supply<Y>(
        id_type: NonFungibleIdType,
        non_fungible_schema: NonFungibleDataSchema,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        // TODO: Do this check in a better way (e.g. via type check)
        if id_type == NonFungibleIdType::UUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleLocalIdProvidedForUUIDType,
                ),
            ));
        }

        let mutable_fields = NonFungibleResourceManagerMutableFieldsSubstate {
            mutable_fields: non_fungible_schema.mutable_fields,
        };

        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalNonFungibleResource)?;
        let resource_address = ResourceAddress::new_or_panic(global_node_id.into());

        let supply: Decimal = Decimal::from(entries.len());

        let ids = entries.keys().cloned().collect();

        let mut non_fungibles = Vec::new();
        for (id, (value,)) in entries {
            if id.id_type() != id_type {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::NonFungibleIdTypeDoesNotMatch(
                            id.id_type(),
                            id_type,
                        ),
                    ),
                ));
            }

            non_fungibles.push((
                scrypto_encode(&id).unwrap(),
                scrypto_encode(&value).unwrap(),
            ));
        }

        let instance_schema = InstanceSchema {
            schema: non_fungible_schema.schema,
            type_index: vec![non_fungible_schema.non_fungible],
        };

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            Some(instance_schema),
            vec![
                scrypto_encode(&id_type).unwrap(),
                scrypto_encode(&mutable_fields).unwrap(),
                scrypto_encode(&supply).unwrap(),
            ],
            vec![non_fungibles],
        )?;
        let bucket = globalize_non_fungible_with_initial_supply(
            object_id,
            resource_address,
            access_rules,
            metadata,
            ids,
            api,
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn create_uuid_with_initial_supply<Y>(
        non_fungible_schema: NonFungibleDataSchema,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        entries: Vec<(ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let mut ids = BTreeSet::new();
        let mut non_fungibles = Vec::new();
        let supply = Decimal::from(entries.len());
        for (entry,) in entries {
            let uuid = Runtime::generate_uuid(api)?;
            let id = NonFungibleLocalId::uuid(uuid).unwrap();
            ids.insert(id.clone());
            non_fungibles.push((
                scrypto_encode(&id).unwrap(),
                scrypto_encode(&entry).unwrap(),
            ));
        }

        let mutable_fields = NonFungibleResourceManagerMutableFieldsSubstate {
            mutable_fields: non_fungible_schema.mutable_fields,
        };

        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalNonFungibleResource)?;
        let resource_address = ResourceAddress::new_or_panic(global_node_id.into());

        let instance_schema = InstanceSchema {
            schema: non_fungible_schema.schema,
            type_index: vec![non_fungible_schema.non_fungible],
        };

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            Some(instance_schema),
            vec![
                scrypto_encode(&NonFungibleIdType::UUID).unwrap(),
                scrypto_encode(&mutable_fields).unwrap(),
                scrypto_encode(&supply).unwrap(),
            ],
            vec![non_fungibles],
        )?;
        let bucket = globalize_non_fungible_with_initial_supply(
            object_id,
            resource_address,
            access_rules,
            metadata,
            ids,
            api,
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint_non_fungible<Y>(
        entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_or_panic(api.get_global_address()?.into());
        let id_type = {
            let handle = api.lock_field(
                NonFungibleResourceManagerOffset::IdType.into(),
                LockFlags::read_only(),
            )?;
            let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;
            api.field_lock_release(handle)?;
            if id_type == NonFungibleIdType::UUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }
            id_type
        };

        // Update total supply
        {
            let total_supply_handle = api.lock_field(
                NonFungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            let amount: Decimal = entries.len().into();
            total_supply += amount;
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
        }

        let ids = {
            let ids: BTreeSet<NonFungibleLocalId> = entries.keys().cloned().collect();
            let non_fungibles = entries.into_iter().map(|(k, v)| (k, v.0)).collect();
            create_non_fungibles(resource_address, id_type, non_fungibles, true, api)?;

            ids
        };

        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok(bucket)
    }

    pub(crate) fn mint_single_uuid_non_fungible<Y>(
        value: ScryptoValue,
        api: &mut Y,
    ) -> Result<(Bucket, NonFungibleLocalId), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_or_panic(api.get_global_address()?.into());

        // Check id_type
        let id_type = {
            let id_type_handle = api.lock_field(
                NonFungibleResourceManagerOffset::IdType.into(),
                LockFlags::MUTABLE,
            )?;
            let id_type: NonFungibleIdType = api.field_lock_read_typed(id_type_handle)?;
            api.field_lock_release(id_type_handle)?;

            if id_type != NonFungibleIdType::UUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }

            id_type
        };

        // Update Total Supply
        {
            let total_supply_handle = api.lock_field(
                NonFungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            total_supply += 1;
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
        }

        let id = {
            // TODO: Is this enough bits to prevent hash collisions?
            // TODO: Possibly use an always incrementing timestamp
            let id = NonFungibleLocalId::uuid(Runtime::generate_uuid(api)?).unwrap();
            let non_fungibles = btreemap!(id.clone() => value);

            create_non_fungibles(resource_address, id_type, non_fungibles, false, api)?;

            id
        };

        let ids = btreeset!(id.clone());
        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok((bucket, id))
    }

    pub(crate) fn mint_uuid_non_fungible<Y>(
        entries: Vec<(ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_or_panic(api.get_global_address()?.into());

        // Check type
        let id_type = {
            let handle = api.lock_field(
                NonFungibleResourceManagerOffset::IdType.into(),
                LockFlags::MUTABLE,
            )?;
            let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;
            api.field_lock_release(handle)?;

            if id_type != NonFungibleIdType::UUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }
            id_type
        };

        // Update total supply
        {
            let total_supply_handle = api.lock_field(
                NonFungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            let amount: Decimal = entries.len().into();
            total_supply += amount;
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
        }

        // Update data
        let ids = {
            let mut ids = BTreeSet::new();
            let mut non_fungibles = BTreeMap::new();
            for value in entries {
                let id = NonFungibleLocalId::uuid(Runtime::generate_uuid(api)?).unwrap();
                ids.insert(id.clone());
                non_fungibles.insert(id, value.0);
            }
            create_non_fungibles(resource_address, id_type, non_fungibles, false, api)?;

            ids
        };

        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok(bucket)
    }

    pub(crate) fn update_non_fungible_data<Y>(
        id: NonFungibleLocalId,
        field_name: String,
        data: ScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_or_panic(api.get_global_address()?.into());
        let data_schema_handle = api.lock_field(
            NonFungibleResourceManagerOffset::DataSchema.into(),
            LockFlags::read_only(),
        )?;
        let mutable_fields: NonFungibleResourceManagerMutableFieldsSubstate =
            api.field_lock_read_typed(data_schema_handle)?;

        let mut instance_schema = api.get_info()?.instance_schema.unwrap();
        let kv_schema = instance_schema.schema;
        let local_index = instance_schema.type_index.remove(0);

        let mutable_fields = mutable_fields.mutable_fields;

        let schema_path = SchemaPath(vec![SchemaSubPath::Field(field_name.clone())]);

        let sbor_path = schema_path.to_sbor_path(&kv_schema, local_index);
        let sbor_path = if let Some((sbor_path, ..)) = sbor_path {
            sbor_path
        } else {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidField(field_name),
                ),
            ));
        };

        if !mutable_fields.contains(&field_name) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::FieldNotMutable(field_name),
                ),
            ));
        }

        let non_fungible_handle =
            api.actor_lock_key_value_entry(&id.to_key(), LockFlags::MUTABLE)?;

        let mut non_fungible_entry: Option<ScryptoValue> =
            api.key_value_entry_get_typed(non_fungible_handle)?;

        if let Some(ref mut non_fungible) = non_fungible_entry {
            let value = sbor_path.get_from_value_mut(non_fungible).unwrap();
            *value = data;
            let buffer = scrypto_encode(non_fungible).unwrap();

            api.key_value_entry_set(non_fungible_handle, buffer)?;
        } else {
            let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, id);
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleNotFound(Box::new(
                        non_fungible_global_id,
                    )),
                ),
            ));
        }

        api.key_value_entry_lock_release(non_fungible_handle)?;

        Ok(())
    }

    pub(crate) fn non_fungible_exists<Y>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let non_fungible_handle =
            api.actor_lock_key_value_entry(&id.to_key(), LockFlags::read_only())?;
        let non_fungible: Option<ScryptoValue> =
            api.key_value_entry_get_typed(non_fungible_handle)?;
        let exists = matches!(non_fungible, Option::Some(..));

        Ok(exists)
    }

    pub(crate) fn get_non_fungible<Y>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_or_panic(api.get_global_address()?.into());

        let non_fungible_handle =
            api.actor_lock_key_value_entry(&id.to_key(), LockFlags::read_only())?;
        let wrapper: Option<ScryptoValue> = api.key_value_entry_get_typed(non_fungible_handle)?;
        if let Some(non_fungible) = wrapper {
            Ok(non_fungible)
        } else {
            let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, id.clone());
            Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleNotFound(Box::new(
                        non_fungible_global_id,
                    )),
                ),
            ))
        }
    }

    pub(crate) fn create_empty_bucket<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::create_bucket(BTreeSet::new(), api)
    }

    pub(crate) fn create_bucket<Y>(
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket_id = api.new_simple_object(
            NON_FUNGIBLE_BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn burn<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Drop the bucket
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Construct the event and only emit it once all of the operations are done.
        Runtime::emit_event(
            api,
            BurnNonFungibleResourceEvent {
                ids: other_bucket.liquid.ids().clone(),
            },
        )?;

        // Update total supply
        // TODO: there might be better for maintaining total supply, especially for non-fungibles
        {
            let total_supply_handle = api.lock_field(
                NonFungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            total_supply -= other_bucket.liquid.amount();
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
        }

        // Update
        {
            for id in other_bucket.liquid.into_ids() {
                api.actor_key_value_entry_remove(&id.to_key())?;
            }
        }

        Ok(())
    }

    pub(crate) fn drop_empty_bucket<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;

        if other_bucket.liquid.amount().is_zero() {
            Ok(())
        } else {
            Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::DropNonEmptyBucket,
                ),
            ))
        }
    }

    pub(crate) fn create_vault<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let ids = Own(api.new_index()?);
        let vault = LiquidNonFungibleVault {
            amount: Decimal::zero(),
            ids,
        };
        let vault_id = api.new_simple_object(
            NON_FUNGIBLE_VAULT_BLUEPRINT,
            vec![
                scrypto_encode(&vault).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(api: &mut Y) -> Result<ResourceType, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleResourceManagerOffset::IdType.into(),
            LockFlags::read_only(),
        )?;

        let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;
        let resource_type = ResourceType::NonFungible { id_type };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            NonFungibleResourceManagerOffset::TotalSupply.into(),
            LockFlags::read_only(),
        )?;
        let total_supply: Decimal = api.field_lock_read_typed(handle)?;
        Ok(total_supply)
    }

    pub(crate) fn drop_proof<Y>(proof: Proof, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let node_substates = api.drop_object(proof.0.as_node_id())?;
        let dropped_proof: DroppedNonFungibleProof = node_substates.into();
        dropped_proof.non_fungible_proof.drop_proof(api)?;

        Ok(())
    }
}
