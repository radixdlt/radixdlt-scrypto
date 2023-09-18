use monkey_tests::OverridePackageCode;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine::vm::VmInvoke;
use radix_engine_interface::api::{AttachedModuleId, ClientApi, LockFlags, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::package::PackageDefinition;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use sbor::basic_well_known_types::ANY_TYPE;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[derive(Clone)]
struct SystemFuzzer {
    rng: ChaCha8Rng,
    handles: Vec<u32>,
    nodes: IndexSet<NodeId>,
}

impl SystemFuzzer {
    fn new(seed: u64) -> Self {
        SystemFuzzer {
            rng: ChaCha8Rng::seed_from_u64(seed),
            handles: vec![],
            nodes: index_set_new(),
        }
    }

    fn add_handle(&mut self, handle: u32) {
        self.handles.push(handle);
    }

    fn next_node(&mut self) -> NodeId {
        let index = self.rng.gen_range(0usize..self.nodes.len());
        self.nodes.get_index(index).unwrap().clone()
    }

    fn next_handle(&mut self) -> u32 {
        if self.handles.is_empty() {
            return 0;
        }
        let index = self.rng.gen_range(0usize..self.handles.len());
        self.handles[index]
    }

    fn next_buffer(&mut self) -> Vec<u8> {
        scrypto_encode(&()).unwrap()
    }

    fn next_lock_flags(&mut self) -> LockFlags {
        if self.rng.gen_bool(0.5) {
            LockFlags::read_only()
        } else {
            LockFlags::MUTABLE
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum SystemFuzzAction {
    FieldOpen,
    FieldRead,
    FieldWrite,
    FieldLock,
    FieldClose,
    KeyValueStoreOpenEntry,
    KeyValueStoreRemoveEntry,
    KeyValueEntryGet,
    KeyValueEntrySet,
    KeyValueEntryRemove,
    KeyValueEntryLock,
    KeyValueEntryClose,
}

impl SystemFuzzAction {
    fn act<Y: ClientApi<RuntimeError>>(
        &self,
        api: &mut Y,
        fuzzer: &mut SystemFuzzer,
    ) -> Result<(), RuntimeError> {
        match self {
            SystemFuzzAction::FieldOpen => {
                let handle = api.actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::read_only())?;
                fuzzer.add_handle(handle);
            }
            SystemFuzzAction::FieldRead => {
                api.field_read(fuzzer.next_handle())?;
            }
            SystemFuzzAction::FieldWrite => {
                api.field_write(fuzzer.next_handle(), fuzzer.next_buffer())?;
            }
            SystemFuzzAction::FieldLock => {
                api.field_lock(fuzzer.next_handle())?;
            }
            SystemFuzzAction::FieldClose => {
                api.field_close(fuzzer.next_handle())?;
            }
            SystemFuzzAction::KeyValueStoreOpenEntry => {
                let handle = api.key_value_store_open_entry(
                    &fuzzer.next_node(),
                    &scrypto_encode(&()).unwrap(),
                    fuzzer.next_lock_flags(),
                )?;
                fuzzer.add_handle(handle);
            }
            SystemFuzzAction::KeyValueStoreRemoveEntry => {
                api.key_value_store_remove_entry(
                    &fuzzer.next_node(),
                    &scrypto_encode(&()).unwrap(),
                )?;
            }
            SystemFuzzAction::KeyValueEntryGet => {
                api.key_value_entry_get(fuzzer.next_handle())?;
            }
            SystemFuzzAction::KeyValueEntrySet => {
                api.key_value_entry_set(fuzzer.next_handle(), fuzzer.next_buffer())?;
            }
            SystemFuzzAction::KeyValueEntryRemove => {
                api.key_value_entry_remove(fuzzer.next_handle())?;
            }
            SystemFuzzAction::KeyValueEntryClose => {
                api.key_value_entry_close(fuzzer.next_handle())?;
            }
            SystemFuzzAction::KeyValueEntryLock => {
                api.key_value_entry_lock(fuzzer.next_handle())?;
            }
        }

        Ok(())
    }
}

// Arrange
const BLUEPRINT_NAME: &str = "MyBlueprint";
const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
#[derive(Clone)]
struct FuzzSystem(SystemFuzzer);
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
                let handle = api.actor_open_field(ACTOR_STATE_SELF, 1u8, LockFlags::read_only())?;
                let own: Own = api.field_read_typed(handle)?;
                self.0.nodes.insert(own.0);
                let node_id = api.actor_get_node_id(ACTOR_REF_SELF)?;
                self.0.nodes.insert(node_id);
                self.0.add_handle(handle);

                for _ in 0u8..10u8 {
                    let action =
                        SystemFuzzAction::from_repr(self.0.rng.gen_range(0u8..=11u8)).unwrap();
                    action.act(api, &mut self.0)?;
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

fn system_fuzz(seed: u64) -> TransactionReceipt {
    let fuzzer = SystemFuzzer::new(seed);
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(
            CUSTOM_PACKAGE_CODE_ID,
            FuzzSystem(fuzzer),
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

    receipt
}

#[test]
fn random_actions() {
    let success_count = (0u64..1000u64)
        .into_par_iter()
        .map(|seed| {
            let receipt = system_fuzz(seed);
            if receipt.is_commit_success() {
                1
            } else {
                0
            }
        })
        .reduce(|| 0, |acc, e| acc + e);

    println!("Success Count: {:?}", success_count);
}
