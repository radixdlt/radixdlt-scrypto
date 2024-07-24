use super::*;
use crate::internal_prelude::*;
use radix_engine_interface::blueprints::locker::*;
use radix_engine_interface::blueprints::package::*;
use sbor::prelude::*;

pub struct LockerNativePackage;

impl LockerNativePackage {
    pub fn definition() -> PackageDefinition {
        let blueprints = indexmap!(
            ACCOUNT_LOCKER_BLUEPRINT.to_string() => AccountLockerBlueprint::definition()
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // Delegated to the blueprint's dispatcher since it's the only blueprint in the package. If
        // we add more then we need to control the dispatch here.
        AccountLockerBlueprint::invoke_export(export_name, input, api)
    }
}
