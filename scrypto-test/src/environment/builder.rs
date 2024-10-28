use radix_common::prelude::*;
use radix_engine::kernel::call_frame::*;
use radix_engine::kernel::heap::*;
use radix_engine::kernel::id_allocator::*;
use radix_engine::kernel::kernel::*;
use radix_engine::kernel::substate_io::*;
use radix_engine::kernel::substate_locks::*;
use radix_engine::system::actor::*;
use radix_engine::system::system::*;
use radix_engine::system::system_callback::*;
use radix_engine::system::system_modules::auth::*;
use radix_engine::system::system_modules::costing::*;
use radix_engine::system::system_modules::execution_trace::ExecutionTraceModule;
use radix_engine::system::system_modules::kernel_trace::KernelTraceModule;
use radix_engine::system::system_modules::limits::LimitsModule;
use radix_engine::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use radix_engine::system::system_modules::*;
use radix_engine::track::*;
use radix_engine::transaction::*;
use radix_engine::updates::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::*;
use radix_substate_store_interface::interface::*;

use crate::sdk::PackageFactory;

use super::*;

pub type DbFlash = IndexMap<DbNodeKey, IndexMap<DbPartitionNum, IndexMap<DbSortKey, Vec<u8>>>>;

pub struct TestEnvironmentBuilder<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    /// The database to use for the test environment.
    database: D,

    /// The database that substates are flashed to and then flashed to the actual database at build
    /// time. This is to make sure that when we add methods for changing the database it doesn't
    /// matter if flash is called before the set database method.
    flash_database: FlashSubstateDatabase,

    /// Additional references to add to the root [`CallFrame`] upon its creation.
    added_global_references: IndexSet<GlobalAddress>,

    /// The protocol updates the the user wishes to execute.
    /// This defaults to all from genesis.
    protocol_executor: ProtocolExecutor,
}

impl Default for TestEnvironmentBuilder<InMemorySubstateDatabase> {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnvironmentBuilder<InMemorySubstateDatabase> {
    pub fn new() -> Self {
        TestEnvironmentBuilder {
            database: InMemorySubstateDatabase::standard(),
            flash_database: FlashSubstateDatabase::standard(),
            added_global_references: Default::default(),
            protocol_executor: ProtocolBuilder::for_simulator().from_bootstrap_to_latest(),
        }
    }
}

impl<D> TestEnvironmentBuilder<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    const DEFAULT_INTENT_HASH: Hash = Hash([0; 32]);

