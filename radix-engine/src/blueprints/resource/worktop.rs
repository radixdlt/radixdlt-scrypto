use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::system_callback::SystemLockData;
use crate::system::system_substates::FieldSubstate;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::resource::*;
use radix_native_sdk::resource::*;

#[derive(Debug, ScryptoSbor)]
pub struct WorktopSubstate {
    pub resources: IndexMap<ResourceAddress, Own>,
}

impl WorktopSubstate {
    pub fn new() -> Self {
        Self {
            resources: index_map_new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WorktopError {
    /// This is now unused, but kept in for backwards compatibility
    /// of error models (at least whilst we need to do that for the node,
    /// so that legacy errors can be serialized to string)
    BasicAssertionFailed,
    InsufficientBalance,
    AssertionFailed(ResourceConstraintsError),
}

pub struct WorktopBlueprint;

//==============================================
// Invariant: no empty buckets in the worktop!
//==============================================

impl WorktopBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = vec![];
        fields.push(FieldSchema::static_field(
            aggregator.add_child_type_and_descendents::<WorktopSubstate>(),
        ));

        let mut functions = index_map_new();
        functions.insert(
            WORKTOP_DROP_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopDropInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopDropOutput>(),
                ),
                export: WORKTOP_DROP_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_PUT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopPutInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopPutOutput>(),
                ),
                export: WORKTOP_PUT_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_TAKE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopTakeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopTakeOutput>(),
                ),
                export: WORKTOP_TAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopTakeNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopTakeNonFungiblesOutput>(),
                ),
                export: WORKTOP_TAKE_NON_FUNGIBLES_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_TAKE_ALL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopTakeAllInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopTakeAllOutput>(),
                ),
                export: WORKTOP_TAKE_ALL_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopAssertContainsInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopAssertContainsOutput>(),
                ),
                export: WORKTOP_ASSERT_CONTAINS_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopAssertContainsAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<WorktopAssertContainsAmountOutput>(),
                ),
                export: WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesOutput>(
                        ),
                ),
                export: WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_DRAIN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopDrainInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopDrainOutput>(),
                ),
                export: WORKTOP_DRAIN_IDENT.to_string(),
            },
        );
        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: true,
            dependencies: indexset!(),
            feature_set: indexset!(),

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state: BlueprintStateSchemaInit {
                    fields,
                    collections: vec![],
                },
                events: BlueprintEventSchemaInit::default(),
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::AllowAll,
            },
        }
    }

    pub(crate) fn drop<Y: SystemApi<RuntimeError> + KernelSubstateApi<SystemLockData>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // TODO: add `drop` callback for drop atomicity, which will remove the necessity of kernel api.

        let input: WorktopDropInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        // Detach buckets from worktop
        let handle = api.kernel_open_substate(
            input.worktop.0.as_node_id(),
            MAIN_BASE_PARTITION,
            &WorktopField::Worktop.into(),
            LockFlags::MUTABLE,
            SystemLockData::Default,
        )?;
        let mut worktop = api
            .kernel_read_substate(handle)?
            .as_typed::<FieldSubstate<WorktopSubstate>>()
            .unwrap()
            .into_payload();
        let resources = core::mem::replace(&mut worktop.resources, index_map_new());
        api.kernel_write_substate(
            handle,
            IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(worktop)),
        )?;
        api.kernel_close_substate(handle)?;

        // Recursively drop buckets
        for (_, bucket) in resources {
            let bucket = Bucket(bucket);
            bucket.drop_empty(api)?;
        }

        // Destroy self
        api.drop_object(input.worktop.0.as_node_id())?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn put<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopPutInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address = input.bucket.resource_address(api)?;
        let amount = input.bucket.amount(api)?;

        if amount.is_zero() {
            input.bucket.drop_empty(api)?;
            Ok(IndexedScryptoValue::from_typed(&()))
        } else {
            let worktop_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                WorktopField::Worktop.into(),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
            if let Some(own) = worktop.resources.get(&resource_address).cloned() {
                Bucket(own).put(input.bucket, api)?;
            } else {
                worktop.resources.insert(resource_address, input.bucket.0);
                api.field_write_typed(worktop_handle, &worktop)?;
            }
            api.field_close(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }

    pub(crate) fn take<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopTakeInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address = input.resource_address;
        let amount = input.amount;

        if amount.is_zero() {
            let bucket = ResourceManager(resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                WorktopField::Worktop.into(),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
            let existing_bucket = Bucket(worktop.resources.get(&resource_address).cloned().ok_or(
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::InsufficientBalance,
                )),
            )?);
            let existing_amount = existing_bucket.amount(api)?;

            if existing_amount < amount {
                Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))
            } else if existing_amount == amount {
                // Move
                worktop.resources.swap_remove(&resource_address);
                api.field_write_typed(worktop_handle, &worktop)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&existing_bucket))
            } else {
                let bucket = existing_bucket.take(amount, api)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&bucket))
            }
        }
    }

    pub(crate) fn take_non_fungibles<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopTakeNonFungiblesInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address = input.resource_address;
        let ids = input.ids;

        if ids.is_empty() {
            let bucket = ResourceManager(resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            let worktop_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                WorktopField::Worktop.into(),
                LockFlags::MUTABLE,
            )?;
            let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
            let existing_bucket = Bucket(worktop.resources.get(&resource_address).cloned().ok_or(
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::InsufficientBalance,
                )),
            )?);
            let existing_non_fungibles = existing_bucket.non_fungible_local_ids(api)?;

            if !existing_non_fungibles.is_superset(&ids) {
                Err(RuntimeError::ApplicationError(
                    ApplicationError::WorktopError(WorktopError::InsufficientBalance),
                ))
            } else if existing_non_fungibles.len() == ids.len() {
                // Move
                worktop = api.field_read_typed(worktop_handle)?;
                worktop.resources.swap_remove(&resource_address);
                api.field_write_typed(worktop_handle, &worktop)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&existing_bucket))
            } else {
                let bucket = existing_bucket.take_non_fungibles(ids, api)?;
                api.field_close(worktop_handle)?;
                Ok(IndexedScryptoValue::from_typed(&bucket))
            }
        }
    }

    pub(crate) fn take_all<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopTakeAllInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::MUTABLE,
        )?;
        let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        if let Some(bucket) = worktop.resources.swap_remove(&input.resource_address) {
            // Move
            api.field_write_typed(worktop_handle, &worktop)?;
            api.field_close(worktop_handle)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        } else {
            api.field_close(worktop_handle)?;
            let bucket = ResourceManager(input.resource_address).new_empty_bucket(api)?;
            Ok(IndexedScryptoValue::from_typed(&bucket))
        }
    }

    pub(crate) fn assert_contains<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopAssertContainsInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::read_only(),
        )?;
        let worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let amount = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            Bucket(bucket).amount(api)?
        } else {
            Decimal::zero()
        };
        if amount.is_zero() {
            let worktop_error =
                WorktopError::AssertionFailed(ResourceConstraintsError::ResourceConstraintFailed {
                    resource_address: input.resource_address,
                    error: ResourceConstraintError::ExpectedNonZeroAmount,
                });
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(worktop_error),
            ));
        }
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn assert_contains_amount<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopAssertContainsAmountInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::read_only(),
        )?;
        let worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let amount = if let Some(bucket) = worktop.resources.get(&input.resource_address).cloned() {
            Bucket(bucket).amount(api)?
        } else {
            Decimal::zero()
        };
        if amount < input.amount {
            let worktop_error =
                WorktopError::AssertionFailed(ResourceConstraintsError::ResourceConstraintFailed {
                    resource_address: input.resource_address,
                    error: ResourceConstraintError::ExpectedAtLeastAmount {
                        expected_at_least_amount: input.amount,
                        actual_amount: amount,
                    },
                });
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(worktop_error),
            ));
        }
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn assert_contains_non_fungibles<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopAssertContainsNonFungiblesInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::read_only(),
        )?;
        let worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let bucket_ids = if let Some(bucket) = worktop.resources.get(&input.resource_address) {
            let bucket = Bucket(bucket.clone());
            bucket.non_fungible_local_ids(api)?
        } else {
            index_set_new()
        };
        if let Some(missing_id) = input.ids.difference(&bucket_ids).next() {
            let worktop_error =
                WorktopError::AssertionFailed(ResourceConstraintsError::ResourceConstraintFailed {
                    resource_address: input.resource_address,
                    error: ResourceConstraintError::NonFungibleMissing {
                        missing_id: missing_id.clone(),
                    },
                });
            return Err(RuntimeError::ApplicationError(
                ApplicationError::WorktopError(worktop_error),
            ));
        }
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn drain<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let _input: WorktopDrainInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::MUTABLE,
        )?;
        let mut worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;
        let buckets: Vec<Own> = worktop.resources.values().cloned().collect();
        worktop.resources.clear();
        api.field_write_typed(worktop_handle, &worktop)?;
        api.field_close(worktop_handle)?;
        Ok(IndexedScryptoValue::from_typed(&buckets))
    }
}

