use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use radix_blueprint_schema_init::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, FieldSchema,
    FunctionSchemaInit, TypeRef,
};
use radix_blueprint_schema_init::{BlueprintSchemaInit, BlueprintStateSchemaInit};
use radix_engine_interface::api::{AttachedModuleId, FieldValue, SystemApi};
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::blueprints::transaction_tracker::*;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_native_sdk::runtime::Runtime;

pub use radix_common::prelude::TRANSACTION_TRACKER_BLUEPRINT;

pub type TransactionTrackerCreateOutput = ComponentAddress;

pub struct TransactionTrackerNativePackage;

pub const PARTITION_RANGE_START: u8 = MAIN_BASE_PARTITION.0 + 1;
pub const PARTITION_RANGE_END: u8 = u8::MAX;
pub const EPOCHS_PER_PARTITION: u64 = 100;

impl TransactionTrackerNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_id = aggregator.add_child_type_and_descendents::<Hash>();
        let value_type_id = aggregator.add_child_type_and_descendents::<TransactionStatus>();

        let mut collections: Vec<BlueprintCollectionSchema<TypeRef<LocalTypeId>>> = vec![];
        for _ in PARTITION_RANGE_START..=PARTITION_RANGE_END {
            collections.push(BlueprintCollectionSchema::KeyValueStore(
                BlueprintKeyValueSchema {
                    key: TypeRef::Static(key_type_id),
                    value: TypeRef::Static(value_type_id),
                    allow_ownership: false,
                },
            ))
        }

        let mut fields = Vec::new();
        fields.push(FieldSchema::static_field(
            aggregator.add_child_type_and_descendents::<TransactionTrackerSubstate>(),
        ));

        let mut functions = index_map_new();
        functions.insert(
            TRANSACTION_TRACKER_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TransactionTrackerCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TransactionTrackerCreateOutput>(),
                ),
                export: TRANSACTION_TRACKER_CREATE_EXPORT_NAME.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = indexmap!(
            TRANSACTION_TRACKER_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: false,
                dependencies: indexset!(
                ),
                feature_set: indexset!(),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                    },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AccessRules(
                        indexmap!(
                            TRANSACTION_TRACKER_CREATE_IDENT.to_string() => rule!(require(system_execution(SystemExecution::Protocol))),
                        )
                    ),
                    method_auth: MethodAuthTemplate::default(),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            TRANSACTION_TRACKER_CREATE_EXPORT_NAME => {
                let input: TransactionTrackerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = TransactionTrackerBlueprint::create(input.address_reservation, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub enum TransactionStatus {
    V1(TransactionStatusV1),
}

impl TransactionStatus {
    pub fn into_v1(self) -> TransactionStatusV1 {
        match self {
            TransactionStatus::V1(status) => status,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub enum TransactionStatusV1 {
    CommittedSuccess,
    CommittedFailure,
    Cancelled,
}

pub type TransactionStatusSubstateContents = TransactionStatus;

#[derive(Debug, Clone, ScryptoSbor)]
pub enum TransactionTrackerSubstate {
    V1(TransactionTrackerSubstateV1),
}

impl TransactionTrackerSubstate {
    pub fn v1(&self) -> &TransactionTrackerSubstateV1 {
        match self {
            TransactionTrackerSubstate::V1(tracker) => tracker,
        }
    }

    pub fn into_v1(self) -> TransactionTrackerSubstateV1 {
        match self {
            TransactionTrackerSubstate::V1(tracker) => tracker,
        }
    }

    pub fn v1_mut(&mut self) -> &mut TransactionTrackerSubstateV1 {
        match self {
            TransactionTrackerSubstate::V1(tracker) => tracker,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TransactionTrackerSubstateV1 {
    pub start_epoch: u64,
    pub start_partition: u8,

    // parameters
    pub partition_range_start_inclusive: u8,
    pub partition_range_end_inclusive: u8,
    pub epochs_per_partition: u64,
}

impl TransactionTrackerSubstateV1 {
    pub fn partition_for_expiry_epoch(&self, epoch: Epoch) -> Option<u8> {
        let epoch = epoch.number();

        // Check if epoch is within range
        let num_partitions =
            self.partition_range_end_inclusive - self.partition_range_start_inclusive + 1;
        let max_epoch_exclusive =
            self.start_epoch + num_partitions as u64 * self.epochs_per_partition;
        if epoch < self.start_epoch || epoch >= max_epoch_exclusive {
            return None;
        }

        // Calculate the destination partition number
        let mut partition_number =
            self.start_partition as u64 + (epoch - self.start_epoch) / self.epochs_per_partition;
        if partition_number > self.partition_range_end_inclusive as u64 {
            partition_number -= num_partitions as u64;
        }

        assert!(partition_number >= self.partition_range_start_inclusive as u64);
        assert!(partition_number <= self.partition_range_end_inclusive as u64);

        Some(partition_number as u8)
    }

    /// This method will shift the start partition by 1, considering the partition range as a buffer.
    /// Protocol-specific implementation is within transaction executor.
    pub fn advance(&mut self) -> u8 {
        let old_start_partition = self.start_partition;
        self.start_epoch += self.epochs_per_partition;
        self.start_partition = if self.start_partition == self.partition_range_end_inclusive {
            self.partition_range_start_inclusive
        } else {
            self.start_partition + 1
        };
        old_start_partition
    }
}

pub struct TransactionTrackerBlueprint;

impl TransactionTrackerBlueprint {
    pub fn create<Y: SystemApi<RuntimeError>>(
        address_reservation: GlobalAddressReservation,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError> {
        let current_epoch = Runtime::current_epoch(api)?;
        let intent_store = api.new_simple_object(
            TRANSACTION_TRACKER_BLUEPRINT,
            indexmap!(
                0u8 => FieldValue::new(&TransactionTrackerSubstate::V1(TransactionTrackerSubstateV1{
                    start_epoch: current_epoch.number(),
                    start_partition: PARTITION_RANGE_START,
                    partition_range_start_inclusive: PARTITION_RANGE_START,
                    partition_range_end_inclusive: PARTITION_RANGE_END,
                    epochs_per_partition: EPOCHS_PER_PARTITION,
                }))
            ),
        )?;
        let role_assignment = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?.0;
        let metadata = Metadata::create(api)?;

        let address = api.globalize(
            intent_store,
            indexmap!(
                AttachedModuleId::RoleAssignment => role_assignment.0,
                AttachedModuleId::Metadata => metadata.0,
            ),
            Some(address_reservation),
        )?;
        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use radix_transactions::validation::TransactionValidationConfig;

    use super::*;

    #[test]
    fn calculate_coverage() {
        let max_epoch_range = TransactionValidationConfig::latest().max_epoch_range;
        let covered_epochs = (EPOCHS_PER_PARTITION as f64
            * (PARTITION_RANGE_END as f64 - (PARTITION_RANGE_START as f64 - 1.0) - 1.0))
            .floor() as u64;
        let covered_days = covered_epochs
            * 5 // Targeted epoch duration: 5 mins
            / 60
            / 24;
        assert!(covered_epochs >= max_epoch_range);
        assert_eq!(covered_days, 65);
    }

    #[test]
    fn test_partition_calculation() {
        let mut store = TransactionTrackerSubstate::V1(TransactionTrackerSubstateV1 {
            start_epoch: 256,
            start_partition: 70,
            partition_range_start_inclusive: PARTITION_RANGE_START,
            partition_range_end_inclusive: PARTITION_RANGE_END,
            epochs_per_partition: EPOCHS_PER_PARTITION,
        });
        let num_partitions = (PARTITION_RANGE_END - PARTITION_RANGE_START + 1) as u64;

        assert_eq!(store.v1().partition_for_expiry_epoch(Epoch::of(0)), None);
        assert_eq!(
            store.v1().partition_for_expiry_epoch(Epoch::of(256)),
            Some(70)
        );
        assert_eq!(
            store
                .v1()
                .partition_for_expiry_epoch(Epoch::of(256 + EPOCHS_PER_PARTITION - 1)),
            Some(70)
        );
        assert_eq!(
            store
                .v1()
                .partition_for_expiry_epoch(Epoch::of(256 + EPOCHS_PER_PARTITION)),
            Some(71)
        );
        assert_eq!(
            store.v1().partition_for_expiry_epoch(Epoch::of(
                256 + EPOCHS_PER_PARTITION * num_partitions - 1
            )),
            Some(69)
        );
        assert_eq!(
            store
                .v1()
                .partition_for_expiry_epoch(Epoch::of(256 + EPOCHS_PER_PARTITION * num_partitions)),
            None,
        );

        store.v1_mut().advance();
        assert_eq!(store.v1().start_epoch, 256 + EPOCHS_PER_PARTITION);
        assert_eq!(store.v1().start_partition, 71);
    }
}
