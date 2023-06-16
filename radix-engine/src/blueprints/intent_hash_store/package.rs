use crate::errors::{ApplicationError, RuntimeError};
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, MethodAuthTemplate, PackageDefinition,
};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, TypeRef,
};
use radix_engine_interface::schema::{BlueprintSchemaInit, BlueprintStateSchemaInit};
use resources_tracker_macro::trace_resources;

pub const INTENT_HASH_STORE_BLUEPRINT: &str = "IntentHashStore";

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
                        functions: btreemap!(),
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
        _input: &IndexedScryptoValue,
        _api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
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
