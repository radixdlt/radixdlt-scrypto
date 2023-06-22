use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemLockData;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, MethodAuthTemplate, PackageDefinition,
};
use radix_engine_interface::blueprints::transaction_processor::*;
use radix_engine_interface::schema::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, BlueprintSchemaInit,
    BlueprintStateSchemaInit, FunctionSchemaInit, TypeRef,
};

use super::TransactionProcessorBlueprint;
use super::TransactionProcessorRunInput;

pub struct TransactionProcessorNativePackage;

impl TransactionProcessorNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let fields = Vec::new();

        let mut functions = BTreeMap::new();
        functions.insert(
            TRANSACTION_PROCESSOR_RUN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TransactionProcessorRunInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TransactionProcessorRunOutput>(),
                ),
                export: TRANSACTION_PROCESSOR_RUN_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            TRANSACTION_PROCESSOR_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                feature_set: btreeset!(),
                dependencies: btreeset!(),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                    events: BlueprintEventSchemaInit::default(),
                },
                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                        TRANSACTION_PROCESSOR_RUN_IDENT.to_string() => rule!(allow_all), // FIXME: Change to only allow root to call? and add auditors' tests
                    ),
                    method_auth: MethodAuthTemplate::AllowAll,
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
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        match export_name {
            TRANSACTION_PROCESSOR_RUN_IDENT => TransactionProcessorBlueprint::run(input, api),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
