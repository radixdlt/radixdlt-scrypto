use super::*;
use crate::blueprints::consensus_manager::EpochChangeEvent;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_callback_api::ExecutionReceipt;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::system::system_modules::costing::*;
use crate::system::system_modules::execution_trace::*;
use crate::system::system_substate_schemas::*;
use crate::transaction::SystemStructure;
use colored::*;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_transactions::prelude::*;
use sbor::representations::*;

/// This type is not intended to be encoded or have a consistent scrypto encoding.
/// Some of the parts of it are encoded in the node, but not the receipt itself.
#[derive(Clone, PartialEq, Eq)]
pub struct TransactionReceipt {
    /// Costing parameters
    pub costing_parameters: CostingParameters,
    /// Transaction costing parameters
    pub transaction_costing_parameters: TransactionCostingParametersReceiptV2,
    /// Transaction fee summary
    pub fee_summary: TransactionFeeSummary,
    /// Transaction fee detail
    /// Available if `ExecutionConfig::enable_cost_breakdown` is enabled
    pub fee_details: Option<TransactionFeeDetails>,
    /// Transaction result
    pub result: TransactionResult,
    /// Hardware resources usage report
    /// Available if `resources_usage` feature flag is enabled
    pub resources_usage: Option<ResourcesUsage>,
    /// This field contains debug information about the transaction which is extracted during the
    /// transaction execution.
    pub debug_information: Option<TransactionDebugInformation>,
}

// Type for backwards compatibility to avoid integrator compile errors
// when they update.
pub type TransactionReceiptV1 = TransactionReceipt;

#[cfg(all(feature = "std", feature = "flamegraph"))]
impl TransactionReceipt {
    pub fn generate_execution_breakdown_flamegraph_svg_bytes(
        &self,
        title: impl AsRef<str>,
        network_definition: &NetworkDefinition,
    ) -> Result<Vec<u8>, FlamegraphError> {
        let title = title.as_ref();

        // The options to use when constructing the flamechart.
        let mut opts = inferno::flamegraph::Options::default();
        "Execution Cost Units".clone_into(&mut opts.count_name);
        opts.title = title.to_owned();

        // Transforming the detailed execution cost breakdown into a string understood by the flamegraph
        // library.
        let Some(TransactionDebugInformation {
            ref detailed_execution_cost_breakdown,
            ..
        }) = self.debug_information
        else {
            return Err(FlamegraphError::DetailedCostBreakdownNotAvailable);
        };

        let flamegraph_string = Self::transform_detailed_execution_breakdown_into_flamegraph_string(
            detailed_execution_cost_breakdown,
            network_definition,
        );

        // Writing the flamegraph string to a temporary file since its required by the flamegraph lib to
        // have a path.
        let result = {
            let tempfile = tempfile::NamedTempFile::new().map_err(FlamegraphError::IOError)?;
            std::fs::write(&tempfile, flamegraph_string).map_err(FlamegraphError::IOError)?;

            let mut result = std::io::Cursor::new(Vec::new());
            inferno::flamegraph::from_files(&mut opts, &[tempfile.path().to_owned()], &mut result)
                .map_err(|_| FlamegraphError::CreationError)?;

            result.set_position(0);
            result.into_inner()
        };

        Ok(result)
    }

    fn transform_detailed_execution_breakdown_into_flamegraph_string(
        detailed_execution_cost_breakdown: &[DetailedExecutionCostBreakdownEntry],
        network_definition: &NetworkDefinition,
    ) -> String {
        // Putting use in here so it doesn't cause unused import compile warning in no-std
        use crate::system::actor::*;

        let address_bech32m_encoder = AddressBech32Encoder::new(&network_definition);

        let mut lines = Vec::<String>::new();
        let mut path_stack = vec![];
        for (
            index,
            DetailedExecutionCostBreakdownEntry {
                item: execution_item,
                ..
            },
        ) in detailed_execution_cost_breakdown.iter().enumerate()
        {
            // Constructing the full path
            match execution_item {
                ExecutionCostBreakdownItem::Invocation { actor, .. } => {
                    let actor_string = match actor {
                        Actor::Root => "root".to_owned(),
                        Actor::Method(MethodActor {
                            node_id,
                            ref ident,
                            ref object_info,
                            ..
                        }) => {
                            format!(
                                "Method <{}>::{}::{}",
                                address_bech32m_encoder
                                    .encode(node_id.as_bytes())
                                    .expect("Encoding of an address can't fail"),
                                object_info.blueprint_info.blueprint_id.blueprint_name,
                                ident
                            )
                        }
                        Actor::Function(FunctionActor {
                            ref blueprint_id,
                            ref ident,
                            ..
                        }) => {
                            format!(
                                "Function <{}>::{}::{}",
                                address_bech32m_encoder
                                    .encode(blueprint_id.package_address.as_bytes())
                                    .expect("Encoding of an address can't fail"),
                                blueprint_id.blueprint_name,
                                ident
                            )
                        }
                        Actor::BlueprintHook(BlueprintHookActor {
                            hook,
                            ref blueprint_id,
                            ..
                        }) => {
                            format!(
                                "Blueprint Hook <{}>::{}::{:?}",
                                address_bech32m_encoder
                                    .encode(blueprint_id.package_address.as_bytes())
                                    .expect("Encoding of an address can't fail"),
                                blueprint_id.blueprint_name,
                                hook
                            )
                        }
                    };
                    path_stack.push(format!("Invocation: {actor_string} ({index})"))
                }
                ExecutionCostBreakdownItem::InvocationComplete => {
                    path_stack.pop();
                }
                ExecutionCostBreakdownItem::Execution {
                    simple_name,
                    cost_units,
                    ..
                } => {
                    lines.push(format!(
                        "{}{}({}) {}",
                        if path_stack.join(";").is_empty() {
                            "".to_owned()
                        } else {
                            format!("{};", path_stack.join(";"))
                        },
                        simple_name,
                        index,
                        cost_units
                    ));
                }
            }
        }

        lines.join("\n")
    }
}

impl ExecutionReceipt for TransactionReceipt {
    fn set_resource_usage(&mut self, resources_usage: ResourcesUsage) {
        self.resources_usage = Some(resources_usage);
    }
}

#[derive(Default, Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct TransactionFeeSummary {
    /// Total execution cost units consumed.
    pub total_execution_cost_units_consumed: u32,
    /// Total finalization cost units consumed.
    pub total_finalization_cost_units_consumed: u32,

    /// Total execution cost in XRD.
    pub total_execution_cost_in_xrd: Decimal,
    /// Total finalization cost in XRD.
    pub total_finalization_cost_in_xrd: Decimal,
    /// Total tipping cost in XRD.
    pub total_tipping_cost_in_xrd: Decimal,
    /// Total storage cost in XRD.
    pub total_storage_cost_in_xrd: Decimal,
    /// Total royalty cost in XRD.
    pub total_royalty_cost_in_xrd: Decimal,
}

