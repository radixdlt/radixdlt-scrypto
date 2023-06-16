use crate::errors::{ApplicationError, RuntimeError};
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, MethodAuthTemplate, PackageDefinition,
};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit,
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

impl IntentHashStoreNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<Hash>();
        let value_type_index = aggregator.add_child_type_and_descendents::<IntentHashStatus>();

        let mut collections: Vec<BlueprintCollectionSchema<TypeRef<LocalTypeIndex>>> = vec![];
        for _ in u8::MIN..u8::MAX {
            collections.push(BlueprintCollectionSchema::KeyValueStore(
                schema::BlueprintKeyValueStoreSchema {
                    key: TypeRef::Static(key_type_index),
                    value: TypeRef::Static(value_type_index),
                    can_own: false,
                },
            ))
        }

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
                        fields: vec![],
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
            INTENT_HASH_STORE_CREATE_IDENT => {
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

pub struct IntentHashStoreBlueprint;

impl IntentHashStoreBlueprint {
    pub fn create<Y>(
        address_reservation: GlobalAddressReservation,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let address = api.globalize_with_address(btreemap!(), address_reservation)?;
        Ok(address)
    }
}
