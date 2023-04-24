use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::heap::DroppedBucket;
use crate::kernel::heap::DroppedBucketResource;
use crate::kernel::kernel_api::{KernelNodeApi};
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::schema::KeyValueStoreSchema;
use radix_engine_interface::types::NodeId;
use radix_engine_interface::*;
use sbor::rust::borrow::Cow;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NonFungibleResourceManagerError {
    NonFungibleAlreadyExists(Box<NonFungibleGlobalId>),
    NonFungibleNotFound(Box<NonFungibleGlobalId>),
    InvalidField(String),
    FieldNotMutable(String),
    MismatchingBucketResource,
    NonFungibleIdTypeDoesNotMatch(NonFungibleIdType, NonFungibleIdType),
    InvalidNonFungibleIdType,
}

pub type NonFungibleResourceManagerIdTypeSubstate = NonFungibleIdType;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct NonFungibleResourceManagerDataSchemaSubstate {
    pub non_fungible_type_index: LocalTypeIndex,
    pub mutable_fields: BTreeSet<String>, // TODO: Integrate with KeyValueStore schema check?
}

pub type NonFungibleResourceManagerTotalSupplySubstate = Decimal;

pub type NonFungibleResourceManagerDataSubstate = Own;

fn build_non_fungible_resource_manager_data_substate<Y>(
    non_fungible_schema: NonFungibleDataSchema,
    api: &mut Y,
) -> Result<(NonFungibleResourceManagerDataSchemaSubstate, NodeId), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let non_fungible_type = aggregator.add_child_type_and_descendents::<NonFungibleLocalId>();
    let key_schema = generate_full_schema(aggregator);

    let mut kv_schema = non_fungible_schema.schema;

    // Key
    kv_schema.type_kinds.extend(key_schema.type_kinds);

    // Optional Value
    {
        let mut variants = BTreeMap::new();
        variants.insert(OPTION_VARIANT_NONE, vec![]);
        variants.insert(OPTION_VARIANT_SOME, vec![non_fungible_schema.non_fungible]);
        let type_kind = TypeKind::Enum { variants };
        kv_schema.type_kinds.push(type_kind);
    }

    // Key
    kv_schema.type_metadata.extend(key_schema.type_metadata);

    // Optional value
    {
        let metadata = TypeMetadata {
            type_name: Some(Cow::Borrowed("Option")),
            child_names: Some(ChildNames::EnumVariants(btreemap!(
                OPTION_VARIANT_NONE => TypeMetadata::no_child_names("None"),
                OPTION_VARIANT_SOME => TypeMetadata::no_child_names("Some"),
            ))),
        };
        kv_schema.type_metadata.push(metadata);
    }

    // Key
    kv_schema
        .type_validations
        .extend(key_schema.type_validations);

    // Optional value
    kv_schema.type_validations.push(TypeValidation::None);
    let value_index = LocalTypeIndex::SchemaLocalIndex(kv_schema.type_validations.len() - 1);

    let kv_schema = KeyValueStoreSchema {
        schema: kv_schema,
        key: non_fungible_type,
        value: value_index,
        can_own: false, // Only allow NonFungibles to store data/references
    };

    let nf_store_id = api.key_value_store_new(kv_schema)?;

    let update_data_substate = NonFungibleResourceManagerDataSchemaSubstate {
        non_fungible_type_index: non_fungible_schema.non_fungible,
        mutable_fields: non_fungible_schema.mutable_fields,
    };

    Ok((update_data_substate, nf_store_id))
}

