use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemLockData;
use radix_blueprint_schema_init::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, BlueprintSchemaInit,
    BlueprintStateSchemaInit, FunctionSchemaInit, TypeRef,
};
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::blueprints::transaction_processor::*;

use super::TransactionProcessorBlueprint;
use super::TransactionProcessorV1MinorVersion;

pub struct TransactionProcessorNativePackage;

impl TransactionProcessorNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let fields = Vec::new();

        let mut functions = index_map_new();
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
        let blueprints = indexmap!(
            TRANSACTION_PROCESSOR_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: true,
                feature_set: indexset!(),
                dependencies: indexset!(),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                    },
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    hooks: BlueprintHooksInit::default(),
                },
                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    // Only allow the root call frame to call any function in transaction processor.
                    // This is a safety precaution to reduce surface area of attack. This may be removed
                    // if/when the transaction processor is verified to be safe.
                    function_auth: FunctionAuth::RootOnly,
                    method_auth: MethodAuthTemplate::AllowAll,
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    >(
        export_name: &str,
        input: &IndexedScryptoValue,
        version: TransactionProcessorV1MinorVersion,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            TRANSACTION_PROCESSOR_RUN_IDENT => {
                let input: TransactionProcessorRunInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = TransactionProcessorBlueprint::run(
                    input.manifest_encoded_instructions,
                    input.global_address_reservations,
                    input.references,
                    input.blobs,
                    version,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
