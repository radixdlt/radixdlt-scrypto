use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemLockData;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    BlueprintDefinitionInit, FunctionSchemaInit, MethodAuthTemplate, PackageSetup,
};
use radix_engine_interface::blueprints::transaction_processor::*;
use radix_engine_interface::schema::BlueprintStateSchemaInit;
use resources_tracker_macro::trace_resources;

use super::TransactionProcessorBlueprint;
use super::TransactionProcessorRunInput;

pub struct TransactionProcessorNativePackage;

impl TransactionProcessorNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let fields = Vec::new();

        let mut functions = BTreeMap::new();
        functions.insert(
            TRANSACTION_PROCESSOR_RUN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<TransactionProcessorRunInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<TransactionProcessorRunOutput>(),
                export: TRANSACTION_PROCESSOR_RUN_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            TRANSACTION_PROCESSOR_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                outer_blueprint: None,
                dependencies: btreeset!(),
                feature_set: btreeset!(),
                blueprint: BlueprintStateSchemaInit {
                    fields,
                    collections: vec![],
                },
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
                schema,
                function_auth: btreemap!(
                    TRANSACTION_PROCESSOR_RUN_IDENT.to_string() => rule!(allow_all), // TODO: Change to only allow root to call?
                ),
                royalty_config: RoyaltyConfig::default(),
                template: MethodAuthTemplate {
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: btreemap!(),
                },
            }
        );

        PackageSetup { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        match export_name {
            TRANSACTION_PROCESSOR_RUN_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                TransactionProcessorBlueprint::run(input, api)
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
