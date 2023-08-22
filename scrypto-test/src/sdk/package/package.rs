use crate::prelude::*;

pub struct Package;

impl Package {
    pub fn publish(
        code: Vec<u8>,
        package_definition: PackageDefinition,
        metadata: MetadataInit,
        api: &mut TestRuntime,
    ) -> Result<(PackageAddress, Bucket), RuntimeError> {
        api.with_auth_module_disabled(|api| {
            api.call_function_typed::<PackagePublishWasmInput, PackagePublishWasmOutput>(
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

    pub fn publish_advanced(
        owner_role: OwnerRole,
        definition: PackageDefinition,
        code: Vec<u8>,
        metadata: MetadataInit,
        package_address: Option<GlobalAddressReservation>,
        api: &mut TestRuntime,
    ) -> Result<PackageAddress, RuntimeError> {
        api.with_auth_module_disabled(|api| {
            api.call_function_typed::<PackagePublishWasmAdvancedInput, PackagePublishWasmAdvancedOutput>(
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
}