#[derive(Default, Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct TransactionFeeDetails {
    /// Execution cost breakdown
    pub execution_cost_breakdown: BTreeMap<String, u32>,
    /// Finalization cost breakdown
    pub finalization_cost_breakdown: BTreeMap<String, u32>,
}

/// Captures whether a transaction should be committed, and its other results
#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum TransactionResult {
    Commit(CommitResult),
    Reject(RejectResult),
    Abort(AbortResult),
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct CommitResult {
    /// Substate updates
    pub state_updates: StateUpdates,
    /// Information extracted from the substate updates
    pub state_update_summary: StateUpdateSummary,
    /// The source of transaction fee
    pub fee_source: FeeSource,
    /// The destination of transaction fee
    pub fee_destination: FeeDestination,
    /// Transaction execution outcome
    pub outcome: TransactionOutcome,
    /// Events emitted
    pub application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    /// Logs emitted
    pub application_logs: Vec<(Level, String)>,
    /// Additional annotation on substates and events
    pub system_structure: SystemStructure,
    /// Transaction execution traces
    /// Available if `ExecutionTrace` module is enabled
    pub execution_trace: Option<TransactionExecutionTrace>,
    /// The actually performed nullifications.
    /// For example, a failed transaction won't include subintent nullifications.
    pub performed_nullifications: Vec<Nullification>,
}

#[derive(Debug, Clone, Default, ScryptoSbor, PartialEq, Eq)]
pub struct FeeSource {
    pub paying_vaults: IndexMap<NodeId, Decimal>,
}

#[derive(Debug, Clone, Default, ScryptoSbor, PartialEq, Eq)]
pub struct FeeDestination {
    pub to_proposer: Decimal,
    pub to_validator_set: Decimal,
    pub to_burn: Decimal,
    pub to_royalty_recipients: IndexMap<RoyaltyRecipient, Decimal>,
}

/// Captures whether a transaction's commit outcome is Success or Failure
#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum TransactionOutcome {
    Success(Vec<InstructionOutput>),
    Failure(RuntimeError),
}

#[derive(Debug, Clone, ScryptoSbor, Default, PartialEq, Eq)]
pub struct TransactionExecutionTrace {
    pub execution_traces: Vec<ExecutionTrace>,
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub fee_locks: FeeLocks,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, Default)]
pub struct FeeLocks {
    pub lock: Decimal,
    pub contingent_lock: Decimal,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum Nullification {
    Intent {
        expiry_epoch: Epoch,
        intent_hash: IntentHash,
    },
}

impl Nullification {
    pub fn of_intent(
        intent_hash_nullification: IntentHashNullification,
        current_epoch: Epoch,
        is_success: bool,
    ) -> Option<Self> {
        let (intent_hash, expiry_epoch) = match intent_hash_nullification {
            IntentHashNullification::TransactionIntent {
                intent_hash,
                expiry_epoch,
            } => (intent_hash.into(), expiry_epoch),
            IntentHashNullification::SimulatedTransactionIntent { simulated } => {
                let intent_hash = simulated.transaction_intent_hash();
                let expiry_epoch = simulated.expiry_epoch(current_epoch);
                (intent_hash.into(), expiry_epoch)
            }
            IntentHashNullification::Subintent {
                intent_hash: subintent_hash,
                expiry_epoch,
            } => {
                // Don't write subintent nullification on failure.
                // Subintents can't pay fees, so this isn't abusable.
                if !is_success {
                    return None;
                }
                (subintent_hash.into(), expiry_epoch)
            }
            IntentHashNullification::SimulatedSubintent { simulated } => {
                if !is_success {
                    return None;
                }
                let subintent_hash = simulated.subintent_hash();
                let expiry_epoch = simulated.expiry_epoch(current_epoch);
                (subintent_hash.into(), expiry_epoch)
            }
        };
        Some(Nullification::Intent {
            expiry_epoch,
            intent_hash,
        })
    }

