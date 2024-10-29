use super::FeeTable;
use crate::internal_prelude::*;
use crate::kernel::kernel_callback_api::{
    CheckReferenceEvent, CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent,
    MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::system::actor::Actor;
use crate::system::system_modules::transaction_runtime::Event;
use crate::track::interface::StoreCommit;

#[derive(Debug, IntoStaticStr)]
pub enum ExecutionCostingEntry<'a> {
    /* verify signature */
    VerifyTxSignatures {
        num_signatures: usize,
    },
    ValidateTxPayload {
        size: usize,
    },
    CheckReference {
        event: &'a CheckReferenceEvent<'a>,
    },
    CheckIntentValidity,
    CheckTimestamp,

    /* run code */
    RunNativeCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        input_size: usize,
    },
    RunWasmCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        wasm_execution_units: u32,
    },
    PrepareWasmCode {
        size: usize,
    },

    /* invoke */
    BeforeInvoke {
        actor: &'a Actor,
        input_size: usize,
    },
    AfterInvoke {
        output_size: usize,
    },

    /* node */
    AllocateNodeId,
    CreateNode {
        event: &'a CreateNodeEvent<'a>,
    },
    DropNode {
        event: &'a DropNodeEvent<'a>,
    },
    PinNode {
        node_id: &'a NodeId,
    },
    MoveModule {
        event: &'a MoveModuleEvent<'a>,
    },

    /* Substate */
    OpenSubstate {
        event: &'a OpenSubstateEvent<'a>,
    },
    ReadSubstate {
        event: &'a ReadSubstateEvent<'a>,
    },
    WriteSubstate {
        event: &'a WriteSubstateEvent<'a>,
    },
    CloseSubstate {
        event: &'a CloseSubstateEvent,
    },
    MarkSubstateAsTransient {
        node_id: &'a NodeId,
        partition_number: &'a PartitionNumber,
        substate_key: &'a SubstateKey,
    },

    /* unstable node apis */
    SetSubstate {
        event: &'a SetSubstateEvent<'a>,
    },
    RemoveSubstate {
        event: &'a RemoveSubstateEvent<'a>,
    },
    ScanKeys {
        event: &'a ScanKeysEvent<'a>,
    },
    ScanSortedSubstates {
        event: &'a ScanSortedSubstatesEvent<'a>,
    },
    DrainSubstates {
        event: &'a DrainSubstatesEvent<'a>,
    },

    /* stack api */
    GetStackId,
    GetOwnedNodes,
    SwitchStack,
    SendToStack {
        data_len: usize,
    },
    SetCallFrameData {
        data_len: usize,
    },

    /* system */
    LockFee,
    QueryFeeReserve,
    QueryCostingModule,
    QueryActor,
    QueryTransactionHash,
    GenerateRuid,
    EmitEvent {
        size: usize,
    },
    EmitLog {
        size: usize,
    },
    EncodeBech32Address,
    Panic {
        size: usize,
    },

    /* crypto utils */
    Bls12381V1Verify {
        size: usize,
    },
    Bls12381V1AggregateVerify {
        sizes: &'a [usize],
    },
    Bls12381V1FastAggregateVerify {
        size: usize,
        keys_cnt: usize,
    },
    Bls12381G2SignatureAggregate {
        signatures_cnt: usize,
    },
    Keccak256Hash {
        size: usize,
    },
    Blake2b256Hash {
        size: usize,
    },
    Ed25519Verify {
        size: usize,
    },
    Secp256k1EcdsaVerify,
    Secp256k1EcdsaVerifyAndKeyRecover,
}

