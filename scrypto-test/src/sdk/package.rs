use std::path::Path;

use crate::prelude::*;
use scrypto_compiler::*;

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
        // Build
        let wasm_path = match ScryptoCompiler::new()
            .manifest_directory(path.as_ref())
            .compile()
        {
            Ok(wasm_path) => wasm_path,
            Err(error) => panic!(
                "Failed to compile package: {:?}, error: {:?}",
                path.as_ref(),
                error
            ),
        };

        // Extract schema
        let code = std::fs::read(&wasm_path).unwrap_or_else(|err| {
            panic!(
                "Failed to read built WASM from path {:?} - {:?}",
                &wasm_path, err
            )
        });
        let definition = extract_definition(&code).unwrap();

        (code, definition)
    }
}