    pub fn transaction_tracker_keys(self) -> (Epoch, Hash) {
        match self {
            Nullification::Intent {
                expiry_epoch,
                intent_hash,
            } => (expiry_epoch, intent_hash.into_hash()),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct RejectResult {
    pub reason: RejectionReason,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct AbortResult {
    pub reason: AbortReason,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Sbor)]
pub enum AbortReason {
    ConfiguredAbortTriggeredOnFeeLoanRepayment,
}

#[derive(Debug, Clone, Default, ScryptoSbor, PartialEq, Eq)]
pub struct ResourcesUsage {
    pub heap_allocations_sum: usize,
    pub heap_peak_memory: usize,
    pub cpu_cycles: u64,
}

/// A structure of debug information about the transaction execution.
///
/// This is intentionally not SBOR codable since we never want this data to be persisted or
/// transmitted over the wire.
#[derive(Clone, PartialEq, Eq)]
pub struct TransactionDebugInformation {
    /* Costing Breakdown */
    /// A detailed trace of where execution cost units were consumed.
    pub detailed_execution_cost_breakdown: Vec<DetailedExecutionCostBreakdownEntry>,
}

impl TransactionExecutionTrace {
    pub fn worktop_changes(&self) -> IndexMap<usize, Vec<WorktopChange>> {
        let mut aggregator = index_map_new::<usize, Vec<WorktopChange>>();
        for trace in &self.execution_traces {
            trace.worktop_changes(&mut aggregator)
        }
        aggregator
    }
}

impl TransactionResult {
    pub fn is_commit_success(&self) -> bool {
        match self {
            TransactionResult::Commit(c) => matches!(c.outcome, TransactionOutcome::Success(_)),
            _ => false,
        }
    }
}

impl CommitResult {
    pub fn empty_with_outcome(outcome: TransactionOutcome) -> Self {
        Self {
            state_updates: Default::default(),
            state_update_summary: Default::default(),
            fee_source: Default::default(),
            fee_destination: Default::default(),
            outcome,
            application_events: Default::default(),
            application_logs: Default::default(),
            system_structure: Default::default(),
            execution_trace: Default::default(),
            performed_nullifications: Default::default(),
        }
    }

    pub fn next_epoch(&self) -> Option<EpochChangeEvent> {
        // Note: Node should use a well-known index id
        for (ref event_type_id, ref event_data) in self.application_events.iter() {
            let is_consensus_manager = match &event_type_id.0 {
                Emitter::Method(node_id, ModuleId::Main)
                    if node_id.entity_type() == Some(EntityType::GlobalConsensusManager) =>
                {
                    true
                }
                Emitter::Function(blueprint_id)
                    if blueprint_id.package_address.eq(&CONSENSUS_MANAGER_PACKAGE) =>
                {
                    true
                }
                _ => false,
            };

            if is_consensus_manager {
                if let Ok(epoch_change_event) = scrypto_decode::<EpochChangeEvent>(&event_data) {
                    return Some(epoch_change_event);
                }
            }
        }
        None
    }

    pub fn new_package_addresses(&self) -> &IndexSet<PackageAddress> {
        &self.state_update_summary.new_packages
    }

    pub fn new_component_addresses(&self) -> &IndexSet<ComponentAddress> {
        &self.state_update_summary.new_components
    }

    pub fn new_resource_addresses(&self) -> &IndexSet<ResourceAddress> {
        &self.state_update_summary.new_resources
    }

    pub fn new_vault_addresses(&self) -> &IndexSet<InternalAddress> {
        &self.state_update_summary.new_vaults
    }

    pub fn vault_balance_changes(&self) -> &IndexMap<NodeId, (ResourceAddress, BalanceChange)> {
        &self.state_update_summary.vault_balance_changes
    }

    pub fn output<T: ScryptoDecode>(&self, nth: usize) -> T {
        match &self.outcome {
            TransactionOutcome::Success(o) => match o.get(nth) {
                Some(InstructionOutput::CallReturn(value)) => {
                    scrypto_decode::<T>(value).expect("Output can't be converted")
                }
                _ => panic!("No output for [{}]", nth),
            },
            TransactionOutcome::Failure(_) => panic!("Transaction failed"),
        }
    }

    pub fn state_updates(
        &self,
    ) -> BTreeMap<NodeId, BTreeMap<PartitionNumber, BTreeMap<SubstateKey, DatabaseUpdate>>> {
        let mut updates = BTreeMap::<
            NodeId,
            BTreeMap<PartitionNumber, BTreeMap<SubstateKey, DatabaseUpdate>>,
        >::new();
        for (node_id, x) in &self.state_updates.by_node {
            let NodeStateUpdates::Delta { by_partition } = x;
            for (partition_num, y) in by_partition {
                match y {
                    PartitionStateUpdates::Delta { by_substate } => {
                        for (substate_key, substate_update) in by_substate {
                            updates
                                .entry(node_id.clone())
                                .or_default()
                                .entry(partition_num.clone())
                                .or_default()
                                .insert(substate_key.clone(), substate_update.clone());
                        }
                    }
                    PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                        new_substate_values,
                    }) => {
                        for (substate_key, substate_value) in new_substate_values {
                            updates
                                .entry(node_id.clone())
                                .or_default()
                                .entry(partition_num.clone())
                                .or_default()
                                .insert(
                                    substate_key.clone(),
                                    DatabaseUpdate::Set(substate_value.clone()),
                                );
                        }
                    }
                }
            }
        }
        updates
    }

    /// Note - there is a better display of these on the receipt, which uses the schemas
    /// to display clear details
    pub fn state_updates_string(&self) -> String {
        let mut buffer = String::new();
        for (node_id, x) in &self.state_updates() {
            buffer.push_str(&format!("\n{:?}, {:?}\n", node_id, node_id.entity_type()));
            for (partition_num, y) in x {
                buffer.push_str(&format!("    {:?}\n", partition_num));
                for (substate_key, substate_update) in y {
                    buffer.push_str(&format!(
                        "        {}\n",
                        match substate_key {
                            SubstateKey::Field(x) => format!("Field: {}", x),
                            SubstateKey::Map(x) =>
                                format!("Map: {:?}", scrypto_decode::<ScryptoValue>(&x).unwrap()),
                            SubstateKey::Sorted(x) => format!(
                                "Sorted: {:?}, {:?}",
                                x.0,
                                scrypto_decode::<ScryptoValue>(&x.1).unwrap()
                            ),
                        },
                    ));
                    buffer.push_str(&format!(
                        "        {}\n",
                        match substate_update {
                            DatabaseUpdate::Set(x) =>
                                format!("Set: {:?}", scrypto_decode::<ScryptoValue>(&x).unwrap()),
                            DatabaseUpdate::Delete => format!("Delete"),
                        }
                    ));
                }
            }
        }
        buffer
    }
}

impl TransactionOutcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn expect_success(&self) -> &Vec<InstructionOutput> {
        match self {
            TransactionOutcome::Success(results) => results,
            TransactionOutcome::Failure(error) => panic!("Outcome was a failure: {error:?}"),
        }
    }

    pub fn expect_failure(&self) -> &RuntimeError {
        match self {
            TransactionOutcome::Success(_) => panic!("Outcome was an unexpected success"),
            TransactionOutcome::Failure(error) => error,
        }
    }

    pub fn success_or_else<E, F: Fn(&RuntimeError) -> E>(
        &self,
        f: F,
    ) -> Result<&Vec<InstructionOutput>, E> {
        match self {
            TransactionOutcome::Success(results) => Ok(results),
            TransactionOutcome::Failure(error) => Err(f(error)),
        }
    }
}

impl TransactionReceipt {
    /// An empty receipt for merging changes into.
    pub fn empty_with_commit(commit_result: CommitResult) -> Self {
        Self {
            costing_parameters: CostingParameters::babylon_genesis(),
            transaction_costing_parameters: Default::default(),
            fee_summary: Default::default(),
            fee_details: Default::default(),
            result: TransactionResult::Commit(commit_result),
            resources_usage: Default::default(),
            debug_information: Default::default(),
        }
    }

    pub fn empty_commit_success() -> Self {
        Self::empty_with_commit(CommitResult::empty_with_outcome(
            TransactionOutcome::Success(vec![]),
        ))
    }

