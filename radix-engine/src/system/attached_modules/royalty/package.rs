use crate::errors::*;
use crate::system::system_modules::costing::{apply_royalty_cost, RoyaltyRecipient};
use crate::types::*;
use native_sdk::resource::NativeVault;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::{ClientApi, FieldValue, GenericArgs, KVEntry, ACTOR_STATE_SELF};
use radix_engine_interface::schema::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, BlueprintSchemaInit,
    FunctionSchemaInit, TypeRef,
};

// Re-export substates
use crate::blueprints::package::PackageError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelApi;
use crate::roles_template;
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_substates::FieldSubstate;
use crate::system::system_substates::KeyValueEntrySubstate;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};

declare_native_blueprint_state! {
    blueprint_ident: ComponentRoyalty,
    blueprint_snake_case: component_royalty,
    features: {
    },
    fields: {
        accumulator: {
            ident: Accumulator,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
    },
    collections: {
        method_royalties: KeyValue {
            entry_ident: MethodAmount,
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

pub type ComponentRoyaltyAccumulatorV1 = ComponentRoyaltySubstate;
pub type ComponentRoyaltyMethodAmountV1 = RoyaltyAmount;

pub struct RoyaltyNativePackage;
impl RoyaltyNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = ComponentRoyaltyStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            COMPONENT_ROYALTY_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentRoyaltyCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentRoyaltyCreateOutput>(),
                ),
                export: COMPONENT_ROYALTY_CREATE_IDENT.to_string(),
            },
        );
        functions.insert(
            COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentRoyaltySetInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentRoyaltySetOutput>(),
                ),
                export: COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
            },
        );
        functions.insert(
            COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentRoyaltyLockInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentRoyaltyLockOutput>(),
                ),
                export: COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT.to_string(),
            },
        );
        functions.insert(
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentClaimRoyaltiesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ComponentClaimRoyaltiesOutput>(),
                ),
                export: COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);

        let blueprints = indexmap!(
            COMPONENT_ROYALTY_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: true,
                feature_set: indexset!(),
                dependencies: indexset!(XRD.into(),),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state,
                    events: BlueprintEventSchemaInit::default(),
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
                                COMPONENT_ROYALTY_SETTER_ROLE => updaters: [COMPONENT_ROYALTY_SETTER_UPDATER_ROLE];
                                COMPONENT_ROYALTY_SETTER_UPDATER_ROLE => updaters: [COMPONENT_ROYALTY_SETTER_UPDATER_ROLE];
                                COMPONENT_ROYALTY_LOCKER_ROLE => updaters: [COMPONENT_ROYALTY_LOCKER_UPDATER_ROLE];
                                COMPONENT_ROYALTY_LOCKER_UPDATER_ROLE => updaters: [COMPONENT_ROYALTY_LOCKER_UPDATER_ROLE];
                                COMPONENT_ROYALTY_CLAIMER_ROLE => updaters: [COMPONENT_ROYALTY_CLAIMER_UPDATER_ROLE];
                                COMPONENT_ROYALTY_CLAIMER_UPDATER_ROLE => updaters: [COMPONENT_ROYALTY_CLAIMER_UPDATER_ROLE];
                            },
                            methods {
                                COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT => [COMPONENT_ROYALTY_CLAIMER_ROLE];
                                COMPONENT_ROYALTY_SET_ROYALTY_IDENT => [COMPONENT_ROYALTY_SETTER_ROLE];
                                COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT => [COMPONENT_ROYALTY_LOCKER_ROLE];
                            }
                        ),
                    ),
                },
            },
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            COMPONENT_ROYALTY_CREATE_IDENT => {
                let input: ComponentRoyaltyCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ComponentRoyaltyBlueprint::create(input.royalty_config, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            COMPONENT_ROYALTY_SET_ROYALTY_IDENT => {
                let input: ComponentRoyaltySetInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ComponentRoyaltyBlueprint::set_royalty(input.method, input.amount, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT => {
                let input: ComponentRoyaltyLockInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ComponentRoyaltyBlueprint::lock_royalty(input.method, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT => {
                let _input: ComponentClaimRoyaltiesInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ComponentRoyaltyBlueprint::claim_royalties(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum ComponentRoyaltyError {
    RoyaltyAmountIsGreaterThanAllowed {
        max: RoyaltyAmount,
        actual: RoyaltyAmount,
    },
    UnexpectedDecimalComputationError,
    RoyaltyAmountIsNegative(RoyaltyAmount),
}

pub struct RoyaltyUtil;

impl RoyaltyUtil {
    pub fn verify_royalty_amounts<'a, I: Iterator<Item = &'a RoyaltyAmount>, Y>(
        royalty_amounts: I,
        is_component: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let max_royalty_in_xrd = match api.max_per_function_royalty_in_xrd() {
            Ok(amount) => Ok(amount),
            Err(RuntimeError::SystemError(SystemError::CostingModuleNotEnabled)) => return Ok(()),
            e => e,
        }?;
        let max_royalty_in_usd = max_royalty_in_xrd.checked_div(api.usd_price()?).ok_or(
            RuntimeError::ApplicationError(ApplicationError::ComponentRoyaltyError(
                ComponentRoyaltyError::UnexpectedDecimalComputationError,
            )),
        )?;

        for royalty_amount in royalty_amounts {
            // Disallow negative royalties, 0 is acceptable.
            if royalty_amount.is_negative() {
                if is_component {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::ComponentRoyaltyError(
                            ComponentRoyaltyError::RoyaltyAmountIsNegative(*royalty_amount),
                        ),
                    ));
                } else {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::PackageError(PackageError::RoyaltyAmountIsNegative(
                            *royalty_amount,
                        )),
                    ));
                }
            }

            match royalty_amount {
                RoyaltyAmount::Free => {}
                RoyaltyAmount::Xrd(xrd_amount) => {
                    if xrd_amount.gt(&max_royalty_in_xrd) {
                        if is_component {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::ComponentRoyaltyError(
                                    ComponentRoyaltyError::RoyaltyAmountIsGreaterThanAllowed {
                                        max: RoyaltyAmount::Xrd(max_royalty_in_xrd),
                                        actual: royalty_amount.clone(),
                                    },
                                ),
                            ));
                        } else {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(
                                    PackageError::RoyaltyAmountIsGreaterThanAllowed {
                                        max: RoyaltyAmount::Xrd(max_royalty_in_xrd),
                                        actual: royalty_amount.clone(),
                                    },
                                ),
                            ));
                        }
                    }
                }
                RoyaltyAmount::Usd(usd_amount) => {
                    if usd_amount.gt(&max_royalty_in_usd) {
                        if is_component {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::ComponentRoyaltyError(
                                    ComponentRoyaltyError::RoyaltyAmountIsGreaterThanAllowed {
                                        max: RoyaltyAmount::Usd(max_royalty_in_usd),
                                        actual: royalty_amount.clone(),
                                    },
                                ),
                            ));
                        } else {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(
                                    PackageError::RoyaltyAmountIsGreaterThanAllowed {
                                        max: RoyaltyAmount::Xrd(max_royalty_in_xrd),
                                        actual: royalty_amount.clone(),
                                    },
                                ),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct ComponentRoyaltyBlueprint;

impl ComponentRoyaltyBlueprint {
    pub(crate) fn create<Y>(
        royalty_config: ComponentRoyaltyConfig,
        api: &mut Y,
    ) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Create a royalty vault
        let accumulator_substate = ComponentRoyaltySubstate {
            royalty_vault: Vault::create(XRD, api)?,
        };

        let mut kv_entries = index_map_new();
        {
            RoyaltyUtil::verify_royalty_amounts(
                royalty_config
                    .royalty_amounts
                    .values()
                    .map(|(amount, _locked)| amount),
                true,
                api,
            )?;

            let mut royalty_config_entries = index_map_new();
            for (method, (amount, locked)) in royalty_config.royalty_amounts {
                let kv_entry = KVEntry {
                    value: Some(
                        scrypto_encode(
                            &ComponentRoyaltyMethodAmountEntryPayload::from_content_source(amount),
                        )
                        .unwrap(),
                    ),
                    locked,
                };
                royalty_config_entries.insert(scrypto_encode(&method).unwrap(), kv_entry);
            }
            kv_entries.insert(
                ComponentRoyaltyCollection::MethodAmountKeyValue.collection_index(),
                royalty_config_entries,
            );
        }

        let component_id = api.new_object(
            COMPONENT_ROYALTY_BLUEPRINT,
            vec![],
            GenericArgs::default(),
            indexmap! {
                ComponentRoyaltyField::Accumulator.field_index() => FieldValue::immutable(&ComponentRoyaltyAccumulatorFieldPayload::from_content_source(accumulator_substate)),
            },
            kv_entries,
        )?;

        Ok(Own(component_id))
    }

    pub(crate) fn set_royalty<Y>(
        method: String,
        amount: RoyaltyAmount,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        RoyaltyUtil::verify_royalty_amounts(vec![amount.clone()].iter(), true, api)?;

        let handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            ComponentRoyaltyCollection::MethodAmountKeyValue.collection_index(),
            &scrypto_encode(&method).unwrap(),
            LockFlags::MUTABLE,
        )?;
        api.key_value_entry_set_typed(
            handle,
            ComponentRoyaltyMethodAmountEntryPayload::from_content_source(amount),
        )?;
        api.key_value_entry_close(handle)?;

        Ok(())
    }

    pub(crate) fn lock_royalty<Y>(method: String, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            ComponentRoyaltyCollection::MethodAmountKeyValue.collection_index(),
            &scrypto_encode(&method).unwrap(),
            LockFlags::MUTABLE,
        )?;
        api.key_value_entry_lock(handle)?;
        api.key_value_entry_close(handle)?;

        Ok(())
    }

    pub(crate) fn claim_royalties<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            RoyaltyField::RoyaltyAccumulator.into(),
            LockFlags::read_only(),
        )?;

        let substate = api
            .field_read_typed::<ComponentRoyaltyAccumulatorFieldPayload>(handle)?
            .into_latest();
        let mut royalty_vault = substate.royalty_vault;
        let bucket = royalty_vault.take_all(api)?;
        api.field_close(handle)?;

        Ok(bucket)
    }

    pub fn charge_component_royalty<Y, V>(
        receiver: &NodeId,
        ident: &str,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        let accumulator_handle = api.kernel_open_substate(
            receiver,
            ROYALTY_BASE_PARTITION
                .at_offset(ROYALTY_FIELDS_PARTITION_OFFSET)
                .unwrap(),
            &RoyaltyField::RoyaltyAccumulator.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;
        let component_royalty: FieldSubstate<ComponentRoyaltyAccumulatorFieldPayload> = api
            .kernel_read_substate(accumulator_handle)?
            .as_typed()
            .unwrap();

        let component_royalty = component_royalty.into_payload().into_latest();

        let royalty_charge = {
            let handle = api.kernel_open_substate_with_default(
                receiver,
                ROYALTY_BASE_PARTITION
                    .at_offset(ROYALTY_CONFIG_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(ident).unwrap()),
                LockFlags::read_only(),
                Some(|| {
                    let kv_entry =
                        KeyValueEntrySubstate::<ComponentRoyaltyMethodAmountEntryPayload>::default(
                        );
                    IndexedScryptoValue::from_typed(&kv_entry)
                }),
                SystemLockData::default(),
            )?;

            let substate: KeyValueEntrySubstate<ComponentRoyaltyMethodAmountEntryPayload> =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            api.kernel_close_substate(handle)?;
            substate
                .into_value()
                .map(|v| v.into_latest())
                .unwrap_or(RoyaltyAmount::Free)
        };

        // We check for negative royalties at the instantiation time of the royalty module,
        // and whenever the royalty amount is updated
        assert!(!royalty_charge.is_negative());

        if royalty_charge.is_non_zero() {
            let vault_id = component_royalty.royalty_vault.0;
            let component_address = ComponentAddress::new_or_panic(receiver.0);

            apply_royalty_cost(
                api,
                royalty_charge,
                RoyaltyRecipient::Component(component_address, vault_id.into()),
            )?;
        }

        api.kernel_close_substate(accumulator_handle)?;

        Ok(())
    }
}