#[derive(Debug, IntoStaticStr)]
pub enum FinalizationCostingEntry<'a> {
    CommitStateUpdates { store_commit: &'a StoreCommit },
    CommitEvents { events: &'a Vec<Event> },
    CommitLogs { logs: &'a Vec<(Level, String)> },
    CommitIntentStatus { num_of_intent_statuses: usize },
}

impl<'a> ExecutionCostingEntry<'a> {
    pub fn to_execution_cost_units(&self, ft: &FeeTable) -> u32 {
        match self {
            ExecutionCostingEntry::VerifyTxSignatures {
                num_signatures: num_of_signatures,
            } => ft.verify_tx_signatures_cost(*num_of_signatures),
            ExecutionCostingEntry::ValidateTxPayload { size } => ft.validate_tx_payload_cost(*size),
            ExecutionCostingEntry::CheckReference { event } => ft.check_reference(event),
            ExecutionCostingEntry::CheckIntentValidity => ft.check_intent_validity(),
            ExecutionCostingEntry::CheckTimestamp => ft.check_timestamp(),
            ExecutionCostingEntry::RunNativeCode {
                package_address,
                export_name,
                input_size,
            } => ft.run_native_code_cost(package_address, export_name, input_size),
            ExecutionCostingEntry::RunWasmCode {
                package_address,
                export_name,
                wasm_execution_units,
            } => ft.run_wasm_code_cost(package_address, export_name, *wasm_execution_units),
            ExecutionCostingEntry::PrepareWasmCode { size } => ft.instantiate_wasm_code_cost(*size),
            ExecutionCostingEntry::BeforeInvoke { actor, input_size } => {
                ft.before_invoke_cost(actor, *input_size)
            }
            ExecutionCostingEntry::AfterInvoke { output_size } => {
                ft.after_invoke_cost(*output_size)
            }
            ExecutionCostingEntry::AllocateNodeId => ft.allocate_node_id_cost(),
            ExecutionCostingEntry::CreateNode { event } => ft.create_node_cost(event),
            ExecutionCostingEntry::DropNode { event } => ft.drop_node_cost(event),
            ExecutionCostingEntry::PinNode { node_id } => ft.pin_node_cost(node_id),
            ExecutionCostingEntry::MoveModule { event } => ft.move_module_cost(event),
            ExecutionCostingEntry::OpenSubstate { event } => ft.open_substate_cost(event),
            ExecutionCostingEntry::ReadSubstate { event } => ft.read_substate_cost(event),
            ExecutionCostingEntry::WriteSubstate { event } => ft.write_substate_cost(event),
            ExecutionCostingEntry::CloseSubstate { event } => ft.close_substate_cost(event),
            ExecutionCostingEntry::SetSubstate { event } => ft.set_substate_cost(event),
            ExecutionCostingEntry::RemoveSubstate { event } => ft.remove_substate_cost(event),
            ExecutionCostingEntry::MarkSubstateAsTransient {
                node_id,
                partition_number,
                substate_key,
            } => ft.mark_substate_as_transient_cost(node_id, partition_number, substate_key),
            ExecutionCostingEntry::ScanKeys { event } => ft.scan_keys_cost(event),
            ExecutionCostingEntry::DrainSubstates { event } => ft.drain_substates_cost(event),
            ExecutionCostingEntry::ScanSortedSubstates { event } => {
                ft.scan_sorted_substates_cost(event)
            }
            ExecutionCostingEntry::GetStackId => ft.get_stack_id(),
            ExecutionCostingEntry::GetOwnedNodes => ft.get_owned_nodes(),
            ExecutionCostingEntry::SwitchStack => ft.switch_stack(),
            ExecutionCostingEntry::SendToStack { data_len } => ft.send_to_stack(*data_len),
            ExecutionCostingEntry::SetCallFrameData { data_len } => {
                ft.set_call_frame_data(*data_len)
            }
            ExecutionCostingEntry::LockFee => ft.lock_fee_cost(),
            ExecutionCostingEntry::QueryFeeReserve => ft.query_fee_reserve_cost(),
            ExecutionCostingEntry::QueryCostingModule => ft.query_costing_module(),
            ExecutionCostingEntry::QueryActor => ft.query_actor_cost(),
            ExecutionCostingEntry::QueryTransactionHash => ft.query_transaction_hash_cost(),
            ExecutionCostingEntry::GenerateRuid => ft.generate_ruid_cost(),
            ExecutionCostingEntry::EmitEvent { size } => ft.emit_event_cost(*size),
            ExecutionCostingEntry::EmitLog { size } => ft.emit_log_cost(*size),
            ExecutionCostingEntry::EncodeBech32Address => ft.encode_bech32_address_cost(),
            ExecutionCostingEntry::Panic { size } => ft.panic_cost(*size),
            ExecutionCostingEntry::Bls12381V1Verify { size } => ft.bls12381_v1_verify_cost(*size),
            ExecutionCostingEntry::Bls12381V1AggregateVerify { sizes } => {
                ft.bls12381_v1_aggregate_verify_cost(sizes)
            }
            ExecutionCostingEntry::Bls12381V1FastAggregateVerify { size, keys_cnt } => {
                ft.bls12381_v1_fast_aggregate_verify_cost(*size, *keys_cnt)
            }
            ExecutionCostingEntry::Bls12381G2SignatureAggregate { signatures_cnt } => {
                ft.bls12381_g2_signature_aggregate_cost(*signatures_cnt)
            }
            ExecutionCostingEntry::Keccak256Hash { size } => ft.keccak256_hash_cost(*size),
            ExecutionCostingEntry::Blake2b256Hash { size } => ft.blake2b256_hash_cost(*size),
            ExecutionCostingEntry::Ed25519Verify { size } => ft.ed25519_verify_cost(*size),
            ExecutionCostingEntry::Secp256k1EcdsaVerify => ft.secp256k1_ecdsa_verify_cost(),
            ExecutionCostingEntry::Secp256k1EcdsaVerifyAndKeyRecover => {
                ft.secp256k1_ecdsa_verify_and_key_recover_cost()
            }
        }
    }
}

impl<'a> FinalizationCostingEntry<'a> {
    pub fn to_finalization_cost_units(&self, ft: &FeeTable) -> u32 {
        match self {
            FinalizationCostingEntry::CommitStateUpdates { store_commit } => {
                ft.commit_state_updates_cost(store_commit)
            }
            FinalizationCostingEntry::CommitEvents { events } => ft.commit_events_cost(events),
            FinalizationCostingEntry::CommitLogs { logs } => ft.commit_logs_cost(logs),

            FinalizationCostingEntry::CommitIntentStatus {
                num_of_intent_statuses,
            } => ft.commit_intent_status(*num_of_intent_statuses),
        }
    }
}

impl<'a> ExecutionCostingEntry<'a> {
    pub fn to_trace_key(&self) -> String {
        match self {
            ExecutionCostingEntry::RunNativeCode { export_name, .. } => {
                format!("RunNativeCode::{}", export_name)
            }
            ExecutionCostingEntry::RunWasmCode { export_name, .. } => {
                format!("RunWasmCode::{}", export_name)
            }
            ExecutionCostingEntry::OpenSubstate { event, .. } => {
                let node_id = match event {
                    OpenSubstateEvent::Start { node_id, .. } => **node_id,
                    OpenSubstateEvent::IOAccess(access) => access.node_id(),
                    OpenSubstateEvent::End { node_id, .. } => **node_id,
                };

                format!(
                    "OpenSubstate::{}",
                    node_id.entity_type().map(|x| x.into()).unwrap_or("?")
                )
            }
            x => Into::<&'static str>::into(x).to_string(),
        }
    }
}

impl<'a> FinalizationCostingEntry<'a> {
    pub fn to_trace_key(&self) -> String {
        match self {
            FinalizationCostingEntry::CommitStateUpdates { store_commit } => {
                format!(
                    "CommitStateUpdates::{}",
                    store_commit
                        .node_id()
                        .entity_type()
                        .map(|x| x.into())
                        .unwrap_or("?")
                )
            }
            x => Into::<&'static str>::into(x).to_string(),
        }
    }
}

/// A module containing various models that do not use references and use owned objects instead.
/// Keep in mind that using references is more efficient and that this is used in applications that
/// are not performance critical.
#[allow(clippy::large_enum_variant)]
pub mod owned {
    use super::*;
    use crate::kernel::substate_io::*;
    use crate::track::*;

    /// An owned model equivalent of [`ExecutionCostingEntry`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum ExecutionCostingEntryOwned {
        /* verify signature */
        VerifyTxSignatures {
            num_signatures: usize,
        },
        ValidateTxPayload {
            size: usize,
        },
        CheckReference {
            event: CheckReferenceEventOwned,
        },
        CheckIntentValidity,
        CheckTimestamp,

        /* run code */
        RunNativeCode {
            package_address: PackageAddress,
            export_name: String,
            input_size: usize,
        },
        RunWasmCode {
            package_address: PackageAddress,
            export_name: String,
            wasm_execution_units: u32,
        },
        PrepareWasmCode {
            size: usize,
        },

        /* invoke */
        BeforeInvoke {
            actor: Actor,
            input_size: usize,
        },
        AfterInvoke {
            output_size: usize,
        },

        /* node */
        AllocateNodeId,
        CreateNode {
            event: CreateNodeEventOwned,
        },
        DropNode {
            event: DropNodeEventOwned,
        },
        PinNode {
            node_id: NodeId,
        },
        MoveModule {
            event: MoveModuleEventOwned,
        },

        /* Substate */
        OpenSubstate {
            event: OpenSubstateEventOwned,
        },
        ReadSubstate {
            event: ReadSubstateEventOwned,
        },
        WriteSubstate {
            event: WriteSubstateEventOwned,
        },
        CloseSubstate {
            event: CloseSubstateEventOwned,
        },
        MarkSubstateAsTransient {
            node_id: NodeId,
            partition_number: PartitionNumber,
            substate_key: SubstateKey,
        },

        /* unstable node apis */
        SetSubstate {
            event: SetSubstateEventOwned,
        },
        RemoveSubstate {
            event: RemoveSubstateEventOwned,
        },
        ScanKeys {
            event: ScanKeysEventOwned,
        },
        ScanSortedSubstates {
            event: ScanSortedSubstatesEventOwned,
        },
        DrainSubstates {
            event: DrainSubstatesEventOwned,
        },

        GetStackId,
        GetOwnedNodes,
        SwitchStack,
        SendToStack {
            data_len: usize,
        },
        SetCallFrameData {
            data_len: usize,
        },

        /* system */
        LockFee,
        QueryFeeReserve,
        QueryCostingModule,
        QueryActor,
        QueryTransactionHash,
        GenerateRuid,
        EmitEvent {
            size: usize,
        },
        EmitLog {
            size: usize,
        },
        EncodeBech32Address,
        Panic {
            size: usize,
        },

        /* crypto utils */
        Bls12381V1Verify {
            size: usize,
        },
        Bls12381V1AggregateVerify {
            sizes: Vec<usize>,
        },
        Bls12381V1FastAggregateVerify {
            size: usize,
            keys_cnt: usize,
        },
        Bls12381G2SignatureAggregate {
            signatures_cnt: usize,
        },
        Keccak256Hash {
            size: usize,
        },
        Blake2b256Hash {
            size: usize,
        },
        Ed25519Verify {
            size: usize,
        },
        Secp256k1EcdsaVerify,
        Secp256k1EcdsaVerifyAndKeyRecover,
    }

    /// An owned model equivalent of [`CreateNodeEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum CreateNodeEventOwned {
        Start(
            NodeId,
            BTreeMap<PartitionNumber, BTreeMap<SubstateKey, (ScryptoValue,)>>,
        ),
        IOAccess(IOAccess),
        End(NodeId),
    }

    /// An owned model equivalent of [`DropNodeEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum DropNodeEventOwned {
        Start(NodeId),
        IOAccess(IOAccess),
        End(
            NodeId,
            BTreeMap<PartitionNumber, BTreeMap<SubstateKey, (ScryptoValue,)>>,
        ),
    }

    /// An owned model equivalent of [`RefCheckEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum CheckReferenceEventOwned {
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`MoveModuleEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum MoveModuleEventOwned {
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`OpenSubstateEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum OpenSubstateEventOwned {
        Start {
            node_id: NodeId,
            partition_num: PartitionNumber,
            substate_key: SubstateKey,
            flags: LockFlags,
        },
        IOAccess(IOAccess),
        End {
            handle: SubstateHandle,
            node_id: NodeId,
            size: usize,
        },
    }

    /// An owned model equivalent of [`ReadSubstateEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum ReadSubstateEventOwned {
        OnRead {
            handle: SubstateHandle,
            value: (ScryptoValue,),
            device: SubstateDevice,
        },
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`WriteSubstateEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum WriteSubstateEventOwned {
        Start {
            handle: SubstateHandle,
            value: (ScryptoValue,),
        },
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`CloseSubstateEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum CloseSubstateEventOwned {
        Start(SubstateHandle),
    }

    /// An owned model equivalent of [`SetSubstateEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum SetSubstateEventOwned {
        Start(NodeId, PartitionNumber, SubstateKey, (ScryptoValue,)),
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`RemoveSubstateEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum RemoveSubstateEventOwned {
        Start(NodeId, PartitionNumber, SubstateKey),
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`ScanKeysEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum ScanKeysEventOwned {
        Start,
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`DrainSubstatesEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum DrainSubstatesEventOwned {
        Start(u32),
        IOAccess(IOAccess),
    }

    /// An owned model equivalent of [`ScanSortedSubstatesEvent`].
    #[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
    pub enum ScanSortedSubstatesEventOwned {
        Start,
        IOAccess(IOAccess),
    }

    impl<'a> From<ExecutionCostingEntry<'a>> for ExecutionCostingEntryOwned {
        fn from(value: ExecutionCostingEntry<'a>) -> Self {
            match value {
                ExecutionCostingEntry::VerifyTxSignatures { num_signatures } => {
                    Self::VerifyTxSignatures { num_signatures }
                }
                ExecutionCostingEntry::ValidateTxPayload { size } => {
                    Self::ValidateTxPayload { size }
                }
                ExecutionCostingEntry::CheckReference { event } => Self::CheckReference {
                    event: event.into(),
                },
                ExecutionCostingEntry::CheckIntentValidity => Self::CheckIntentValidity,
                ExecutionCostingEntry::CheckTimestamp => Self::CheckTimestamp,
                ExecutionCostingEntry::RunNativeCode {
                    package_address,
                    export_name,
                    input_size,
                } => Self::RunNativeCode {
                    package_address: *package_address,
                    export_name: export_name.to_owned(),
                    input_size,
                },
                ExecutionCostingEntry::RunWasmCode {
                    package_address,
                    export_name,
                    wasm_execution_units,
                } => Self::RunWasmCode {
                    package_address: *package_address,
                    export_name: export_name.to_owned(),
                    wasm_execution_units,
                },
                ExecutionCostingEntry::PrepareWasmCode { size } => Self::PrepareWasmCode { size },
                ExecutionCostingEntry::BeforeInvoke { actor, input_size } => Self::BeforeInvoke {
                    actor: actor.clone(),
                    input_size,
                },
                ExecutionCostingEntry::AfterInvoke { output_size } => {
                    Self::AfterInvoke { output_size }
                }
                ExecutionCostingEntry::AllocateNodeId => Self::AllocateNodeId,
                ExecutionCostingEntry::CreateNode { event } => Self::CreateNode {
                    event: event.into(),
                },
                ExecutionCostingEntry::DropNode { event } => Self::DropNode {
                    event: event.into(),
                },
                ExecutionCostingEntry::PinNode { node_id } => Self::PinNode { node_id: *node_id },
                ExecutionCostingEntry::MoveModule { event } => Self::MoveModule {
                    event: event.into(),
                },
                ExecutionCostingEntry::OpenSubstate { event } => Self::OpenSubstate {
                    event: event.into(),
                },
                ExecutionCostingEntry::ReadSubstate { event } => Self::ReadSubstate {
                    event: event.into(),
                },
                ExecutionCostingEntry::WriteSubstate { event } => Self::WriteSubstate {
                    event: event.into(),
                },
                ExecutionCostingEntry::CloseSubstate { event } => Self::CloseSubstate {
                    event: event.into(),
                },
                ExecutionCostingEntry::MarkSubstateAsTransient {
                    node_id,
                    partition_number,
                    substate_key,
                } => Self::MarkSubstateAsTransient {
                    node_id: *node_id,
                    partition_number: *partition_number,
                    substate_key: substate_key.clone(),
                },
                ExecutionCostingEntry::SetSubstate { event } => Self::SetSubstate {
                    event: event.into(),
                },
                ExecutionCostingEntry::RemoveSubstate { event } => Self::RemoveSubstate {
                    event: event.into(),
                },
                ExecutionCostingEntry::ScanKeys { event } => Self::ScanKeys {
                    event: event.into(),
                },
                ExecutionCostingEntry::ScanSortedSubstates { event } => Self::ScanSortedSubstates {
                    event: event.into(),
                },
                ExecutionCostingEntry::DrainSubstates { event } => Self::DrainSubstates {
                    event: event.into(),
                },
                ExecutionCostingEntry::GetStackId => Self::GetStackId,
                ExecutionCostingEntry::GetOwnedNodes => Self::GetOwnedNodes,
                ExecutionCostingEntry::SwitchStack => Self::SwitchStack,
                ExecutionCostingEntry::SendToStack { data_len } => Self::SendToStack { data_len },
                ExecutionCostingEntry::SetCallFrameData { data_len } => {
                    Self::SetCallFrameData { data_len }
                }
                ExecutionCostingEntry::LockFee => Self::LockFee,
                ExecutionCostingEntry::QueryFeeReserve => Self::QueryFeeReserve,
                ExecutionCostingEntry::QueryCostingModule => Self::QueryCostingModule,
                ExecutionCostingEntry::QueryActor => Self::QueryActor,
                ExecutionCostingEntry::QueryTransactionHash => Self::QueryTransactionHash,
                ExecutionCostingEntry::GenerateRuid => Self::GenerateRuid,
                ExecutionCostingEntry::EmitEvent { size } => Self::EmitEvent { size },
                ExecutionCostingEntry::EmitLog { size } => Self::EmitLog { size },
                ExecutionCostingEntry::EncodeBech32Address => Self::EncodeBech32Address,
                ExecutionCostingEntry::Panic { size } => Self::Panic { size },
                ExecutionCostingEntry::Bls12381V1Verify { size } => Self::Bls12381V1Verify { size },
                ExecutionCostingEntry::Bls12381V1AggregateVerify { sizes } => {
                    Self::Bls12381V1AggregateVerify {
                        sizes: sizes.to_vec(),
                    }
                }
                ExecutionCostingEntry::Bls12381V1FastAggregateVerify { size, keys_cnt } => {
                    Self::Bls12381V1FastAggregateVerify { size, keys_cnt }
                }
                ExecutionCostingEntry::Bls12381G2SignatureAggregate { signatures_cnt } => {
                    Self::Bls12381G2SignatureAggregate { signatures_cnt }
                }
                ExecutionCostingEntry::Keccak256Hash { size } => Self::Keccak256Hash { size },
                ExecutionCostingEntry::Blake2b256Hash { size } => Self::Blake2b256Hash { size },
                ExecutionCostingEntry::Ed25519Verify { size } => Self::Ed25519Verify { size },
                ExecutionCostingEntry::Secp256k1EcdsaVerify => Self::Secp256k1EcdsaVerify,
                ExecutionCostingEntry::Secp256k1EcdsaVerifyAndKeyRecover => {
                    Self::Secp256k1EcdsaVerifyAndKeyRecover
                }
            }
        }
    }

    impl<'a> From<&'a CreateNodeEvent<'a>> for CreateNodeEventOwned {
        fn from(value: &'a CreateNodeEvent<'a>) -> Self {
            match value {
                CreateNodeEvent::Start(item1, item2) => Self::Start(
                    **item1,
                    item2
                        .iter()
                        .map(|(key, value)| {
                            (
                                key.clone(),
                                value
                                    .iter()
                                    .map(|(key, value)| {
                                        (key.clone(), (value.as_scrypto_value().to_owned(),))
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                ),
                CreateNodeEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
                CreateNodeEvent::End(item) => Self::End(**item),
            }
        }
    }

    impl<'a> From<&'a DropNodeEvent<'a>> for DropNodeEventOwned {
        fn from(value: &'a DropNodeEvent<'a>) -> Self {
            match value {
                DropNodeEvent::Start(item) => Self::Start(**item),
                DropNodeEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
                DropNodeEvent::End(item1, item2) => Self::End(
                    **item1,
                    item2
                        .iter()
                        .map(|(key, value)| {
                            (
                                key.clone(),
                                value
                                    .iter()
                                    .map(|(key, value)| {
                                        (key.clone(), (value.as_scrypto_value().to_owned(),))
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                ),
            }
        }
    }

    impl<'a> From<&'a CheckReferenceEvent<'a>> for CheckReferenceEventOwned {
        fn from(value: &'a CheckReferenceEvent<'a>) -> Self {
            match value {
                CheckReferenceEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl<'a> From<&'a MoveModuleEvent<'a>> for MoveModuleEventOwned {
        fn from(value: &'a MoveModuleEvent<'a>) -> Self {
            match value {
                MoveModuleEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl<'a> From<&'a OpenSubstateEvent<'a>> for OpenSubstateEventOwned {
        fn from(value: &'a OpenSubstateEvent<'a>) -> Self {
            match value {
                OpenSubstateEvent::Start {
                    node_id,
                    partition_num,
                    substate_key,
                    flags,
                } => Self::Start {
                    node_id: **node_id,
                    partition_num: **partition_num,
                    substate_key: (*substate_key).clone(),
                    flags: **flags,
                },
                OpenSubstateEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
                OpenSubstateEvent::End {
                    handle,
                    node_id,
                    size,
                } => Self::End {
                    handle: *handle,
                    node_id: **node_id,
                    size: *size,
                },
            }
        }
    }

    impl<'a> From<&'a ReadSubstateEvent<'a>> for ReadSubstateEventOwned {
        fn from(value: &'a ReadSubstateEvent<'a>) -> Self {
            match value {
                ReadSubstateEvent::OnRead {
                    handle,
                    value,
                    device,
                } => Self::OnRead {
                    handle: *handle,
                    value: (value.as_scrypto_value().to_owned(),),
                    device: *device,
                },
                ReadSubstateEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl<'a> From<&'a WriteSubstateEvent<'a>> for WriteSubstateEventOwned {
        fn from(value: &'a WriteSubstateEvent<'a>) -> Self {
            match value {
                WriteSubstateEvent::Start { handle, value } => Self::Start {
                    handle: *handle,
                    value: (value.as_scrypto_value().to_owned(),),
                },
                WriteSubstateEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl From<&CloseSubstateEvent> for CloseSubstateEventOwned {
        fn from(value: &CloseSubstateEvent) -> Self {
            match value {
                CloseSubstateEvent::Start(item) => Self::Start(*item),
            }
        }
    }

    impl<'a> From<&'a SetSubstateEvent<'a>> for SetSubstateEventOwned {
        fn from(value: &'a SetSubstateEvent<'a>) -> Self {
            match value {
                SetSubstateEvent::Start(item1, item2, item3, item4) => Self::Start(
                    **item1,
                    **item2,
                    (*item3).clone(),
                    (item4.as_scrypto_value().to_owned(),),
                ),
                SetSubstateEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl<'a> From<&'a RemoveSubstateEvent<'a>> for RemoveSubstateEventOwned {
        fn from(value: &'a RemoveSubstateEvent<'a>) -> Self {
            match value {
                RemoveSubstateEvent::Start(item1, item2, item3) => {
                    Self::Start(**item1, **item2, (*item3).clone())
                }
                RemoveSubstateEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl<'a> From<&'a ScanKeysEvent<'a>> for ScanKeysEventOwned {
        fn from(value: &'a ScanKeysEvent<'a>) -> Self {
            match value {
                ScanKeysEvent::Start => Self::Start,
                ScanKeysEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl<'a> From<&'a DrainSubstatesEvent<'a>> for DrainSubstatesEventOwned {
        fn from(value: &'a DrainSubstatesEvent<'a>) -> Self {
            match value {
                DrainSubstatesEvent::Start(item) => Self::Start(*item),
                DrainSubstatesEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }

    impl<'a> From<&'a ScanSortedSubstatesEvent<'a>> for ScanSortedSubstatesEventOwned {
        fn from(value: &'a ScanSortedSubstatesEvent<'a>) -> Self {
            match value {
                ScanSortedSubstatesEvent::Start => Self::Start,
                ScanSortedSubstatesEvent::IOAccess(item) => Self::IOAccess((*item).clone()),
            }
        }
    }
}