    pub fn is_commit_success(&self) -> bool {
        matches!(
            self.result,
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Success(_),
                ..
            })
        )
    }

    pub fn is_commit_failure(&self) -> bool {
        matches!(
            self.result,
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Failure(_),
                ..
            })
        )
    }

    pub fn is_rejection(&self) -> bool {
        matches!(self.result, TransactionResult::Reject(_))
    }

    pub fn expect_commit_ignore_outcome(&self) -> &CommitResult {
        match &self.result {
            TransactionResult::Commit(c) => c,
            TransactionResult::Reject(e) => panic!("Transaction was rejected: {:?}", e),
            TransactionResult::Abort(_) => panic!("Transaction was aborted"),
        }
    }

    pub fn into_commit_ignore_outcome(self) -> CommitResult {
        match self.result {
            TransactionResult::Commit(c) => c,
            TransactionResult::Reject(e) => panic!("Transaction was rejected: {:?}", e),
            TransactionResult::Abort(_) => panic!("Transaction was aborted"),
        }
    }

    pub fn expect_commit(&self, success: bool) -> &CommitResult {
        let c = self.expect_commit_ignore_outcome();
        if c.outcome.is_success() != success {
            panic!(
                "Expected {} but was {}: {:?}",
                if success { "success" } else { "failure" },
                if c.outcome.is_success() {
                    "success"
                } else {
                    "failure"
                },
                c.outcome
            )
        }
        c
    }

    pub fn expect_commit_success(&self) -> &CommitResult {
        self.expect_commit(true)
    }

    pub fn expect_commit_failure(&self) -> &CommitResult {
        self.expect_commit(false)
    }

    pub fn expect_commit_failure_containing_error(&self, error_needle: &str) {
        let error_message = self
            .expect_commit_failure()
            .outcome
            .expect_failure()
            .to_string(NO_NETWORK);
        assert!(
            error_message.contains(error_needle),
            "{error_needle:?} was not contained in RuntimeError"
        );
    }

    pub fn expect_rejection(&self) -> &RejectionReason {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was commit"),
            TransactionResult::Reject(ref r) => &r.reason,
            TransactionResult::Abort(..) => panic!("Expected rejection but was abort"),
        }
    }

    pub fn expect_rejection_containing_error(&self, error_needle: &str) {
        let error_message = self.expect_rejection().to_string(NO_NETWORK);
        assert!(
            error_message.contains(error_needle),
            "{error_needle:?} was not contained in RejectionReason"
        );
    }

    pub fn expect_abortion(&self) -> &AbortReason {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected abortion but was commit"),
            TransactionResult::Reject(..) => panic!("Expected abortion but was reject"),
            TransactionResult::Abort(ref r) => &r.reason,
        }
    }

    pub fn expect_not_success(&self) {
        match &self.result {
            TransactionResult::Commit(c) => {
                if c.outcome.is_success() {
                    panic!("Transaction succeeded unexpectedly")
                }
            }
            TransactionResult::Reject(..) => {}
            TransactionResult::Abort(..) => {}
        }
    }

    pub fn expect_specific_rejection<F>(&self, f: F)
    where
        F: Fn(&RejectionReason) -> bool,
    {
        match &self.result {
            TransactionResult::Commit(..) => panic!("Expected rejection but was committed"),
            TransactionResult::Reject(result) => {
                if !f(&result.reason) {
                    panic!(
                        "Expected specific rejection but was different error:\n{:?}",
                        self
                    );
                }
            }
            TransactionResult::Abort(..) => panic!("Expected rejection but was abort"),
        }
    }

    pub fn expect_failure(&self) -> &RuntimeError {
        match &self.result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(_) => panic!("Expected failure but was success"),
                TransactionOutcome::Failure(error) => error,
            },
            TransactionResult::Reject(_) => panic!("Transaction was rejected"),
            TransactionResult::Abort(..) => panic!("Transaction was aborted"),
        }
    }

    pub fn expect_specific_failure<F>(&self, f: F)
    where
        F: Fn(&RuntimeError) -> bool,
    {
        if !f(self.expect_failure()) {
            panic!(
                "Expected specific failure but was different error:\n{:?}",
                self
            );
        }
    }

    pub fn expect_auth_failure(&self) {
        self.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
            )
        })
    }

    pub fn expect_auth_assertion_failure(&self) {
        self.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
            )
        })
    }

    pub fn effective_execution_cost_unit_price(&self) -> Decimal {
        // Below unwraps are safe, no chance to overflow considering current costing parameters
        self.costing_parameters
            .execution_cost_unit_price
            .checked_mul(
                Decimal::ONE
                    .checked_add(self.transaction_costing_parameters.tip_proportion)
                    .unwrap(),
            )
            .unwrap()
    }

    pub fn effective_finalization_cost_unit_price(&self) -> Decimal {
        let one_percent = Decimal::ONE_HUNDREDTH;

        // Below unwraps are safe, no chance to overflow considering current costing parameters
        self.costing_parameters
            .finalization_cost_unit_price
            .checked_mul(
                Decimal::ONE
                    .checked_add(
                        one_percent
                            .checked_mul(self.transaction_costing_parameters.tip_proportion)
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
    }
}

macro_rules! prefix {
    ($i:expr, $list:expr) => {
        if $i == $list.len() - 1 {
            "└─"
        } else {
            "├─"
        }
    };
}

impl fmt::Debug for TransactionReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.display(TransactionReceiptDisplayContext::default())
        )
    }
}

pub struct TransactionReceiptDisplayContext<'a> {
    pub encoder: Option<&'a AddressBech32Encoder>,
    pub system_database_reader: Option<SystemDatabaseReader<'a, dyn SubstateDatabase + 'a>>,
    pub display_state_updates: bool,
    pub use_ansi_colors: bool,
    pub max_substate_length_to_display: usize,
}

impl<'a> Default for TransactionReceiptDisplayContext<'a> {
    fn default() -> Self {
        Self {
            encoder: None,
            system_database_reader: None,
            display_state_updates: true,
            use_ansi_colors: true,
            max_substate_length_to_display: 1024,
        }
    }
}

impl<'a> TransactionReceiptDisplayContext<'a> {
    pub fn display_context(&self) -> ScryptoValueDisplayContext<'a> {
        ScryptoValueDisplayContext::with_optional_bech32(self.encoder)
    }

    pub fn address_display_context(&self) -> AddressDisplayContext<'a> {
        AddressDisplayContext {
            encoder: self.encoder,
        }
    }

    pub fn max_substate_length_to_display(&self) -> usize {
        self.max_substate_length_to_display
    }

    pub fn lookup_schema<T: AsRef<NodeId>>(
        &self,
        full_type_id: &FullyScopedTypeId<T>,
    ) -> Option<(LocalTypeId, Rc<VersionedScryptoSchema>)> {
        self.system_database_reader.as_ref().map(|system_reader| {
            let schema = system_reader
                .get_schema(full_type_id.0.as_ref(), &full_type_id.1)
                .unwrap();

            (full_type_id.2.clone(), schema)
        })
    }

    fn format_first_top_level_title_with_detail<F: fmt::Write, D: fmt::Display>(
        &self,
        f: &mut F,
        title: &str,
        detail: D,
    ) -> Result<(), fmt::Error> {
        if self.use_ansi_colors {
            write!(f, "{} {}", format!("{}:", title).bold().green(), detail)
        } else {
            write!(f, "{}: {}", title.to_uppercase(), detail)
        }
    }

    fn format_top_level_title_with_detail<F: fmt::Write, D: fmt::Display>(
        &self,
        f: &mut F,
        title: &str,
        detail: D,
    ) -> Result<(), fmt::Error> {
        if self.use_ansi_colors {
            write!(f, "\n{} {}", format!("{}:", title).bold().green(), detail)
        } else {
            write!(f, "\n\n{}: {}", title.to_uppercase(), detail)
        }
    }

    fn display_title(&self, title: &str) -> MaybeAnsi {
        if self.use_ansi_colors {
            MaybeAnsi::Ansi(title.bold().green())
        } else {
            MaybeAnsi::Normal(title.to_string())
        }
    }

    fn display_result(&self, result: &TransactionResult) -> MaybeAnsi {
        let (string, format): (String, fn(String) -> ColoredString) = match result {
            TransactionResult::Commit(c) => match &c.outcome {
                TransactionOutcome::Success(_) => ("COMMITTED SUCCESS".to_string(), |x| x.green()),
                TransactionOutcome::Failure(e) => (
                    format!("COMMITTED FAILURE: {}", e.display(self.display_context())),
                    |x| x.red(),
                ),
            },
            TransactionResult::Reject(r) => (
                format!("REJECTED: {}", r.reason.display(self.display_context())),
                |x| x.red(),
            ),
            TransactionResult::Abort(a) => (format!("ABORTED: {}", a.reason), |x| x.bright_red()),
        };
        if self.use_ansi_colors {
            MaybeAnsi::Ansi(format(string))
        } else {
            MaybeAnsi::Normal(string)
        }
    }

    fn display_log(&self, level: &Level, message: &str) -> (MaybeAnsi, MaybeAnsi) {
        let (level, format): (_, fn(&str) -> ColoredString) = match level {
            Level::Error => ("ERROR", |x| x.red()),
            Level::Warn => ("WARN", |x| x.yellow()),
            Level::Info => ("INFO", |x| x.green()),
            Level::Debug => ("DEBUG", |x| x.cyan()),
            Level::Trace => ("TRACE", |x| x.normal()),
        };

        if self.use_ansi_colors {
            (
                MaybeAnsi::Ansi(format(level)),
                MaybeAnsi::Ansi(format(message)),
            )
        } else {
            (
                MaybeAnsi::Normal(level.to_string()),
                MaybeAnsi::Normal(message.to_string()),
            )
        }
    }
}

