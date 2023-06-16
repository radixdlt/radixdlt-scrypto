use crate::errors::{ApplicationError, RuntimeError};
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, MethodAuthTemplate, PackageDefinition,
};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, FieldSchema,
    FunctionSchemaInit, TypeRef,
};
use radix_engine_interface::schema::{BlueprintSchemaInit, BlueprintStateSchemaInit};
use resources_tracker_macro::trace_resources;

pub const INTENT_HASH_STORE_BLUEPRINT: &str = "IntentHashStore";

pub const INTENT_HASH_STORE_CREATE_IDENT: &str = "create";

pub const INTENT_HASH_STORE_CREATE_EXPORT_NAME: &str = "create";

#[derive(Debug, Clone, ScryptoSbor)]
pub struct IntentHashStoreCreateInput {
    pub address_reservation: GlobalAddressReservation,
}

#[derive(Debug, Clone, ManifestSbor)]
pub struct IntentHashStoreCreateManifestInput {
    pub address_reservation: ManifestAddressReservation,
}

pub type IntentHashStoreCreateOutput = ComponentAddress;

pub struct IntentHashStoreNativePackage;

pub const PARTITION_MIN: u8 = MAIN_BASE_PARTITION.0 + 1;
pub const PARTITION_MAX: u8 = u8::MAX;
pub const EPOCHS_PER_PARTITION: usize = 100;

impl IntentHashStoreNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<Hash>();
        let value_type_index = aggregator.add_child_type_and_descendents::<IntentHashStatus>();

        let mut collections: Vec<BlueprintCollectionSchema<TypeRef<LocalTypeIndex>>> = vec![];
        for _ in PARTITION_MIN..=PARTITION_MAX {
            collections.push(BlueprintCollectionSchema::KeyValueStore(
                schema::BlueprintKeyValueStoreSchema {
                    key: TypeRef::Static(key_type_index),
                    value: TypeRef::Static(value_type_index),
                    can_own: false,
                },
            ))
        }

        let mut fields = Vec::new();
        fields.push(FieldSchema::static_field(
            aggregator.add_child_type_and_descendents::<IntentHashStore>(),
        ));

        let mut functions = BTreeMap::new();
        functions.insert(
            INTENT_HASH_STORE_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<IntentHashStoreCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<IntentHashStoreCreateOutput>(),
                ),
                export: INTENT_HASH_STORE_CREATE_EXPORT_NAME.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            INTENT_HASH_STORE_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                outer_blueprint: None,
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

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                    ),
                    method_auth: MethodAuthTemplate::Static {
                        auth: btreemap!(),
                        outer_auth: btreemap!(),
                    },
                },
            }
        );

        PackageDefinition { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            INTENT_HASH_STORE_CREATE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: IntentHashStoreCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IntentHashStoreBlueprint::create(input.address_reservation, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub enum IntentHashStatus {
    CommittedSuccess,
    CommittedFailure,
    Cancelled,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct IntentHashStore {
    current_partition: u8,
}

pub struct IntentHashStoreBlueprint;

impl IntentHashStoreBlueprint {
    pub fn create<Y>(
        address_reservation: GlobalAddressReservation,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let intent_store = api.new_simple_object(
            INTENT_HASH_STORE_BLUEPRINT,
            vec![scrypto_encode(&IntentHashStore {
                current_partition: PARTITION_MIN,
            })
            .unwrap()],
        )?;
        let access_rules = AccessRules::create(Roles::new(), api)?.0;
        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

        let address = api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => intent_store,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address_reservation,
        )?;
        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_coverage() {
        let covered_days = EPOCHS_PER_PARTITION as f64
            * (PARTITION_MAX as f64 - (PARTITION_MIN as f64 - 1.0) - 1.0)
            * 5.0
            / 60.0
            / 24.0;

        assert_eq!(covered_days.floor() as usize, 65);
    }
}
