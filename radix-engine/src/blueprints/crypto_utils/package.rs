use crate::errors::{ApplicationError, RuntimeError};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::schema::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, FunctionSchemaInit, TypeRef,
};
use radix_engine_interface::schema::{BlueprintSchemaInit, BlueprintStateSchemaInit};
use transaction::prelude::*;
use transaction::validation::*;

pub const CRYPTO_UTILS_BLUEPRINT: &str = "CryptoUtils";

pub const CRYPTO_UTILS_BLS_VERIFY_IDENT: &str = "bls_verify";
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct CryptoUtilsBlsVerifyInput {
    pub msg_hash: Hash,
    pub pub_key: BlsPublicKey,
    pub signature: BlsSignature,
}
pub type CryptoUtilsBlsVerifyOutput = bool;

pub struct CryptoUtilsNativePackage;

impl CryptoUtilsNativePackage {
    pub fn definition() -> PackageDefinition {
        let blueprints = indexmap!(
        CRYPTO_UTILS_BLUEPRINT.to_string() => CryptoUtilsBlueprint::get_definition(),
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
            CRYPTO_UTILS_BLS_VERIFY_IDENT => {
                let input: CryptoUtilsBlsVerifyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = CryptoUtilsBlueprint::bls_verify(
                    input.msg_hash,
                    input.pub_key,
                    input.signature,
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

pub struct CryptoUtilsBlueprint;

impl CryptoUtilsBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let functions = indexmap! {
            CRYPTO_UTILS_BLS_VERIFY_IDENT.to_string() => FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<CryptoUtilsBlsVerifyInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<CryptoUtilsBlsVerifyOutput>(),
                ),
                export: CRYPTO_UTILS_BLS_VERIFY_IDENT.to_string(),
            }
        };
        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set: Default::default(),
            dependencies: Default::default(),
            schema: BlueprintSchemaInit {
                generics: Default::default(),
                schema,
                state: BlueprintStateSchemaInit {
                    fields: Default::default(),
                    collections: Default::default(),
                },
                events: BlueprintEventSchemaInit::default(),
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },
            royalty_config: Default::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::default(),
            },
        }
    }

    pub fn bls_verify<Y>(
        msg_hash: Hash,
        pub_key: BlsPublicKey,
        signature: BlsSignature,
        _api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Ok(verify_bls(&msg_hash, &pub_key, &signature))
    }
}
