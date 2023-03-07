use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_substates::RuntimeSubstate;
use crate::types::*;
use native_sdk::access_rules::AccessRulesObject;
use native_sdk::metadata::Metadata;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::package::TRANSACTION_PROCESSOR_RUN_IDENT;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::FunctionSchema;
use radix_engine_interface::schema::PackageSchema;

pub struct TransactionProcessorNativePackage;

impl TransactionProcessorNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<TransactionProcessorSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            TransactionProcessor_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<TransactionProcessorCreateInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<TransactionProcessorCreateOutput>(),
                export_name: TransactionProcessor_CREATE_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                TransactionProcessor_BLUEPRINT.to_string() => BlueprintSchema {
                    schema,
                    substates,
                    functions
                }
            ),
        }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            TRANSACTION_PROCESSOR_RUN_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                TransactionProcessorBlueprint::create(input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
