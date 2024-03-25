use super::*;
use radix_engine_interface::blueprints::locker::*;
use radix_engine_interface::blueprints::package::*;
use sbor::prelude::*;

pub struct AccountLockerNativePackage;

impl AccountLockerNativePackage {
    pub fn definition() -> PackageDefinition {
        let blueprints = indexmap!(
            ACCOUNT_LOCKER_BLUEPRINT.to_string() => AccountLockerBlueprint::definition()
        );

        PackageDefinition { blueprints }
    }
}
