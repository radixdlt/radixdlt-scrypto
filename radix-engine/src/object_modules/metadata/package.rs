use crate::internal_prelude::*;
use crate::{errors::*, event_schema, roles_template};
use radix_blueprint_schema_init::{
    BlueprintFunctionsSchemaInit, BlueprintSchemaInit, FunctionSchemaInit, TypeRef,
};
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{FieldValue, GenericArgs, KVEntry, SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::object_modules::metadata::*;
use radix_native_sdk::runtime::Runtime;

use super::{RemoveMetadataEvent, SetMetadataEvent};

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum MetadataError {
    KeyStringExceedsMaxLength { max: usize, actual: usize },
    ValueSborExceedsMaxLength { max: usize, actual: usize },
    ValueDecodeError(DecodeError),
    MetadataValidationError(MetadataValidationError),
}

declare_native_blueprint_state! {
    blueprint_ident: Metadata,
    blueprint_snake_case: metadata,
    features: {
    },
    fields: {
    },
    collections: {
        entries: KeyValue {
            entry_ident: Entry,
            key_type: {
                kind: Static,
                content_type: String,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
    }
}

pub type MetadataEntryV1 = MetadataValue;

pub struct MetadataNativePackage;

impl MetadataNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = MetadataStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            METADATA_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataCreateOutput>(),
                ),
                export: METADATA_CREATE_IDENT.to_string(),
            },
        );
        functions.insert(
            METADATA_CREATE_WITH_DATA_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataCreateWithDataInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataCreateWithDataOutput>(),
                ),
                export: METADATA_CREATE_WITH_DATA_IDENT.to_string(),
            },
        );
        functions.insert(
            METADATA_SET_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataSetInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataSetOutput>(),
                ),
                export: METADATA_SET_IDENT.to_string(),
            },
        );
        functions.insert(
            METADATA_LOCK_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataLockInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataLockOutput>(),
                ),
                export: METADATA_LOCK_IDENT.to_string(),
            },
        );
        functions.insert(
            METADATA_GET_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataGetInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataGetOutput>(),
                ),
                export: METADATA_GET_IDENT.to_string(),
            },
        );
        functions.insert(
            METADATA_REMOVE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataRemoveInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<MetadataRemoveOutput>(),
                ),
                export: METADATA_REMOVE_IDENT.to_string(),
            },
        );

        let events = event_schema! {
            aggregator,
            [SetMetadataEvent, RemoveMetadataEvent]
        };

        let schema = generate_full_schema(aggregator);
        let blueprints = indexmap!(
            METADATA_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: true,
                feature_set: indexset!(),
                dependencies: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state,
                    events,
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                    },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoleDefinition(
                        roles_template!(
                            roles {
                                METADATA_SETTER_ROLE => updaters: [METADATA_SETTER_UPDATER_ROLE];
                                METADATA_SETTER_UPDATER_ROLE => updaters: [METADATA_SETTER_UPDATER_ROLE];
                                METADATA_LOCKER_ROLE => updaters: [METADATA_LOCKER_UPDATER_ROLE];
                                METADATA_LOCKER_UPDATER_ROLE => updaters: [METADATA_LOCKER_UPDATER_ROLE];
                            },
                            methods {
                                METADATA_SET_IDENT => [METADATA_SETTER_ROLE];
                                METADATA_REMOVE_IDENT => [METADATA_SETTER_ROLE];
                                METADATA_LOCK_IDENT => [METADATA_LOCKER_ROLE];
                                METADATA_GET_IDENT => MethodAccessibility::Public;
                            }
                        ),
                    ),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedOwnedScryptoValue, RuntimeError> {
        match export_name {
            METADATA_CREATE_IDENT => {
                let _input: MetadataCreateInput = input.into_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::create(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            METADATA_CREATE_WITH_DATA_IDENT => {
                let input: MetadataCreateWithDataInput = input.into_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::create_with_data(input.data, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            METADATA_SET_IDENT => {
                let input: MetadataSetInput = input.into_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::set(input.key, input.value, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            METADATA_LOCK_IDENT => {
                let input: MetadataLockInput = input.into_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::lock(input.key, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            METADATA_GET_IDENT => {
                let input: MetadataGetInput = input.into_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::get(input.key, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            METADATA_REMOVE_IDENT => {
                let input: MetadataRemoveInput = input.into_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::remove(input.key, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn create<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Own, RuntimeError> {
        let node_id = api.new_object(
            METADATA_BLUEPRINT,
            vec![],
            GenericArgs::default(),
            indexmap!(),
            indexmap!(),
        )?;

        Ok(Own(node_id))
    }

    pub fn init_system_struct(
        data: MetadataInit,
    ) -> Result<
        (
            IndexMap<u8, FieldValue>,
            IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
        ),
        MetadataError,
    > {
        let mut init_kv_entries = index_map_new();
        for (key, entry) in data.data {
            if key.len() > MAX_METADATA_KEY_STRING_LEN {
                return Err(MetadataError::KeyStringExceedsMaxLength {
                    max: MAX_METADATA_KEY_STRING_LEN,
                    actual: key.len(),
                });
            }

            let key = scrypto_encode(&key).unwrap();

            let value = match entry.value {
                Some(metadata_value) => {
                    let value = scrypto_encode(&MetadataEntryEntryPayload::from_content_source(
                        metadata_value,
                    ))
                    .unwrap();
                    if value.len() > MAX_METADATA_VALUE_SBOR_LEN {
                        return Err(MetadataError::ValueSborExceedsMaxLength {
                            max: MAX_METADATA_VALUE_SBOR_LEN,
                            actual: value.len(),
                        });
                    }
                    Some(value)
                }
                None => None,
            };

            let kv_entry = KVEntry {
                value,
                locked: entry.lock,
            };

            init_kv_entries.insert(key, kv_entry);
        }

        Ok((
            indexmap!(),
            indexmap!(MetadataCollection::EntryKeyValue.collection_index() => init_kv_entries),
        ))
    }

    pub(crate) fn create_with_data<Y: SystemApi<RuntimeError>>(
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<Own, RuntimeError> {
        for value in metadata_init.data.values() {
            if let Some(v) = &value.value {
                validate_metadata_value(&v).map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::MetadataError(
                        MetadataError::MetadataValidationError(e),
                    ))
                })?;
            }
        }

        let (fields, kv_entries) = Self::init_system_struct(metadata_init)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::MetadataError(e)))?;

        let node_id = api.new_object(
            METADATA_BLUEPRINT,
            vec![],
            GenericArgs::default(),
            fields,
            kv_entries,
        )?;

        Ok(Own(node_id))
    }

    pub(crate) fn set<Y: SystemApi<RuntimeError>>(
        key: String,
        value: MetadataValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        validate_metadata_value(&value).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataValidationError(e),
            ))
        })?;

        if key.len() > MAX_METADATA_KEY_STRING_LEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::MetadataError(MetadataError::KeyStringExceedsMaxLength {
                    max: MAX_METADATA_KEY_STRING_LEN,
                    actual: key.len(),
                }),
            ));
        }

        let sbor_value = scrypto_encode_to_value(&MetadataEntryEntryPayload::from_content_source(
            value.clone(),
        ))
        .unwrap();
        if sbor_value.payload_len() > MAX_METADATA_VALUE_SBOR_LEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::MetadataError(MetadataError::ValueSborExceedsMaxLength {
                    max: MAX_METADATA_VALUE_SBOR_LEN,
                    actual: sbor_value.payload_len(),
                }),
            ));
        }

        let handle = api.actor_open_key_value_entry_typed(
            ACTOR_STATE_SELF,
            MetadataCollection::EntryKeyValue.collection_index(),
            &key,
            LockFlags::MUTABLE,
        )?;
        api.key_value_entry_set(handle, sbor_value.into_unvalidated())?;
        api.key_value_entry_close(handle)?;

        Runtime::emit_event(api, SetMetadataEvent { key, value })?;

        Ok(())
    }

    pub(crate) fn lock<Y: SystemApi<RuntimeError>>(
        key: String,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_key_value_entry_typed(
            ACTOR_STATE_SELF,
            MetadataCollection::EntryKeyValue.collection_index(),
            &key,
            LockFlags::MUTABLE,
        )?;
        api.key_value_entry_lock(handle)?;
        api.key_value_entry_close(handle)?;

        Ok(())
    }

    pub(crate) fn get<Y: SystemApi<RuntimeError>>(
        key: String,
        api: &mut Y,
    ) -> Result<Option<MetadataValue>, RuntimeError> {
        let handle = api.actor_open_key_value_entry_typed(
            ACTOR_STATE_SELF,
            MetadataCollection::EntryKeyValue.collection_index(),
            &key,
            LockFlags::read_only(),
        )?;

        let substate: Option<MetadataEntryEntryPayload> = api.key_value_entry_get_typed(handle)?;

        Ok(substate.map(|v: MetadataEntryEntryPayload| v.fully_update_and_into_latest_version()))
    }

    pub(crate) fn remove<Y: SystemApi<RuntimeError>>(
        key: String,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        let cur_value: Option<MetadataEntryEntryPayload> =
            api.actor_remove_key_value_entry_typed(ACTOR_STATE_SELF, 0u8, &key)?;
        let rtn = cur_value.is_some();

        Runtime::emit_event(api, RemoveMetadataEvent { key })?;

        Ok(rtn)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MetadataValidationError {
    InvalidURL(String),
    InvalidOrigin(String),
}

pub fn validate_metadata_value(value: &MetadataValue) -> Result<(), MetadataValidationError> {
    match value {
        MetadataValue::String(_) => {}
        MetadataValue::Bool(_) => {}
        MetadataValue::U8(_) => {}
        MetadataValue::U32(_) => {}
        MetadataValue::U64(_) => {}
        MetadataValue::I32(_) => {}
        MetadataValue::I64(_) => {}
        MetadataValue::Decimal(_) => {}
        MetadataValue::GlobalAddress(_) => {}
        MetadataValue::PublicKey(_) => {}
        MetadataValue::NonFungibleGlobalId(_) => {}
        MetadataValue::NonFungibleLocalId(_) => {}
        MetadataValue::Instant(_) => {}
        MetadataValue::Url(url) => {
            CheckedUrl::of(url.as_str())
                .ok_or(MetadataValidationError::InvalidURL(url.as_str().to_owned()))?;
        }
        MetadataValue::Origin(origin) => {
            CheckedOrigin::of(origin.as_str()).ok_or(MetadataValidationError::InvalidOrigin(
                origin.as_str().to_owned(),
            ))?;
        }
        MetadataValue::PublicKeyHash(_) => {}
        MetadataValue::StringArray(_) => {}
        MetadataValue::BoolArray(_) => {}
        MetadataValue::U8Array(_) => {}
        MetadataValue::U32Array(_) => {}
        MetadataValue::U64Array(_) => {}
        MetadataValue::I32Array(_) => {}
        MetadataValue::I64Array(_) => {}
        MetadataValue::DecimalArray(_) => {}
        MetadataValue::GlobalAddressArray(_) => {}
        MetadataValue::PublicKeyArray(_) => {}
        MetadataValue::NonFungibleGlobalIdArray(_) => {}
        MetadataValue::NonFungibleLocalIdArray(_) => {}
        MetadataValue::InstantArray(_) => {}
        MetadataValue::UrlArray(urls) => {
            for url in urls {
                CheckedUrl::of(url.as_str())
                    .ok_or(MetadataValidationError::InvalidURL(url.as_str().to_owned()))?;
            }
        }
        MetadataValue::OriginArray(origins) => {
            for origin in origins {
                CheckedOrigin::of(origin.as_str()).ok_or(
                    MetadataValidationError::InvalidOrigin(origin.as_str().to_owned()),
                )?;
            }
        }
        MetadataValue::PublicKeyHashArray(_) => {}
    }

    Ok(())
}
