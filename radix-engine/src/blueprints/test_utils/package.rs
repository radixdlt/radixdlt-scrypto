use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::blueprints::test_utils::invocations::*;

use super::TestUtilsBlueprint;

pub struct TestUtilsNativePackage;

impl TestUtilsNativePackage {
    pub fn definition() -> PackageDefinition {
        PackageDefinition {
            blueprints: btreemap! {
                TEST_UTILS_BLUEPRINT.to_owned() => TestUtilsBlueprint::get_definition()
            },
        }
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
            TEST_UTILS_PANIC_IDENT => {
                let TestUtilsPanicInput(input) = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = TestUtilsBlueprint::panic(input.as_str(), api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