enum MaybeAnsi {
    Ansi(ColoredString),
    Normal(String),
}

impl fmt::Display for MaybeAnsi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaybeAnsi::Ansi(value) => write!(f, "{}", value),
            MaybeAnsi::Normal(value) => write!(f, "{}", value),
        }
    }
}

impl<'a> From<&'a AddressBech32Encoder> for TransactionReceiptDisplayContext<'a> {
    fn from(encoder: &'a AddressBech32Encoder) -> Self {
        Self {
            encoder: Some(encoder),
            ..Default::default()
        }
    }
}

impl<'a> From<Option<&'a AddressBech32Encoder>> for TransactionReceiptDisplayContext<'a> {
    fn from(encoder: Option<&'a AddressBech32Encoder>) -> Self {
        Self {
            encoder,
            ..Default::default()
        }
    }
}

pub struct TransactionReceiptDisplayContextBuilder<'a>(TransactionReceiptDisplayContext<'a>);

impl<'a> TransactionReceiptDisplayContextBuilder<'a> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn encoder(mut self, encoder: &'a AddressBech32Encoder) -> Self {
        self.0.encoder = Some(encoder);
        self
    }

    pub fn schema_lookup_from_db(mut self, db: &'a dyn SubstateDatabase) -> Self {
        self.0.system_database_reader = Some(SystemDatabaseReader::new(db));
        self
    }

    pub fn display_state_updates(mut self, setting: bool) -> Self {
        self.0.display_state_updates = setting;
        self
    }

    pub fn use_ansi_colors(mut self, setting: bool) -> Self {
        self.0.use_ansi_colors = setting;
        self
    }

    pub fn set_max_substate_length_to_display(mut self, setting: usize) -> Self {
        self.0.max_substate_length_to_display = setting;
        self
    }

    pub fn build(self) -> TransactionReceiptDisplayContext<'a> {
        self.0
    }
}

impl<'a> ContextualDisplay<TransactionReceiptDisplayContext<'a>> for TransactionReceipt {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &TransactionReceiptDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        let result = &self.result;
        let scrypto_value_display_context = context.display_context();
        let address_display_context = context.address_display_context();

        context.format_first_top_level_title_with_detail(
            f,
            "Transaction Status",
            context.display_result(result),
        )?;

        context.format_top_level_title_with_detail(
            f,
            "Transaction Cost",
            format!("{} XRD", self.fee_summary.total_cost()),
        )?;
        write!(
            f,
            "\n├─ {} {} XRD, {} execution cost units",
            context.display_title("Network execution:"),
            self.fee_summary.total_execution_cost_in_xrd,
            self.fee_summary.total_execution_cost_units_consumed,
        )?;
        write!(
            f,
            "\n├─ {} {} XRD, {} finalization cost units",
            context.display_title("Network finalization:"),
            self.fee_summary.total_finalization_cost_in_xrd,
            self.fee_summary.total_finalization_cost_units_consumed,
        )?;
        write!(
            f,
            "\n├─ {} {} XRD",
            context.display_title("Tip:"),
            self.fee_summary.total_tipping_cost_in_xrd
        )?;
        write!(
            f,
            "\n├─ {} {} XRD",
            context.display_title("Network Storage:"),
            self.fee_summary.total_storage_cost_in_xrd
        )?;
        write!(
            f,
            "\n└─ {} {} XRD",
            context.display_title("Royalties:"),
            self.fee_summary.total_royalty_cost_in_xrd
        )?;

        if let TransactionResult::Commit(c) = &result {
            context.format_top_level_title_with_detail(f, "Logs", c.application_logs.len())?;
            for (i, (level, msg)) in c.application_logs.iter().enumerate() {
                let (level, msg) = context.display_log(level, msg);
                write!(
                    f,
                    "\n{} [{:5}] {}",
                    prefix!(i, c.application_logs),
                    level,
                    msg
                )?;
            }

            context.format_top_level_title_with_detail(f, "Events", c.application_events.len())?;
            for (i, (event_type_identifier, event_data)) in c.application_events.iter().enumerate()
            {
                display_event(
                    f,
                    prefix!(i, c.application_events),
                    event_type_identifier,
                    &c.system_structure,
                    event_data,
                    context,
                )?;
            }

            if context.display_state_updates {
                (&c.state_updates, &c.system_structure).contextual_format(f, context)?;
            }

            if let TransactionOutcome::Success(outputs) = &c.outcome {
                context.format_top_level_title_with_detail(f, "Outputs", outputs.len())?;
                for (i, output) in outputs.iter().enumerate() {
                    write!(
                        f,
                        "\n{} {}",
                        prefix!(i, outputs),
                        match output {
                            InstructionOutput::CallReturn(x) => IndexedScryptoValue::from_slice(&x)
                                .expect("Impossible case! Instruction output can't be decoded")
                                .to_string(ValueDisplayParameters::Schemaless {
                                    display_mode: DisplayMode::RustLike(RustLikeOptions::full()),
                                    print_mode: PrintMode::MultiLine {
                                        indent_size: 2,
                                        base_indent: 3,
                                        first_line_indent: 0
                                    },
                                    custom_context: scrypto_value_display_context,
                                    depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH
                                }),
                            InstructionOutput::None => "None".to_string(),
                        }
                    )?;
                }
            }

            let balance_changes = c.vault_balance_changes();
            context.format_top_level_title_with_detail(
                f,
                "Balance Changes",
                balance_changes.len(),
            )?;
            for (i, (vault_id, (resource, delta))) in balance_changes.iter().enumerate() {
                write!(
                    f,
                    // NB - we use ResAddr instead of Resource to protect people who read new resources as
                    //      `Resource: ` from the receipts (see eg resim.sh)
                    "\n{} Vault: {}\n   ResAddr: {}\n   Change: {}",
                    prefix!(i, balance_changes),
                    vault_id.display(address_display_context),
                    resource.display(address_display_context),
                    match delta {
                        BalanceChange::Fungible(d) => format!("{}", d),
                        BalanceChange::NonFungible { added, removed } => {
                            format!("+{:?}, -{:?}", added, removed)
                        }
                    }
                )?;
            }

            context.format_top_level_title_with_detail(
                f,
                "New Entities",
                c.new_package_addresses().len()
                    + c.new_component_addresses().len()
                    + c.new_resource_addresses().len(),
            )?;
            for (i, package_address) in c.new_package_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Package: {}",
                    prefix!(i, c.new_package_addresses()),
                    package_address.display(address_display_context)
                )?;
            }
            for (i, component_address) in c.new_component_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Component: {}",
                    prefix!(i, c.new_component_addresses()),
                    component_address.display(address_display_context)
                )?;
            }
            for (i, resource_address) in c.new_resource_addresses().iter().enumerate() {
                write!(
                    f,
                    "\n{} Resource: {}",
                    prefix!(i, c.new_resource_addresses()),
                    resource_address.display(address_display_context)
                )?;
            }
        }

        Ok(())
    }
}

