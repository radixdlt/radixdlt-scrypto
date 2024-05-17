use std::path::Path;

use crate::prelude::*;

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
        compile_profile: CompileProfile,
    ) -> Result<PackageAddress, RuntimeError>
    where
        P: AsRef<Path>,
        D: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        let (wasm, package_definition) = Self::compile(path, compile_profile);
        Self::publish_advanced(
            OwnerRole::None,
            package_definition,
            wasm,
            Default::default(),
            Default::default(),
            env,
        )
    }

    pub fn compile<P>(path: P, compile_profile: CompileProfile) -> (Vec<u8>, PackageDefinition)
    where
        P: AsRef<Path>,
    {
        Compile::compile_with_env_vars(path, BTreeMap::new(), compile_profile, false)
    }
}