    pub fn flash(mut self, data: DbFlash) -> Self {
        // Flash the substates to the database.
        let database_updates = DatabaseUpdates {
            node_updates: data
                .into_iter()
                .map(|(db_node_key, partition_num_to_updates_mapping)| {
                    (
                        db_node_key,
                        NodeDatabaseUpdates {
                            partition_updates: partition_num_to_updates_mapping
                                .into_iter()
                                .map(|(partition_num, substates)| {
                                    (
                                        partition_num,
                                        PartitionDatabaseUpdates::Delta {
                                            substate_updates: substates
                                                .into_iter()
                                                .map(|(db_sort_key, value)| {
                                                    (db_sort_key, DatabaseUpdate::Set(value))
                                                })
                                                .collect(),
                                        },
                                    )
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        };
        self.flash_database.commit(&database_updates);

        self
            /* Global references found in the NodeKeys */
            .add_global_references(
                database_updates
                    .node_ids()
                    .filter_map(|item| GlobalAddress::try_from(item).ok()),
            )
            /* Global references found in the Substate Values */
            .add_global_references(
                database_updates
                    .node_updates
                    .values()
                    .flat_map(|NodeDatabaseUpdates { partition_updates }| {
                        partition_updates.values()
                    })
                    .flat_map(|item| -> Box<dyn Iterator<Item = &Vec<u8>>> {
                        match item {
                            PartitionDatabaseUpdates::Delta { substate_updates } => {
                                Box::new(substate_updates.values().filter_map(|item| {
                                    if let DatabaseUpdate::Set(value) = item {
                                        Some(value)
                                    } else {
                                        None
                                    }
                                }))
                            }
                            PartitionDatabaseUpdates::Reset {
                                new_substate_values,
                            } => Box::new(new_substate_values.values()),
                        }
                    })
                    .flat_map(|value| {
                        IndexedScryptoValue::from_slice(value)
                            .unwrap()
                            .references()
                            .clone()
                    })
                    .filter_map(|item| GlobalAddress::try_from(item).ok()),
            )
    }

    pub fn add_global_reference(mut self, global_address: GlobalAddress) -> Self {
        self.added_global_references.insert(global_address);
        self
    }

    pub fn add_global_references(
        mut self,
        global_addresses: impl IntoIterator<Item = GlobalAddress>,
    ) -> Self {
        self.added_global_references.extend(global_addresses);
        self
    }

    pub fn database<ND>(self, database: ND) -> TestEnvironmentBuilder<ND>
    where
        ND: SubstateDatabase + CommittableSubstateDatabase,
    {
        TestEnvironmentBuilder {
            database,
            added_global_references: self.added_global_references,
            flash_database: self.flash_database,
            protocol_executor: self.protocol_executor,
        }
    }

    /// Can be used to configure whether to bootstrap and which protocol
    /// updates to run.
    pub fn with_protocol(
        mut self,
        executor: impl FnOnce(ProtocolBuilder) -> ProtocolExecutor,
    ) -> Self {
        self.protocol_executor = executor(ProtocolBuilder::for_simulator());
        self
    }

    pub fn build(mut self) -> TestEnvironment<D> {
        // Create the various VMs we will use
        let native_vm = NativeVm::new();
        let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();

        // Run bootstrap and any protocol updates against the database, if requested.
        self.protocol_executor
            .commit_each_protocol_update(&mut self.database);

        // Create the Id allocator we will be using throughout this test
        let id_allocator = IdAllocator::new(Self::DEFAULT_INTENT_HASH);

        // If a flash is specified execute it.
        let database_updates = self.flash_database.database_updates();
        if !database_updates.node_updates.is_empty() {
            self.database.commit(&database_updates);
        }

        // Getting the kernel boot to use for the kernel creation.
        let kernel_boot = KernelBoot::load(&self.database);

        let mut env = TestEnvironment(EncapsulatedRadixEngine::create(
            self.database,
            scrypto_vm,
            native_vm.clone(),
            id_allocator,
            |substate_database| Track::new(substate_database),
            |scrypto_vm, database| {
                let vm_boot = VmBoot::load(database);

                let transaction_runtime_module = TransactionRuntimeModule::new(
                    NetworkDefinition::simulator(),
                    Self::DEFAULT_INTENT_HASH,
                );

                let auth_module = AuthModule::new();

                let limits_module = LimitsModule::from_params(LimitParameters::babylon_genesis());

                let system_version = SystemVersion::latest();

                let costing_module = CostingModule {
                    current_depth: 0,
                    fee_reserve: SystemLoanFeeReserve::default(),
                    fee_table: FeeTable::new(system_version),
                    tx_payload_len: 0,
                    tx_num_of_signature_validations: 0,
                    config: CostingModuleConfig::babylon_genesis(),
                    cost_breakdown: Some(Default::default()),
                    detailed_cost_breakdown: Some(Default::default()),
                    on_apply_cost: Default::default(),
                };

                System::new(
                    system_version,
                    Vm {
                        scrypto_vm,
                        native_vm: native_vm.clone(),
                        vm_boot,
                    },
                    SystemModuleMixer::new(
                        EnabledModules::LIMITS
                            | EnabledModules::AUTH
                            | EnabledModules::TRANSACTION_RUNTIME,
                        KernelTraceModule,
                        transaction_runtime_module,
                        auth_module,
                        limits_module,
                        costing_module,
                        ExecutionTraceModule::new(MAX_EXECUTION_TRACE_DEPTH),
                    ),
                    SystemFinalization::no_nullifications(),
                )
            },
            |system_config, track, id_allocator| {
                Kernel::kernel_create_kernel_for_testing(
                    SubstateIO {
                        heap: Heap::new(),
                        store: track,
                        non_global_node_refs: NonGlobalNodeRefs::new(),
                        substate_locks: SubstateLocks::new(),
                        heap_transient_substates: TransientSubstates {
                            transient_substates: Default::default(),
                        },
                        pinned_to_heap: Default::default(),
                    },
                    id_allocator,
                    system_config,
                    kernel_boot.always_visible_global_nodes(),
                )
            },
        ));

        // Adding references to all of the well-known global nodes.
        env.0.with_kernel_mut(|kernel| {
            let (_, current_frame) = kernel.kernel_current_frame_mut();
            for node_id in GLOBAL_VISIBLE_NODES {
                let Ok(global_address) = GlobalAddress::try_from(node_id.0) else {
                    continue;
                };
                current_frame.add_global_reference(global_address)
            }
            for global_address in self.added_global_references.iter() {
                current_frame.add_global_reference(*global_address)
            }
        });

        // Publishing the test-environment package.
        let test_environment_package = {
            let code = include_bytes!("../../assets/test_environment.wasm");
            let package_definition = manifest_decode::<PackageDefinition>(include_bytes!(
                "../../assets/test_environment.rpd"
            ))
            .expect("Must succeed");

            env.with_auth_module_disabled(|env| {
                PackageFactory::publish_advanced(
                    OwnerRole::None,
                    package_definition,
                    code.to_vec(),
                    Default::default(),
                    None,
                    env,
                )
                .expect("Must succeed")
            })
        };

        // Creating the call-frame of the test environment & making it the current call frame
        {
            // Creating the auth zone of the next call-frame
            let auth_zone = env.0.with_kernel_mut(|kernel| {
                let mut system_service = SystemService::new(kernel);
                AuthModule::on_call_fn_mock(
                    &mut system_service,
                    Some((TRANSACTION_PROCESSOR_PACKAGE.as_node_id(), false)),
                    Default::default(),
                    Default::default(),
                )
                .expect("Must succeed")
            });

            // Define the actor of the next call-frame. This would be a function actor of the test
            // environment package.
            let actor = Actor::Function(FunctionActor {
                blueprint_id: BlueprintId {
                    package_address: test_environment_package,
                    blueprint_name: "TestEnvironment".to_owned(),
                },
                ident: "run".to_owned(),
                auth_zone,
            });

            // Creating the message, call-frame, and doing the replacement.
            let message = {
                let mut message =
                    CallFrameMessage::from_input(&IndexedScryptoValue::from_typed(&()), &actor);
                for node_id in GLOBAL_VISIBLE_NODES {
                    message.copy_global_references.push(node_id);
                }
                for global_address in self.added_global_references.iter() {
                    message
                        .copy_global_references
                        .push(global_address.into_node_id())
                }
                message
            };
            env.0.with_kernel_mut(|kernel| {
                let (substate_io, current_frame) = kernel.kernel_current_frame_mut();
                let new_frame =
                    CallFrame::new_child_from_parent(substate_io, current_frame, actor, message)
                        .expect("Must succeed.");
                let previous_frame = core::mem::replace(current_frame, new_frame);
                kernel.kernel_prev_frame_stack_mut().push(previous_frame)
            });
        }

        env
    }
}

//=========================
// Flash Substate Database
//=========================

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FlashSubstateDatabase {
    partitions: BTreeMap<DbPartitionKey, BTreeMap<DbSortKey, DbSubstateValue>>,
}

impl FlashSubstateDatabase {
    pub fn standard() -> Self {
        Self {
            partitions: BTreeMap::new(),
        }
    }

    pub fn database_updates(self) -> DatabaseUpdates {
        let mut database_updates = DatabaseUpdates::default();

        self.partitions.into_iter().for_each(
            |(
                DbPartitionKey {
                    node_key,
                    partition_num,
                },
                items,
            )| {
                database_updates
                    .node_updates
                    .entry(node_key)
                    .or_default()
                    .partition_updates
                    .insert(
                        partition_num,
                        PartitionDatabaseUpdates::Delta {
                            substate_updates: items
                                .into_iter()
                                .map(|(key, value)| (key, DatabaseUpdate::Set(value)))
                                .collect(),
                        },
                    );
            },
        );

        database_updates
    }
}

impl SubstateDatabase for FlashSubstateDatabase {
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.partitions
            .get(partition_key)
            .and_then(|partition| partition.get(sort_key))
            .cloned()
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let from_sort_key = from_sort_key.cloned();
        let iter = self
            .partitions
            .get(partition_key)
            .into_iter()
            .flat_map(|partition| partition.iter())
            .skip_while(move |(key, _substate)| Some(*key) < from_sort_key.as_ref())
            .map(|(key, substate)| (key.clone(), substate.clone()));

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for FlashSubstateDatabase {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        for (node_key, node_updates) in &database_updates.node_updates {
            for (partition_num, partition_updates) in &node_updates.partition_updates {
                let partition_key = DbPartitionKey {
                    node_key: node_key.clone(),
                    partition_num: *partition_num,
                };
                let partition = self.partitions.entry(partition_key.clone()).or_default();
                match partition_updates {
                    PartitionDatabaseUpdates::Delta { substate_updates } => {
                        for (sort_key, update) in substate_updates {
                            match update {
                                DatabaseUpdate::Set(substate_value) => {
                                    partition.insert(sort_key.clone(), substate_value.clone())
                                }
                                DatabaseUpdate::Delete => partition.remove(sort_key),
                            };
                        }
                    }
                    PartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    } => {
                        *partition = BTreeMap::from_iter(
                            new_substate_values
                                .iter()
                                .map(|(sort_key, value)| (sort_key.clone(), value.clone())),
                        )
                    }
                }
                if partition.is_empty() {
                    self.partitions.remove(&partition_key);
                }
            }
        }
    }
}