pub struct WorktopBlueprintCuttlefishExtension;

impl WorktopBlueprintCuttlefishExtension {
    pub fn added_functions_schema() -> (
        IndexMap<String, FunctionSchemaInit>,
        VersionedSchema<ScryptoCustomSchema>,
    ) {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let mut functions = index_map_new();
        functions.insert(
            WORKTOP_ASSERT_RESOURCES_INCLUDE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<WorktopAssertResourcesIncludeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<WorktopAssertResourcesIncludeOutput>(),
                ),
                export: WORKTOP_ASSERT_RESOURCES_INCLUDE_IDENT.to_string(),
            },
        );

        functions.insert(
            WORKTOP_ASSERT_RESOURCES_ONLY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopAssertResourcesOnlyInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<WorktopAssertResourcesOnlyOutput>(),
                ),
                export: WORKTOP_ASSERT_RESOURCES_ONLY_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        (functions, schema)
    }

    pub(crate) fn assert_resources_includes<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopAssertResourcesIncludeInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        Self::aggregate_resources(api)?
            .validate_includes(input.constraints)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::AssertionFailed(e),
                ))
            })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn assert_resources_only<Y: SystemApi<RuntimeError>>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let input: WorktopAssertResourcesIncludeInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        Self::aggregate_resources(api)?
            .validate_only(input.constraints)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::WorktopError(
                    WorktopError::AssertionFailed(e),
                ))
            })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn aggregate_resources(
        api: &mut impl SystemApi<RuntimeError>,
    ) -> Result<AggregateResourceBalances, RuntimeError> {
        let worktop_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            WorktopField::Worktop.into(),
            LockFlags::read_only(),
        )?;
        let worktop: WorktopSubstate = api.field_read_typed(worktop_handle)?;

        let mut aggregated_balances = AggregateResourceBalances::new();

        for (resource, bucket) in worktop.resources {
            let bucket = Bucket(bucket.clone());
            if resource.is_fungible() {
                let amount = bucket.amount(api)?;
                aggregated_balances.add_fungible(resource, amount);
            } else {
                let ids = bucket.non_fungible_local_ids(api)?;
                aggregated_balances.add_non_fungible(resource, ids);
            }
        }

        Ok(aggregated_balances)
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            WORKTOP_ASSERT_RESOURCES_INCLUDE_IDENT => {
                WorktopBlueprintCuttlefishExtension::assert_resources_includes(input, api)
            }
            WORKTOP_ASSERT_RESOURCES_ONLY_IDENT => {
                WorktopBlueprintCuttlefishExtension::assert_resources_only(input, api)
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
