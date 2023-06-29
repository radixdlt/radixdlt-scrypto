use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::{ClientApi, CollectionIndex, KVEntry, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::schema::InstanceSchema;
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
    NonFungibleLocalIdProvidedForRUIDType,
    DropNonEmptyBucket,
    NotMintable,
    NotBurnable,
}

pub type NonFungibleResourceManagerIdTypeSubstate = NonFungibleIdType;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMutableFieldsSubstate {
    pub mutable_fields: BTreeSet<String>, // FIXME double check the behavior of invalid field name
}

pub type NonFungibleResourceManagerTotalSupplySubstate = Decimal;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE: CollectionIndex = 0u8;

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

        let non_fungible_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE,
            &non_fungible_local_id.to_key(),
            LockFlags::MUTABLE,
        )?;

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
        api.key_value_entry_release(non_fungible_handle)?;
        ids.insert(non_fungible_local_id);
    }

    Ok(())
}

pub struct NonFungibleResourceManagerBlueprint;

impl NonFungibleResourceManagerBlueprint {
    pub(crate) fn create<Y>(
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        let mutable_fields = NonFungibleResourceManagerMutableFieldsSubstate {
            mutable_fields: non_fungible_schema.mutable_fields,
        };

        let instance_schema = InstanceSchema {
            schema: non_fungible_schema.schema,
            type_index: vec![non_fungible_schema.non_fungible],
        };

        let features = features(track_total_supply, &access_rules);

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features,
            Some(instance_schema),
            vec![
                scrypto_encode(&id_type).unwrap(),
                scrypto_encode(&mutable_fields).unwrap(),
                scrypto_encode(&Decimal::zero()).unwrap(),
            ],
            btreemap!(),
        )?;

