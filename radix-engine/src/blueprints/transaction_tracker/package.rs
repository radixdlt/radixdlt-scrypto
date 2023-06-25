use crate::errors::{ApplicationError, RuntimeError};
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, FieldSchema,
    FunctionSchemaInit, TypeRef,
};
use radix_engine_interface::schema::{BlueprintSchemaInit, BlueprintStateSchemaInit};

pub const TRANSACTION_TRACKER_BLUEPRINT: &str = "TransactionTracker";

pub const TRANSACTION_TRACKER_CREATE_IDENT: &str = "create";

pub const TRANSACTION_TRACKER_CREATE_EXPORT_NAME: &str = "create";

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TransactionTrackerCreateInput {
    pub address_reservation: GlobalAddressReservation,
}

#[derive(Debug, Clone, ManifestSbor)]
pub struct TransactionTrackerCreateManifestInput {
    pub address_reservation: ManifestAddressReservation,
}

pub type TransactionTrackerCreateOutput = ComponentAddress;

pub struct TransactionTrackerNativePackage;

pub const PARTITION_RANGE_START: u8 = MAIN_BASE_PARTITION.0 + 1;
pub const PARTITION_RANGE_END: u8 = u8::MAX;
pub const EPOCHS_PER_PARTITION: u64 = 100;

impl TransactionTrackerNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<Hash>();
        let value_type_index = aggregator.add_child_type_and_descendents::<TransactionStatus>();

        let mut collections: Vec<BlueprintCollectionSchema<TypeRef<LocalTypeIndex>>> = vec![];
        for _ in PARTITION_RANGE_START..=PARTITION_RANGE_END {
            collections.push(BlueprintCollectionSchema::KeyValueStore(
                BlueprintKeyValueStoreSchema {
                    key: TypeRef::Static(key_type_index),
                    value: TypeRef::Static(value_type_index),
                    can_own: false,
                },
            ))
        }

        let mut fields = Vec::new();
        fields.push(FieldSchema::static_field(
            aggregator.add_child_type_and_descendents::<TransactionTrackerSubstate>(),
        ));

        let mut functions = BTreeMap::new();
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
        let blueprints = btreemap!(
            TRANSACTION_TRACKER_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                dependencies: btreeset!(
                ),
                feature_set: btreeset!(),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        virtual_lazy_load_functions: btreemap!(),
                        functions,
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AccessRules(
                        btreemap!(
                            TRANSACTION_TRACKER_CREATE_IDENT.to_string() => rule!(require(AuthAddresses::system_role())),
                        )
                    ),
                    method_auth: MethodAuthTemplate::default(),
                },
            }
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
    CommittedSuccess,
    CommittedFailure,
    Cancelled,
}

pub type TransactionStatusSubstateContents = TransactionStatus;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TransactionTrackerSubstate {
    pub start_epoch: u64,
    pub start_partition: u8,

    // parameters
    pub partition_range_start_inclusive: u8,
    pub partition_range_end_inclusive: u8,
    pub epochs_per_partition: u64,
}

impl TransactionTrackerSubstate {
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
    pub fn create<Y>(
        address_reservation: GlobalAddressReservation,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let current_epoch = Runtime::current_epoch(api)?;
        let intent_store = api.new_simple_object(
            TRANSACTION_TRACKER_BLUEPRINT,
            vec![scrypto_encode(&TransactionTrackerSubstate {
                start_epoch: current_epoch.number(),
                start_partition: PARTITION_RANGE_START,
                partition_range_start_inclusive: PARTITION_RANGE_START,
                partition_range_end_inclusive: PARTITION_RANGE_END,
                epochs_per_partition: EPOCHS_PER_PARTITION,
            })
            .unwrap()],
        )?;
        let access_rules = AccessRules::create(OwnerRole::None, btreemap!(), api)?.0;
        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;

        let address = api.globalize(
            btreemap!(
                ObjectModuleId::Main => intent_store,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            Some(address_reservation),
        )?;
        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_coverage() {
        let covered_epochs = (EPOCHS_PER_PARTITION as f64
            * (PARTITION_RANGE_END as f64 - (PARTITION_RANGE_START as f64 - 1.0) - 1.0))
            .floor() as u64;
        let covered_days = covered_epochs
            * 5 // Targeted epoch duration: 5 mins
            / 60
            / 24;
        assert!(covered_epochs >= DEFAULT_MAX_EPOCH_RANGE);
        assert_eq!(covered_days, 65);
    }

    #[test]
    fn test_partition_calculation() {
        let mut store = TransactionTrackerSubstate {
            start_epoch: 256,
            start_partition: 70,
            partition_range_start_inclusive: PARTITION_RANGE_START,
            partition_range_end_inclusive: PARTITION_RANGE_END,
            epochs_per_partition: EPOCHS_PER_PARTITION,
        };
        let num_partitions = (PARTITION_RANGE_END - PARTITION_RANGE_START + 1) as u64;

        assert_eq!(store.partition_for_expiry_epoch(Epoch::of(0)), None);
        assert_eq!(store.partition_for_expiry_epoch(Epoch::of(256)), Some(70));
        assert_eq!(
            store.partition_for_expiry_epoch(Epoch::of(256 + EPOCHS_PER_PARTITION - 1)),
            Some(70)
        );
        assert_eq!(
            store.partition_for_expiry_epoch(Epoch::of(256 + EPOCHS_PER_PARTITION)),
            Some(71)
        );
        assert_eq!(
            store.partition_for_expiry_epoch(Epoch::of(
                256 + EPOCHS_PER_PARTITION * num_partitions - 1
            )),
            Some(69)
        );
        assert_eq!(
            store
                .partition_for_expiry_epoch(Epoch::of(256 + EPOCHS_PER_PARTITION * num_partitions)),
            None,
        );

        store.advance();
        assert_eq!(store.start_epoch, 256 + EPOCHS_PER_PARTITION);
        assert_eq!(store.start_partition, 71);
    }
}