fn build_non_fungible_bucket<Y>(
    resource_address: ResourceAddress,
    id_type: NonFungibleIdType,
    nf_store_id: NodeId,
    entries: BTreeMap<NonFungibleLocalId, ScryptoValue>,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let bucket = {
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

            let non_fungible_handle = api.key_value_store_lock_entry(
                &nf_store_id,
                &non_fungible_local_id.to_key(),
                LockFlags::MUTABLE,
            )?;

            // TODO: Change interface so that we accept Option instead
            api.key_value_entry_set_typed(non_fungible_handle, Some(value))?;
            api.key_value_entry_lock_release(non_fungible_handle)?;
            ids.insert(non_fungible_local_id);
        }

        let info = BucketInfoSubstate {
            resource_address,
            resource_type: ResourceType::NonFungible { id_type },
        };
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&info).unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Bucket(Own(bucket_id))
    };

    Ok(bucket)
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
        let resource_address = ResourceAddress::new_unchecked(global_node_id.into());
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
        // If address isn't user frame allocated or pre_allocated then
        // using this node_id will fail on create_node below
        let (resource_manager_substate, nf_store) =
            build_non_fungible_resource_manager_data_substate(non_fungible_schema, api)?;

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&id_type).unwrap(),
                scrypto_encode(&resource_manager_substate).unwrap(),
                scrypto_encode(&Decimal::zero()).unwrap(),
                scrypto_encode(&Own(nf_store)).unwrap(),
            ],
        )?;

        let resource_address = ResourceAddress::new_unchecked(resource_address);
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
                    NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                ),
            ));
        }

        let (resource_manager, nf_store_id) =
            build_non_fungible_resource_manager_data_substate(non_fungible_schema, api)?;

        let entries: BTreeMap<NonFungibleLocalId, ScryptoValue> = entries
            .into_iter()
            .map(|(id, (value,))| (id, value))
            .collect();

        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalNonFungibleResource)?;
        let resource_address = ResourceAddress::new_unchecked(global_node_id.into());

        let supply: Decimal = Decimal::from(entries.len());
        let bucket =
            build_non_fungible_bucket(resource_address, id_type, nf_store_id, entries, api)?;

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&id_type).unwrap(),
                scrypto_encode(&resource_manager).unwrap(),
                scrypto_encode(&supply).unwrap(),
                scrypto_encode(&Own(nf_store_id)).unwrap(),
            ],
        )?;

        globalize_resource_manager(object_id, resource_address, access_rules, metadata, api)?;

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
        let mut non_fungible_entries = BTreeMap::new();
        for (entry,) in entries {
            let uuid = Runtime::generate_uuid(api)?;
            let id = NonFungibleLocalId::uuid(uuid).unwrap();
            non_fungible_entries.insert(id, entry);
        }

        let (data, nf_store_id) =
            build_non_fungible_resource_manager_data_substate(non_fungible_schema, api)?;

        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalNonFungibleResource)?;
        let resource_address = ResourceAddress::new_unchecked(global_node_id.into());

        let supply = Decimal::from(non_fungible_entries.len());
        let bucket = build_non_fungible_bucket(
            resource_address,
            NonFungibleIdType::UUID,
            nf_store_id,
            non_fungible_entries,
            api,
        )?;

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&NonFungibleIdType::UUID).unwrap(),
                scrypto_encode(&data).unwrap(),
                scrypto_encode(&supply).unwrap(),
                scrypto_encode(&Own(nf_store_id)).unwrap(),
            ],
        )?;

        globalize_resource_manager(object_id, resource_address, access_rules, metadata, api)?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint_non_fungible<Y>(
        entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let handle = api.lock_field(
            NonFungibleResourceManagerOffset::IdType.into(),
            LockFlags::MUTABLE,
        )?;
        let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;

        let (bucket_id, non_fungibles) = {
            if id_type == NonFungibleIdType::UUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }

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

            // Allocate non-fungibles
            let mut ids = BTreeSet::new();
            let mut non_fungibles = BTreeMap::new();
            for (id, (non_fungible,)) in entries.clone().into_iter() {
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

                ids.insert(id.clone());
                non_fungibles.insert(id, non_fungible);
            }

            let info = BucketInfoSubstate {
                resource_address,
                resource_type: ResourceType::NonFungible { id_type },
            };
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&info).unwrap(),
                    scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            (bucket_id, non_fungibles)
        };

        let data_handle = api.lock_field(
            NonFungibleResourceManagerOffset::Data.into(),
            LockFlags::read_only(),
        )?;
        let data: Own = api.field_lock_read_typed(data_handle)?;

        for (id, non_fungible) in non_fungibles {
            let non_fungible_handle = api.key_value_store_lock_entry(
                data.as_node_id(),
                &id.to_key(),
                LockFlags::MUTABLE,
            )?;

            {
                let cur_non_fungible: Option<ScryptoValue> =
                    api.key_value_entry_get_typed(non_fungible_handle)?;

                if let Some(..) = cur_non_fungible {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::NonFungibleResourceManagerError(
                            NonFungibleResourceManagerError::NonFungibleAlreadyExists(Box::new(
                                NonFungibleGlobalId::new(resource_address, id),
                            )),
                        ),
                    ));
                }

                api.key_value_entry_set_typed(non_fungible_handle, Some(non_fungible))?;
            }

            api.key_value_entry_lock_release(non_fungible_handle)?;
        }

        Runtime::emit_event(
            api,
            MintNonFungibleResourceEvent {
                ids: entries.into_iter().map(|(k, _)| k).collect(),
            },
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn mint_single_uuid_non_fungible<Y>(
        value: ScryptoValue,
        api: &mut Y,
    ) -> Result<(Bucket, NonFungibleLocalId), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let data_handle = api.lock_field(
            NonFungibleResourceManagerOffset::Data.into(),
            LockFlags::read_only(),
        )?;
        let nf_store: Own = api.field_lock_read_typed(data_handle)?;

        let id_type_handle = api.lock_field(
            NonFungibleResourceManagerOffset::IdType.into(),
            LockFlags::MUTABLE,
        )?;
        let id_type: NonFungibleIdType = api.field_lock_read_typed(id_type_handle)?;

        if id_type != NonFungibleIdType::UUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                ),
            ));
        }

        // Update Total Supply
        {
            let total_supply_handle = api.lock_field(
                NonFungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            total_supply -= 1;
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
        }

        // TODO: Is this enough bits to prevent hash collisions?
        // TODO: Possibly use an always incrementing timestamp
        let uuid = Runtime::generate_uuid(api)?;
        let id = NonFungibleLocalId::uuid(uuid).unwrap();

        {
            let non_fungible_handle = api.key_value_store_lock_entry(
                nf_store.as_node_id(),
                &id.to_key(),
                LockFlags::MUTABLE,
            )?;
            api.key_value_entry_set_typed(non_fungible_handle, Some(value))?;

            api.key_value_entry_lock_release(non_fungible_handle)?;
        }

        let info = BucketInfoSubstate {
            resource_address,
            resource_type: ResourceType::NonFungible { id_type },
        };
        let ids = BTreeSet::from([id.clone()]);
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&info).unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&LiquidNonFungibleResource::new(ids.clone())).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok((Bucket(Own(bucket_id)), id))
    }

    pub(crate) fn mint_uuid_non_fungible<Y>(
        entries: Vec<(ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());

        let (bucket_id, ids) = {
            let data_handle = api.lock_field(
                NonFungibleResourceManagerOffset::Data.into(),
                LockFlags::read_only(),
            )?;
            let nf_store: Own = api.field_lock_read_typed(data_handle)?;

            let handle = api.lock_field(
                NonFungibleResourceManagerOffset::IdType.into(),
                LockFlags::MUTABLE,
            )?;
            let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;

            if id_type != NonFungibleIdType::UUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }

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

            // Allocate non-fungibles
            let mut ids = BTreeSet::new();
            for (value,) in entries {
                // TODO: Is this enough bits to prevent hash collisions?
                // TODO: Possibly use an always incrementing timestamp
                let uuid = Runtime::generate_uuid(api)?;
                let id = NonFungibleLocalId::uuid(uuid).unwrap();
                ids.insert(id.clone());

                {
                    let non_fungible_handle = api.key_value_store_lock_entry(
                        nf_store.as_node_id(),
                        &id.to_key(),
                        LockFlags::MUTABLE,
                    )?;
                    api.key_value_entry_set_typed(non_fungible_handle, Some(value))?;

                    api.key_value_entry_lock_release(non_fungible_handle)?;
                }
            }

            let info = BucketInfoSubstate {
                resource_address,
                resource_type: ResourceType::NonFungible { id_type },
            };
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&info).unwrap(),
                    scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&LiquidNonFungibleResource::new(ids.clone())).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            (bucket_id, ids)
        };

        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok(Bucket(Own(bucket_id)))
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
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let data_schema_handle = api.lock_field(
            NonFungibleResourceManagerOffset::DataSchema.into(),
            LockFlags::read_only(),
        )?;
        let data_schema: NonFungibleResourceManagerDataSchemaSubstate =
            api.field_lock_read_typed(data_schema_handle)?;
        let non_fungible_type_index = data_schema.non_fungible_type_index;
        let mutable_fields = data_schema.mutable_fields.clone();

        let nf_store_handle = api.lock_field(
            NonFungibleResourceManagerOffset::Data.into(),
            LockFlags::read_only(),
        )?;
        let nf_store: Own = api.field_lock_read_typed(nf_store_handle)?;

        let kv_schema = api.key_value_store_get_info(nf_store.as_node_id())?;
        let schema_path = SchemaPath(vec![SchemaSubPath::Field(field_name.clone())]);
        let sbor_path = schema_path.to_sbor_path(&kv_schema.schema, non_fungible_type_index);
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

        let non_fungible_handle = api.key_value_store_lock_entry(
            nf_store.as_node_id(),
            &id.to_key(),
            LockFlags::MUTABLE,
        )?;

        let mut non_fungible_entry: Option<ScryptoValue> =
            api.key_value_entry_get_typed(non_fungible_handle)?;

        if let Some(ref mut non_fungible) = non_fungible_entry {
            let value = sbor_path.get_from_value_mut(non_fungible).unwrap();
            *value = data;

            api.key_value_entry_set_typed(non_fungible_handle, &non_fungible_entry)?;
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
        let data_handle = api.lock_field(
            NonFungibleResourceManagerOffset::Data.into(),
            LockFlags::read_only(),
        )?;

        let nf_store: Own = api.field_lock_read_typed(data_handle)?;

        let non_fungible_handle = api.key_value_store_lock_entry(
            nf_store.as_node_id(),
            &id.to_key(),
            LockFlags::read_only(),
        )?;
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
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let data_handle = api.lock_field(
            NonFungibleResourceManagerOffset::Data.into(),
            LockFlags::read_only(),
        )?;

        let nf_store: Own = api.field_lock_read_typed(data_handle)?;

        let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, id.clone());

        let non_fungible_handle = api.key_value_store_lock_entry(
            nf_store.as_node_id(),
            &id.to_key(),
            LockFlags::read_only(),
        )?;
        let wrapper: Option<ScryptoValue> = api.key_value_entry_get_typed(non_fungible_handle)?;
        if let Some(non_fungible) = wrapper {
            Ok(non_fungible)
        } else {
            Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleNotFound(Box::new(
                        non_fungible_global_id,
                    )),
                ),
            ))
        }
    }

    pub(crate) fn create_bucket<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let handle = api.lock_field(
            NonFungibleResourceManagerOffset::IdType.into(),
            LockFlags::MUTABLE,
        )?;
        let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;

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
                scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn burn<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        // FIXME: check if the bucket is locked
        let dropped_bucket: DroppedBucket = api.kernel_drop_node(bucket.0.as_node_id())?.into();

        // Construct the event and only emit it once all of the operations are done.
        match dropped_bucket.resource {
            DroppedBucketResource::Fungible(..) => {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::MismatchingBucketResource,
                    ),
                ));
            }
            DroppedBucketResource::NonFungible(resource) => {
                Runtime::emit_event(
                    api,
                    BurnNonFungibleResourceEvent {
                        ids: resource.ids().clone(),
                    },
                )?;

                // Check if resource matches
                // TODO: Move this check into actor check
                {
                    let data_handle = api.lock_field(
                        NonFungibleResourceManagerOffset::Data.into(),
                        LockFlags::read_only(),
                    )?;
                    let resource_address =
                        ResourceAddress::new_unchecked(api.get_global_address()?.into());
                    let nf_store: Own = api.field_lock_read_typed(data_handle)?;
                    if dropped_bucket.info.resource_address != resource_address {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::NonFungibleResourceManagerError(
                                NonFungibleResourceManagerError::MismatchingBucketResource,
                            ),
                        ));
                    }

                    // Update total supply
                    // TODO: there might be better for maintaining total supply, especially for non-fungibles
                    {
                        let total_supply_handle = api.lock_field(
                            NonFungibleResourceManagerOffset::TotalSupply.into(),
                            LockFlags::MUTABLE,
                        )?;
                        let mut total_supply: Decimal =
                            api.field_lock_read_typed(total_supply_handle)?;
                        total_supply -= resource.amount();
                        api.field_lock_write_typed(total_supply_handle, &total_supply)?;
                    }

                    for id in resource.into_ids() {
                        let non_fungible_handle = api.key_value_store_lock_entry(
                            nf_store.as_node_id(),
                            &id.to_key(),
                            LockFlags::MUTABLE,
                        )?;

                        api.key_value_entry_set_typed(non_fungible_handle, None::<ScryptoValue>)?;
                        api.key_value_entry_lock_release(non_fungible_handle)?;
                    }
                }
            }
        }

        Ok(())
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
        let vault_id = api.new_object(
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
}