impl<'a, 'b> ContextualDisplay<TransactionReceiptDisplayContext<'a>>
    for (&'b StateUpdates, &'b SystemStructure)
{
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &TransactionReceiptDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        let state_updates = self.0;
        let system_structure = self.1;
        context.format_top_level_title_with_detail(
            f,
            "State Updates",
            format!(
                "{} {}",
                state_updates.by_node.len(),
                if state_updates.by_node.len() == 1 {
                    "entity"
                } else {
                    "entities"
                },
            ),
        )?;
        for (i, (node_id, node_updates)) in state_updates.by_node.iter().enumerate() {
            let by_partition = match node_updates {
                NodeStateUpdates::Delta { by_partition } => by_partition,
            };
            write!(
                f,
                "\n{} {} across {} partitions",
                prefix!(i, state_updates.by_node),
                node_id.display(context.address_display_context()),
                by_partition.len(),
            )?;

            for (j, (partition_number, partition_updates)) in by_partition.iter().enumerate() {
                // NOTE: This could be improved by mapping partition numbers back to a system-focused name
                //       This would require either adding partition descriptions into SystemStructure, or
                //       having some inverse entity-type specific descriptors.
                match partition_updates {
                    PartitionStateUpdates::Delta { by_substate } => {
                        write!(
                            f,
                            "\n  {} Partition({}): {} {}",
                            prefix!(j, by_partition),
                            partition_number.0,
                            by_substate.len(),
                            if by_substate.len() == 1 {
                                "change"
                            } else {
                                "changes"
                            },
                        )?;
                        for (k, (substate_key, update)) in by_substate.iter().enumerate() {
                            display_substate_change(
                                f,
                                prefix!(k, by_substate),
                                system_structure,
                                context,
                                node_id,
                                partition_number,
                                substate_key,
                                update.as_ref(),
                            )?;
                        }
                    }
                    PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                        new_substate_values,
                    }) => {
                        write!(
                            f,
                            "\n {} Partition({}): RESET ({} new values)",
                            prefix!(j, by_partition),
                            partition_number.0,
                            new_substate_values.len()
                        )?;
                        for (k, (substate_key, value)) in new_substate_values.iter().enumerate() {
                            display_substate_change(
                                f,
                                prefix!(k, new_substate_values),
                                system_structure,
                                context,
                                node_id,
                                partition_number,
                                substate_key,
                                DatabaseUpdateRef::Set(value),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn display_substate_change<'a, F: fmt::Write>(
    f: &mut F,
    prefix: &str,
    system_structure: &SystemStructure,
    receipt_context: &TransactionReceiptDisplayContext<'a>,
    node_id: &NodeId,
    partition_number: &PartitionNumber,
    substate_key: &SubstateKey,
    change: DatabaseUpdateRef,
) -> Result<(), fmt::Error> {
    let substate_structure = system_structure
        .substate_system_structures
        .get(node_id)
        .unwrap()
        .get(partition_number)
        .unwrap()
        .get(substate_key)
        .unwrap();
    match change {
        DatabaseUpdateRef::Set(substate_value) => {
            write!(f, "\n    {prefix} Set: ")?;
            format_receipt_substate_key(f, substate_structure, receipt_context, substate_key)?;
            write!(f, "\n       Value: ")?;
            format_receipt_substate_value(f, substate_structure, receipt_context, substate_value)?;
        }
        DatabaseUpdateRef::Delete => {
            write!(f, "\n    {prefix} Delete: ")?;
            format_receipt_substate_key(f, substate_structure, receipt_context, substate_key)?;
        }
    }
    Ok(())
}

fn format_receipt_substate_key<'a, F: fmt::Write>(
    f: &mut F,
    substate_structure: &SubstateSystemStructure,
    receipt_context: &TransactionReceiptDisplayContext<'a>,
    substate_key: &SubstateKey,
) -> Result<(), fmt::Error> {
    let print_mode = PrintMode::SingleLine;
    match substate_structure {
        SubstateSystemStructure::SystemField(structure) => {
            write!(f, "{:?}", structure.field_kind)
        }
        SubstateSystemStructure::SystemSchema => {
            let key_contents = substate_key.for_map().unwrap();
            let hash: SchemaHash = scrypto_decode(&*key_contents).unwrap();
            write!(f, "SchemaHash({})", hash.0)
        }
        SubstateSystemStructure::KeyValueStoreEntry(structure) => {
            let value = scrypto_decode(substate_key.for_map().unwrap()).unwrap();
            format_scrypto_value_with_full_type_id(
                f,
                print_mode,
                value,
                receipt_context,
                &structure.key_full_type_id,
            )
        }
        SubstateSystemStructure::ObjectField(_) => {
            let key_contents = substate_key.for_field().unwrap();
            write!(f, "Field({})", key_contents)
        }
        SubstateSystemStructure::ObjectKeyValuePartitionEntry(structure) => {
            let value = scrypto_decode(substate_key.for_map().unwrap()).unwrap();
            let full_type_id = extract_object_type_id(&structure.key_schema);
            format_scrypto_value_with_full_type_id(
                f,
                print_mode,
                value,
                receipt_context,
                &full_type_id,
            )
        }
        SubstateSystemStructure::ObjectIndexPartitionEntry(structure) => {
            let value = scrypto_decode(substate_key.for_map().unwrap()).unwrap();
            let full_type_id = extract_object_type_id(&structure.key_schema);
            format_scrypto_value_with_full_type_id(
                f,
                print_mode,
                value,
                receipt_context,
                &full_type_id,
            )
        }
        SubstateSystemStructure::ObjectSortedIndexPartitionEntry(structure) => {
            let (sort_bytes, key_contents) = substate_key.for_sorted().unwrap();
            let value = scrypto_decode(key_contents).unwrap();
            let full_type_id = extract_object_type_id(&structure.key_schema);
            write!(f, "SortKey({}, ", u16::from_be_bytes(sort_bytes.clone()))?;
            format_scrypto_value_with_full_type_id(
                f,
                print_mode,
                value,
                receipt_context,
                &full_type_id,
            )?;
            write!(f, ")")
        }
    }
}

pub fn format_receipt_substate_value<'a, F: fmt::Write>(
    f: &mut F,
    substate_structure: &SubstateSystemStructure,
    receipt_context: &TransactionReceiptDisplayContext<'a>,
    substate_value: &[u8],
) -> Result<(), fmt::Error> {
    let print_mode = PrintMode::MultiLine {
        indent_size: 2,
        base_indent: 7,
        first_line_indent: 0,
    };
    if substate_value.len() > receipt_context.max_substate_length_to_display() {
        write!(
            f,
            "(Hidden as longer than {} bytes. Hash: {})",
            receipt_context.max_substate_length_to_display(),
            hash(substate_value)
        )
    } else {
        let (payload, full_type_id) = match substate_structure {
            SubstateSystemStructure::SystemField(structure) => {
                let single_type_schema = resolve_system_field_schema(structure.field_kind);
                let raw_value = scrypto_decode(substate_value).unwrap();
                return format_scrypto_value_with_schema(
                    f,
                    print_mode,
                    raw_value,
                    receipt_context,
                    &single_type_schema.schema,
                    single_type_schema.type_id,
                );
            }
            SubstateSystemStructure::SystemSchema => {
                let single_type_schema = resolve_system_schema_schema();
                let raw_value = scrypto_decode(substate_value).unwrap();
                return format_scrypto_value_with_schema(
                    f,
                    print_mode,
                    raw_value,
                    receipt_context,
                    &single_type_schema.schema,
                    single_type_schema.type_id,
                );
            }
            SubstateSystemStructure::KeyValueStoreEntry(structure) => {
                let payload =
                    scrypto_decode::<KeyValueEntrySubstate<ScryptoRawValue>>(substate_value)
                        .unwrap();
                (payload.into_value(), structure.value_full_type_id.clone())
            }
            SubstateSystemStructure::ObjectField(structure) => {
                let payload =
                    scrypto_decode::<FieldSubstate<ScryptoRawValue>>(substate_value).unwrap();
                write_lock_status(f, payload.lock_status())?;
                (
                    Some(payload.into_payload()),
                    extract_object_type_id(&structure.value_schema),
                )
            }
            SubstateSystemStructure::ObjectKeyValuePartitionEntry(structure) => {
                let payload =
                    scrypto_decode::<KeyValueEntrySubstate<ScryptoRawValue>>(substate_value)
                        .unwrap();
                write_lock_status(f, payload.lock_status())?;
                (
                    payload.into_value(),
                    extract_object_type_id(&structure.value_schema),
                )
            }
            SubstateSystemStructure::ObjectIndexPartitionEntry(structure) => {
                let payload =
                    scrypto_decode::<IndexEntrySubstate<ScryptoRawValue>>(substate_value).unwrap();
                (
                    Some(payload.into_value()),
                    extract_object_type_id(&structure.value_schema),
                )
            }
            SubstateSystemStructure::ObjectSortedIndexPartitionEntry(structure) => {
                let payload =
                    scrypto_decode::<SortedIndexEntrySubstate<ScryptoRawValue>>(substate_value)
                        .unwrap();
                (
                    Some(payload.into_value()),
                    extract_object_type_id(&structure.value_schema),
                )
            }
        };
        match payload {
            Some(payload) => format_scrypto_value_with_full_type_id(
                f,
                print_mode,
                payload,
                receipt_context,
                &full_type_id,
            ),
            None => write!(f, "EMPTY"),
        }
    }
}

fn write_lock_status<F: fmt::Write>(f: &mut F, lock_status: LockStatus) -> Result<(), fmt::Error> {
    match lock_status {
        LockStatus::Unlocked => write!(f, "UNLOCKED "),
        LockStatus::Locked => write!(f, "LOCKED "),
    }
}

fn extract_object_type_id(structure: &ObjectSubstateTypeReference) -> FullyScopedTypeId<NodeId> {
    match structure {
        ObjectSubstateTypeReference::Package(r) => r.full_type_id.clone().into_general(),
        ObjectSubstateTypeReference::ObjectInstance(r) => r.resolved_full_type_id.clone(),
    }
}

fn display_event<'a, F: fmt::Write>(
    f: &mut F,
    prefix: &str,
    event_type_identifier: &EventTypeIdentifier,
    system_structure: &SystemStructure,
    event_data: &[u8],
    receipt_context: &TransactionReceiptDisplayContext<'a>,
) -> Result<(), fmt::Error> {
    let event_system_structure = system_structure
        .event_system_structures
        .get(event_type_identifier)
        .expect("Expected event to appear in the system structure");

    let full_type_id = event_system_structure.package_type_reference.full_type_id;
    let schema_lookup = receipt_context.lookup_schema(&full_type_id);
    let emitter = &event_type_identifier.0;
    let print_mode = PrintMode::MultiLine {
        indent_size: 2,
        base_indent: 3,
        first_line_indent: 0,
    };
    let raw_value = scrypto_decode::<ScryptoRawValue>(event_data).unwrap();
    if let Some(_) = schema_lookup {
        write!(
            f,
            "\n{} Emitter: {}\n   Event: ",
            prefix,
            emitter.display(receipt_context.address_display_context()),
        )?;
        format_scrypto_value_with_full_type_id(
            f,
            print_mode,
            raw_value,
            receipt_context,
            &full_type_id,
        )?;
    } else {
        write!(
            f,
            "\n{} Emitter: {}\n   Name: {:?}\n   Data: ",
            prefix,
            emitter.display(receipt_context.address_display_context()),
            event_type_identifier.1,
        )?;
        format_scrypto_value_with_full_type_id(
            f,
            print_mode,
            raw_value,
            receipt_context,
            &full_type_id,
        )?;
    }
    Ok(())
}

fn format_scrypto_value_with_full_type_id<'a, F: fmt::Write, T: AsRef<NodeId>>(
    f: &mut F,
    print_mode: PrintMode,
    raw_value: ScryptoRawValue<'_>,
    receipt_context: &TransactionReceiptDisplayContext<'a>,
    full_type_id: &FullyScopedTypeId<T>,
) -> Result<(), fmt::Error> {
    let schema_lookup = receipt_context.lookup_schema(full_type_id);
    match schema_lookup {
        Some((local_type_id, schema)) => format_scrypto_value_with_schema(
            f,
            print_mode,
            raw_value,
            receipt_context,
            &schema,
            local_type_id,
        ),
        None => {
            let display_parameters: ValueDisplayParameters<'_, '_, ScryptoCustomExtension> =
                ValueDisplayParameters::Schemaless {
                    display_mode: DisplayMode::RustLike(RustLikeOptions::full()),
                    print_mode,
                    custom_context: receipt_context.display_context(),
                    depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
                };
            write!(f, "{}", raw_value.display(display_parameters))
        }
    }
}

