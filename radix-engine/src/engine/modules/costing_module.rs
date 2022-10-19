use crate::engine::*;
use crate::fee::{FeeReserve, FeeReserveError, SystemApiCostingEntry};
use crate::model::Resource;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum CostingError {
    FeeReserveError(FeeReserveError),
}

#[derive(Default)]
pub struct CostingModule;

impl<R: FeeReserve> Module<R> for CostingModule {
    fn pre_sys_call(
        &mut self,
        track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            SysCallInput::InvokeScrypto {
                fn_ident,
                args,
                depth,
            } => {
                if depth > 0 {
                    track
                        .fee_reserve
                        .consume(
                            track
                                .fee_table
                                .system_api_cost(SystemApiCostingEntry::InvokeScrypto {
                                    fn_ident: fn_ident.clone(),
                                    args: &args,
                                }),
                            "invoke_scrypto",
                            false,
                        )
                        .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
                }
            }
            SysCallInput::InvokeNative {
                fn_ident,
                args,
                depth,
            } => {
                if depth > 0 {
                    track
                        .fee_reserve
                        .consume(
                            track
                                .fee_table
                                .system_api_cost(SystemApiCostingEntry::InvokeNative {
                                    fn_ident: fn_ident.clone(),
                                    args: &args,
                                }),
                            "invoke_native",
                            false,
                        )
                        .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
                }
            }
            SysCallInput::ReadOwnedNodes => {
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::ReadOwnedNodes),
                        "read_owned_nodes",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::BorrowNode { node_id } => {
                track
                    .fee_reserve
                    .consume(
                        track.fee_table.system_api_cost({
                            match node_id {
                                RENodeId::Global(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                RENodeId::AuthZoneStack(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                RENodeId::Bucket(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                RENodeId::Proof(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                RENodeId::Worktop => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                RENodeId::Vault(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                                RENodeId::Component(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                                RENodeId::KeyValueStore(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                                RENodeId::NonFungibleStore(_) => {
                                    SystemApiCostingEntry::BorrowNode {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                RENodeId::ResourceManager(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                                RENodeId::Package(_) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                                RENodeId::System(..) => SystemApiCostingEntry::BorrowNode {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                            }
                        }),
                        "borrow_node",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::DropNode { .. } => {
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::DropNode { size: 0 }),
                        "drop_node",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::CreateNode { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::CreateNode {
                                size: 0, // TODO: get size of the value
                            }),
                        "create_node",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::GlobalizeNode { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::GlobalizeNode {
                                size: 0, // TODO: get size of the value
                            }),
                        "globalize_node",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::LockSubstate { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::LockSubstate {
                                size: 0, // TODO: get size of the value
                            }),
                        "lock_substate",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::GetRef { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::ReadSubstate {
                                size: 0, // TODO: get size of the value
                            }),
                        "read_substate",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::GetRefMut { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::WriteSubstate {
                                size: 0, // TODO: get size of the value
                            }),
                        "write_substate",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::DropLock { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::DropLock),
                        "drop_lock",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::TakeSubstate { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::TakeSubstate {
                                size: 0, // TODO: get size of the value
                            }),
                        "take_substate",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::ReadTransactionHash => {
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::ReadTransactionHash),
                        "read_transaction_hash",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::ReadBlob { .. } => {
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::ReadBlob { size: 0 }), // TODO pass the right size
                        "read_blob",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::GenerateUuid => {
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::GenerateUuid),
                        "generate_uuid",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
            SysCallInput::EmitLog { message, .. } => {
                track
                    .fee_reserve
                    .consume(
                        track
                            .fee_table
                            .system_api_cost(SystemApiCostingEntry::EmitLog {
                                size: message.len() as u32,
                            }),
                        "emit_log",
                        false,
                    )
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            }
        }

        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        _output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        code: &[u8],
    ) -> Result<(), ModuleError> {
        track
            .fee_reserve
            .consume(
                track.fee_table.wasm_instantiation_per_byte() * code.len() as u32,
                "instantiate_wasm",
                false,
            )
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))
    }

    fn on_wasm_costing(
        &mut self,
        track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        units: u32,
    ) -> Result<(), ModuleError> {
        track
            .fee_reserve
            .consume(units, "run_wasm", false)
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))
    }

    fn on_lock_fee(
        &mut self,
        track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, ModuleError> {
        track
            .fee_reserve
            .repay(vault_id, fee, contingent)
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))
    }

    fn on_run(
        &mut self,
        track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        actor: &REActor,
        input: &ScryptoValue,
    ) -> Result<(), ModuleError> {
        match actor {
            REActor::Function(ResolvedFunction::Native(native_function)) => track
                .fee_reserve
                .consume(
                    track
                        .fee_table
                        .run_native_function_cost(&native_function, &input),
                    "run_native_function",
                    false,
                )
                .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e))),
            REActor::Method(ResolvedMethod::Native(native_method), _) => track
                .fee_reserve
                .consume(
                    track
                        .fee_table
                        .run_native_method_cost(&native_method, &input),
                    "run_native_method",
                    false,
                )
                .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e))),
            _ => Ok(()),
        }
    }
}
