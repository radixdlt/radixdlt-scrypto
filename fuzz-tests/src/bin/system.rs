#![cfg_attr(feature = "libfuzzer-sys", no_main)]

use arbitrary::Arbitrary;
#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_common::manifest_args;

use radix_engine::prelude::ManifestArgs;
use radix_engine_common::prelude::{Own, scrypto_encode, ScryptoCustomTypeKind};
use radix_engine_interface::api::{ACTOR_STATE_SELF, FieldHandle, FieldValue, KeyValueStoreDataSchema, LockFlags};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::prelude::{AttachedModuleId, ClientApi, OwnerRole};
use radix_engine_interface::types::IndexedScryptoValue;
use sbor::{generate_full_schema, LocalTypeId, TypeAggregator};
use sbor::basic_well_known_types::ANY_TYPE;
use scrypto_unit::TestRunnerBuilder;
use transaction::builder::ManifestBuilder;
use utils::indexmap;


// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|actions: SystemActions| {
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(
            CUSTOM_PACKAGE_CODE_ID,
            FuzzSystem(actions),
        ))
        .skip_receipt_check()
        .build();

    let component_address = {
        let package_address = test_runner.publish_native_package(
            CUSTOM_PACKAGE_CODE_ID,
            PackageDefinition::new_with_fields_test_definition(
                BLUEPRINT_NAME,
                2,
                vec![("test", "test", true), ("new", "new", false)],
            ),
        );
        let receipt = test_runner.execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32)
                .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
                .build(),
            vec![],
        );
        receipt.expect_commit_success().new_component_addresses()[0]
    };

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_method(component_address, "test", manifest_args!())
            .build(),
        vec![],
    );
});

#[derive(Debug, Clone, Arbitrary)]
struct SystemActions {
    pub actions: [SystemAction; 5],
}

#[derive(Debug, Clone, Arbitrary)]
enum SystemAction {
    FieldOpen(u8, u32),
    FieldRead(usize),
    FieldWrite(usize, Vec<u8>),
    FieldLock(usize),
    FieldClose(usize),
}

struct AppState {
    handles: Vec<FieldHandle>,
}

impl SystemAction {
    fn act<Y: ClientApi<RuntimeError>>(
        &self,
        api: &mut Y,
        state: &mut AppState,
    ) -> Result<(), RuntimeError> {
        match self {
            SystemAction::FieldOpen(index, flags) => unsafe {
                let handle = api.actor_open_field(ACTOR_STATE_SELF, *index, LockFlags::from_bits_unchecked(*flags))?;
                state.handles.push(handle);
            }
            SystemAction::FieldRead(index) => {
                if !state.handles.is_empty() {
                    let handle = state.handles[(*index) % state.handles.len()];
                    api.field_read(handle)?;
                }
            }
            SystemAction::FieldWrite(index, value) => {
                if !state.handles.is_empty() {
                    let handle = state.handles[(*index) % state.handles.len()];
                    api.field_write(handle, value.clone())?;
                }
            }
            SystemAction::FieldLock(index) => {
                if !state.handles.is_empty() {
                    let handle = state.handles[(*index) % state.handles.len()];
                    api.field_lock(handle)?;
                }
            }
            SystemAction::FieldClose(index) => {
                if !state.handles.is_empty() {
                    let handle = state.handles[(*index) % state.handles.len()];
                    api.field_close(handle)?;
                }
            }
        }

        Ok(())
    }
}

const BLUEPRINT_NAME: &str = "MyBlueprint";
const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
#[derive(Clone)]
struct FuzzSystem(SystemActions);
impl VmInvoke for FuzzSystem {
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        _input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match export_name {
            "test" => {
                let mut state = AppState {
                    handles: vec![]
                };
                for action in &self.0.actions {
                    action.act(api, &mut state)?;
                }
            }
            "new" => {
                let metadata = Metadata::create(api)?;
                let access_rules = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                let aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
                let kv_store = api.key_value_store_new(KeyValueStoreDataSchema::Local {
                    additional_schema: generate_full_schema(aggregator),
                    key_type: LocalTypeId::WellKnown(ANY_TYPE),
                    value_type: LocalTypeId::WellKnown(ANY_TYPE),
                    allow_ownership: true,
                })?;

                let node_id = api.new_simple_object(
                    BLUEPRINT_NAME,
                    indexmap![
                        0u8 => FieldValue::new(()),
                        1u8 => FieldValue::new(Own(kv_store)),
                    ],
                )?;

                api.globalize(
                    node_id,
                    indexmap!(
                        AttachedModuleId::Metadata => metadata.0,
                        AttachedModuleId::RoleAssignment => access_rules.0.0,
                    ),
                    None,
                )?;
            }
            _ => {}
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}