fn format_scrypto_value_with_schema<'a, F: fmt::Write>(
    f: &mut F,
    print_mode: PrintMode,
    raw_value: ScryptoRawValue<'_>,
    receipt_context: &TransactionReceiptDisplayContext<'a>,
    schema: &VersionedScryptoSchema,
    local_type_id: LocalTypeId,
) -> Result<(), fmt::Error> {
    let display_parameters = ValueDisplayParameters::Annotated {
        display_mode: DisplayMode::RustLike(RustLikeOptions::full()),
        print_mode,
        custom_context: receipt_context.display_context(),
        schema: schema.v1(),
        type_id: local_type_id,
        depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
    };
    write!(f, "{}", raw_value.display(display_parameters))
}

impl From<FeeReserveFinalizationSummary> for TransactionFeeSummary {
    fn from(value: FeeReserveFinalizationSummary) -> Self {
        Self {
            total_execution_cost_units_consumed: value.total_execution_cost_units_consumed,
            total_finalization_cost_units_consumed: value.total_finalization_cost_units_consumed,
            total_execution_cost_in_xrd: value.total_execution_cost_in_xrd,
            total_finalization_cost_in_xrd: value.total_finalization_cost_in_xrd,
            total_tipping_cost_in_xrd: value.total_tipping_cost_in_xrd,
            total_storage_cost_in_xrd: value.total_storage_cost_in_xrd,
            total_royalty_cost_in_xrd: value.total_royalty_cost_in_xrd,
        }
    }
}