        globalize_resource_manager(
            owner_role,
            object_id,
            address_reservation,
            access_rules,
            metadata,
            api,
        )
    }

    pub(crate) fn create_with_initial_supply<Y>(
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
        access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        // TODO: Do this check in a better way (e.g. via type check)
        if id_type == NonFungibleIdType::RUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleLocalIdProvidedForRUIDType,
                ),
            ));
        }

        let mutable_fields = NonFungibleResourceManagerMutableFieldsSubstate {
            mutable_fields: non_fungible_schema.mutable_fields,
        };

        let supply: Decimal = Decimal::from(entries.len());

        let ids = entries.keys().cloned().collect();

        let mut non_fungibles = BTreeMap::new();
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

            let kv_entry = KVEntry {
                value: Some(scrypto_encode(&value).unwrap()),
                locked: false,
            };

            non_fungibles.insert(scrypto_encode(&id).unwrap(), kv_entry);
        }

        let instance_schema = InstanceSchema {
            schema: non_fungible_schema.schema,
            type_index: vec![non_fungible_schema.non_fungible],
        };

        let features = features(track_total_supply, &access_rules);

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features,
            Some(instance_schema),
            vec![
                scrypto_encode(&id_type).unwrap(),
                scrypto_encode(&mutable_fields).unwrap(),
                scrypto_encode(&supply).unwrap(),
            ],
            btreemap!(NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE => non_fungibles),
        )?;
        let (resource_address, bucket) = globalize_non_fungible_with_initial_supply(
            owner_role,
            object_id,
            address_reservation,
            access_rules,
            metadata,
            ids,
            api,
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn create_ruid_with_initial_supply<Y>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        entries: Vec<(ScryptoValue,)>,
        access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        let mut ids = BTreeSet::new();
        let mut non_fungibles = BTreeMap::new();
        let supply = Decimal::from(entries.len());
        for (entry,) in entries {
            let ruid = Runtime::generate_ruid(api)?;
            let id = NonFungibleLocalId::ruid(ruid);
            ids.insert(id.clone());
            let kv_entry = KVEntry {
                value: Some(scrypto_encode(&entry).unwrap()),
                locked: false,
            };
            non_fungibles.insert(scrypto_encode(&id).unwrap(), kv_entry);
        }

        let mutable_fields = NonFungibleResourceManagerMutableFieldsSubstate {
            mutable_fields: non_fungible_schema.mutable_fields,
        };

        let instance_schema = InstanceSchema {
            schema: non_fungible_schema.schema,
            type_index: vec![non_fungible_schema.non_fungible],
        };

        let features = features(track_total_supply, &access_rules);

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features,
            Some(instance_schema),
            vec![
                scrypto_encode(&NonFungibleIdType::RUID).unwrap(),
                scrypto_encode(&mutable_fields).unwrap(),
                scrypto_encode(&supply).unwrap(),
            ],
            btreemap!(NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE => non_fungibles),
        )?;
        let (resource_address, bucket) = globalize_non_fungible_with_initial_supply(
            owner_role,
            object_id,
            address_reservation,
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
        Self::assert_mintable(api)?;

        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());
        let id_type = {
            let handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::IdType.into(),
                LockFlags::read_only(),
            )?;
            let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;
            api.field_lock_release(handle)?;
            if id_type == NonFungibleIdType::RUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }
            id_type
        };

        // Update total supply
        // TODO: Could be further cleaned up by using event
        if api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, TRACK_TOTAL_SUPPLY_FEATURE)? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply.into(),
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

    pub(crate) fn mint_single_ruid_non_fungible<Y>(
        value: ScryptoValue,
        api: &mut Y,
    ) -> Result<(Bucket, NonFungibleLocalId), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_mintable(api)?;

        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());

        // Check id_type
        let id_type = {
            let id_type_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::IdType.into(),
                LockFlags::MUTABLE,
            )?;
            let id_type: NonFungibleIdType = api.field_lock_read_typed(id_type_handle)?;
            api.field_lock_release(id_type_handle)?;

            if id_type != NonFungibleIdType::RUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }

            id_type
        };

        // Update Total Supply
        // TODO: Could be further cleaned up by using event
        if api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, TRACK_TOTAL_SUPPLY_FEATURE)? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            total_supply += 1;
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
        }

        let id = {
            let id = NonFungibleLocalId::ruid(Runtime::generate_ruid(api)?);
            let non_fungibles = btreemap!(id.clone() => value);

            create_non_fungibles(resource_address, id_type, non_fungibles, false, api)?;

            id
        };

        let ids = btreeset!(id.clone());
        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok((bucket, id))
    }

    pub(crate) fn mint_ruid_non_fungible<Y>(
        entries: Vec<(ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_mintable(api)?;

        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());

        // Check type
        let id_type = {
            let handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::IdType.into(),
                LockFlags::MUTABLE,
            )?;
            let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;
            api.field_lock_release(handle)?;

            if id_type != NonFungibleIdType::RUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }
            id_type
        };

        // Update total supply
        // TODO: there might be better for maintaining total supply, especially for non-fungibles
        if api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, TRACK_TOTAL_SUPPLY_FEATURE)? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply.into(),
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
                let id = NonFungibleLocalId::ruid(Runtime::generate_ruid(api)?);
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
        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());
        let data_schema_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerField::MutableFields.into(),
            LockFlags::read_only(),
        )?;
        let mutable_fields: NonFungibleResourceManagerMutableFieldsSubstate =
            api.field_lock_read_typed(data_schema_handle)?;

        let mut instance_schema = api.actor_get_info()?.instance_schema.unwrap();
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

        let non_fungible_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE,
            &id.to_key(),
            LockFlags::MUTABLE,
        )?;

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

        api.key_value_entry_release(non_fungible_handle)?;

        Ok(())
    }

    pub(crate) fn non_fungible_exists<Y>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let non_fungible_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE,
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
        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());

        let non_fungible_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE,
            &id.to_key(),
            LockFlags::read_only(),
        )?;
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
        Self::burn_internal(bucket, api)
    }

    /// Only callable within this package - this is to allow the burning of tokens from a vault.
    pub(crate) fn package_burn<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::burn_internal(bucket, api)
    }

    fn burn_internal<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_burnable(api)?;

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
        if api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, TRACK_TOTAL_SUPPLY_FEATURE)? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            total_supply -= other_bucket.liquid.amount();
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
        }

        // Update
        {
            for id in other_bucket.liquid.into_ids() {
                let handle = api.actor_open_key_value_entry(
                    OBJECT_HANDLE_SELF,
                    NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE,
                    &id.to_key(),
                    LockFlags::MUTABLE,
                )?;
                api.key_value_entry_remove(handle)?;
                // Tombstone the non fungible
                // TODO: RUID non fungibles with no data don't need to go through this process
                api.key_value_entry_freeze(handle)?;
                api.key_value_entry_release(handle)?;
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

    pub(crate) fn create_empty_vault<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        //let ids = Own(api.new_index()?);
        let vault = LiquidNonFungibleVault {
            amount: Decimal::zero(),
        };
        let vault_id = api.new_simple_object(
            NON_FUNGIBLE_VAULT_BLUEPRINT,
            vec![
                scrypto_encode(&vault).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                scrypto_encode(&VaultFrozenFlag::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(api: &mut Y) -> Result<ResourceType, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerField::IdType.into(),
            LockFlags::read_only(),
        )?;

        let id_type: NonFungibleIdType = api.field_lock_read_typed(handle)?;
        let resource_type = ResourceType::NonFungible { id_type };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(api: &mut Y) -> Result<Option<Decimal>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, TRACK_TOTAL_SUPPLY_FEATURE)? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply.into(),
                LockFlags::read_only(),
            )?;
            let total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            Ok(Some(total_supply))
        } else {
            Ok(None)
        }
    }

    fn assert_mintable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, MINT_FEATURE)? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NotMintable,
                ),
            ));
        }

        return Ok(());
    }

    fn assert_burnable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, BURN_FEATURE)? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NotBurnable,
                ),
            ));
        }

        return Ok(());
    }
}
