use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemLockData;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{BlueprintSetup, PackageSetup};
use radix_engine_interface::blueprints::transaction_processor::*;
use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::FunctionSchema;
use resources_tracker_macro::trace_resources;

use super::TransactionProcessorBlueprint;
use super::TransactionProcessorRunInput;

pub struct TransactionProcessorNativePackage;

impl TransactionProcessorNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(aggregator.add_child_type_and_descendents::<()>());

        let mut functions = BTreeMap::new();
        functions.insert(
            TRANSACTION_PROCESSOR_RUN_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<TransactionProcessorRunInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<TransactionProcessorRunOutput>(),
                export_name: TRANSACTION_PROCESSOR_RUN_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            TRANSACTION_PROCESSOR_BLUEPRINT.to_string() => BlueprintSetup {
                schema: BlueprintSchema {
                    outer_blueprint: None,
                    schema,
                    fields,
                    collections: vec![],
                    functions,
                    virtual_lazy_load_functions: btreemap!(),
                    event_schema: [].into(),
                    dependencies: btreeset!(),
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: btreemap!(),
                },
                function_access_rules: btreemap!(
                    TRANSACTION_PROCESSOR_RUN_IDENT.to_string() => rule!(allow_all), // TODO: Change to only allow root to call?
                ),
            }
        );

        PackageSetup {
            blueprints,
            royalty_config: btreemap!(),
        }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        match export_name {
            TRANSACTION_PROCESSOR_RUN_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                TransactionProcessorBlueprint::run(input, api)
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
