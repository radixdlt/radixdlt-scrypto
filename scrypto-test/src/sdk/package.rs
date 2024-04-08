use crate::environment::*;
use radix_common::constants::*;
use radix_common::types::*;
use radix_common::*;
use radix_engine::errors::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::*;
use radix_substate_store_interface::interface::*;
use radix_substate_store_queries::typed_substate_layout::*;
use sbor::prelude::*;
use scrypto_compiler::*;
use std::path::*;

pub struct PackageFactory;

impl PackageFactory {
    pub fn publish<D>(
        code: Vec<u8>,
        package_definition: PackageDefinition,
        metadata: MetadataInit,
        env: &mut TestEnvironment<D>,
    ) -> Result<(PackageAddress, Bucket), RuntimeError>
    where
        D: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        env.with_auth_module_disabled(|env| {
            env.call_function_typed::<PackagePublishWasmInput, PackagePublishWasmOutput>(
                PACKAGE_PACKAGE,
                PACKAGE_BLUEPRINT,
                PACKAGE_PUBLISH_WASM_IDENT,
                &PackagePublishWasmInput {
                    code,
                    definition: package_definition,
                    metadata,
                },
            )
        })
    }

    pub fn publish_advanced<D>(
        owner_role: OwnerRole,
        definition: PackageDefinition,
        code: Vec<u8>,
        metadata: MetadataInit,
        package_address: Option<GlobalAddressReservation>,
        env: &mut TestEnvironment<D>,
    ) -> Result<PackageAddress, RuntimeError>
    where
        D: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        env.with_auth_module_disabled(|env| {
            env.call_function_typed::<PackagePublishWasmAdvancedInput, PackagePublishWasmAdvancedOutput>(
                PACKAGE_PACKAGE,
                PACKAGE_BLUEPRINT,
                PACKAGE_PUBLISH_WASM_ADVANCED_IDENT,
                &PackagePublishWasmAdvancedInput {
                    owner_role,
                    definition,
                    code,
                    metadata,
                    package_address
                },
            )
        })
    }

    pub fn compile_and_publish<P, D>(
        path: P,
        env: &mut TestEnvironment<D>,
    ) -> Result<PackageAddress, RuntimeError>
    where
        P: AsRef<Path>,
        D: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        let (wasm, package_definition) = Self::compile(path);
        Self::publish_advanced(
            OwnerRole::None,
            package_definition,
            wasm,
            Default::default(),
            Default::default(),
            env,
        )
    }

    pub fn compile<P>(path: P) -> (Vec<u8>, PackageDefinition)
    where
        P: AsRef<Path>,
    {
        // Initialize compiler
        let mut compiler = ScryptoCompiler::builder()
            .manifest_path(path.as_ref())
            .optimize_with_wasm_opt(None)
            .build()
            .unwrap_or_else(|err| panic!("Failed to initialize Scrypto Compiler  {:?}", err));

        // Build
        let mut build_artifacts = compiler.compile().unwrap_or_else(|err| {
            panic!(
                "Failed to compile package: {:?}, error: {:?}",
                path.as_ref(),
                err
            )
        });

        if !build_artifacts.is_empty() {
            let build_artifact = build_artifacts.remove(0); // take first element
            (
                build_artifact.wasm.content,
                build_artifact.package_definition.content,
            )
        } else {
            panic!("Build artifacts list is empty: {:?}", path.as_ref(),);
        }
    }
}