impl TransactionFeeSummary {
    pub fn total_cost(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            .checked_add(self.total_finalization_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_tipping_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_storage_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_royalty_cost_in_xrd)
            .unwrap()
    }

    pub fn network_fees(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            .checked_add(self.total_finalization_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_storage_cost_in_xrd)
            .unwrap()
    }

    //===================
    // For testing only
    //===================

    pub fn expected_reward_if_single_validator(&self) -> Decimal {
        self.expected_reward_as_proposer_if_single_validator()
            .checked_add(self.expected_reward_as_active_validator_if_single_validator())
            .unwrap()
    }

    pub fn expected_reward_as_proposer_if_single_validator(&self) -> Decimal {
        let one_percent = Decimal::ONE_HUNDREDTH;

        one_percent
            .checked_mul(TIPS_PROPOSER_SHARE_PERCENTAGE)
            .unwrap()
            .checked_mul(self.total_tipping_cost_in_xrd)
            .unwrap()
            .checked_add(
                one_percent
                    .checked_mul(NETWORK_FEES_PROPOSER_SHARE_PERCENTAGE)
                    .unwrap()
                    .checked_mul(
                        self.total_execution_cost_in_xrd
                            .checked_add(self.total_finalization_cost_in_xrd)
                            .unwrap()
                            .checked_add(self.total_storage_cost_in_xrd)
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
    }

    pub fn expected_reward_as_active_validator_if_single_validator(&self) -> Decimal {
        let one_percent = Decimal::ONE_HUNDREDTH;

        one_percent
            .checked_mul(TIPS_VALIDATOR_SET_SHARE_PERCENTAGE)
            .unwrap()
            .checked_mul(self.total_tipping_cost_in_xrd)
            .unwrap()
            .checked_add(
                one_percent
                    .checked_mul(NETWORK_FEES_VALIDATOR_SET_SHARE_PERCENTAGE)
                    .unwrap()
                    .checked_mul(
                        self.total_execution_cost_in_xrd
                            .checked_add(self.total_finalization_cost_in_xrd)
                            .unwrap()
                            .checked_add(self.total_storage_cost_in_xrd)
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
    }
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub enum FlamegraphError {
    IOError(std::io::Error),
    CreationError,
    DetailedCostBreakdownNotAvailable,
}

#[cfg(test)]
mod tests {
    use radix_transactions::model::TransactionCostingParametersReceiptV2;

    use super::*;

    define_versioned!(
        #[derive(ScryptoSbor)]
        VersionedLocalTransactionExecution(LocalTransactionExecutionVersions) {
            previous_versions: [
                1 => LocalTransactionExecutionV1: { updates_to: 2 },
            ],
            latest_version: {
                2 => LocalTransactionExecution = LocalTransactionExecutionV2,
            },
        },
        outer_attributes: [
            // This is an effective copy of the contents of the local transaction execution store in the node.
            // This needs to be decodable!
            // By all means introduce _new versions_, with conversions between them,
            // and we can do the same in the node.
            // But this schema can't change, else we won't be able to decode existing executions in the node.
            // NOTE: This is just copied here to catch issues / changes earlier; an identical test exists in the node.
            #[derive(ScryptoSborAssertion)]
            #[sbor_assert(
                backwards_compatible(
                    bottlenose = "FILE:node_versioned_local_transaction_execution_bottlenose.bin",
                    cuttlefish = "FILE:node_versioned_local_transaction_execution_cuttlefish.bin"
                ),
                settings(allow_name_changes)
            )]
        ],
    );

    #[derive(ScryptoSbor)]
    struct LocalTransactionExecutionV1 {
        outcome: Result<(), ScryptoOwnedRawValue>,
        fee_summary: TransactionFeeSummary,
        fee_source: FeeSource,
        fee_destination: FeeDestination,
        engine_costing_parameters: CostingParameters,
        transaction_costing_parameters: TransactionCostingParametersReceiptV1,
        application_logs: Vec<(Level, String)>,
        state_update_summary: StateUpdateSummary,
        global_balance_summary: IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
        substates_system_structure: Vec<SubstateSystemStructure>,
        events_system_structure: IndexMap<EventTypeIdentifier, EventSystemStructure>,
        next_epoch: Option<EpochChangeEvent>,
    }

    #[derive(ScryptoSbor)]
    struct LocalTransactionExecutionV2 {
        outcome: Result<(), PersistableRuntimeError>,
        fee_summary: TransactionFeeSummary,
        fee_source: FeeSource,
        fee_destination: FeeDestination,
        engine_costing_parameters: CostingParameters,
        transaction_costing_parameters: TransactionCostingParametersReceiptV2,
        application_logs: Vec<(Level, String)>,
        state_update_summary: StateUpdateSummary,
        global_balance_summary: IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
        substates_system_structure: Vec<SubstateSystemStructure>,
        events_system_structure: IndexMap<EventTypeIdentifier, EventSystemStructure>,
        next_epoch: Option<EpochChangeEvent>,
    }

    impl From<LocalTransactionExecutionV1> for LocalTransactionExecutionV2 {
        fn from(value: LocalTransactionExecutionV1) -> Self {
            Self {
                outcome: value.outcome.map_err(|err| PersistableRuntimeError {
                    schema_index: 0,
                    encoded_error: err,
                }),
                fee_summary: value.fee_summary,
                fee_source: value.fee_source,
                fee_destination: value.fee_destination,
                engine_costing_parameters: value.engine_costing_parameters,
                transaction_costing_parameters: value.transaction_costing_parameters.into(),
                application_logs: value.application_logs,
                state_update_summary: value.state_update_summary,
                global_balance_summary: value.global_balance_summary,
                substates_system_structure: value.substates_system_structure,
                events_system_structure: value.events_system_structure,
                next_epoch: value.next_epoch,
            }
        }
    }
}
