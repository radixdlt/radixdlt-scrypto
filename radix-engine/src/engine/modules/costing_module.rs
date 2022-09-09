use transaction::model::ExecutableTransaction;

use crate::engine::*;
use crate::fee::{FeeReserve, FeeTable, SystemApiCostingEntry};
use crate::model::ResourceContainer;
use crate::types::*;

pub struct CostingModule {
    fee_table: FeeTable,
}

impl CostingModule {
    pub fn new(fee_table: FeeTable) -> Self {
        Self { fee_table }
    }

    pub fn apply_static_fees<R: FeeReserve, T: ExecutableTransaction>(
        fee_reserve: &mut R,
        fee_table: &FeeTable,
        transaction: &T,
    ) {
        fee_reserve
            .consume(fee_table.tx_base_fee(), "base_fee")
            .expect("TX base fee not covered by system loan");

        fee_reserve
            .consume(
                fee_table.tx_decoding_per_byte() * transaction.transaction_payload_size() as u32,
                "decode_transaction",
            )
            .expect("TX decoding fee not covered by system loan");

        fee_reserve
            .consume(
                fee_table.tx_manifest_verification_per_byte()
                    * transaction.transaction_payload_size() as u32,
                "verify_manifest",
            )
            .expect("TX manifest verification fee not covered by system loan");

        fee_reserve
            .consume(
                fee_table.tx_signature_verification_per_sig()
                    * transaction.signer_public_keys().len() as u32,
                "verify_signatures",
            )
            .expect("TX signature verification fee not covered by system loan");
    }
}

impl<R: FeeReserve> Module<R> for CostingModule {
    fn pre_sys_call(
        &mut self,
        track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            SysCallInput::InvokeFunction {
                fn_identifier,
                input,
            } => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::InvokeFunction {
                                fn_identifier: fn_identifier.clone(),
                                input: &input,
                            }),
                        "invoke_function",
                    )
                    .map_err(ModuleError::CostingError)?;
                track
                    .fee_reserve
                    .consume(
                        self.fee_table.run_method_cost(None, &fn_identifier, &input),
                        "run_function",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::InvokeMethod {
                receiver,
                fn_identifier,
                input,
            } => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::InvokeMethod {
                                receiver: receiver.clone(),
                                input: &input,
                            }),
                        "invoke_method",
                    )
                    .map_err(ModuleError::CostingError)?;

                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .run_method_cost(Some(receiver), &fn_identifier, &input),
                        "run_method",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::BorrowNode { node_id } => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table.system_api_cost({
                            match node_id {
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
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::DropNode { .. } => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::DropNode { size: 0 }),
                        "drop_node",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::CreateNode { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::CreateNode {
                                size: 0, // TODO: get size of the value
                            }),
                        "create_node",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::GlobalizeNode { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::GlobalizeNode {
                                size: 0, // TODO: get size of the value
                            }),
                        "globalize_node",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::BorrowSubstateMut { substate_id } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        self.fee_table.system_api_cost({
                            match substate_id {
                                SubstateId::Bucket(_) => SystemApiCostingEntry::BorrowSubstate {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                SubstateId::Proof(_) => SystemApiCostingEntry::BorrowSubstate {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                SubstateId::Worktop => SystemApiCostingEntry::BorrowSubstate {
                                    // TODO: figure out loaded state and size
                                    loaded: true,
                                    size: 0,
                                },
                                SubstateId::Vault(_) => SystemApiCostingEntry::BorrowSubstate {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                                SubstateId::ComponentState(..) => {
                                    SystemApiCostingEntry::BorrowSubstate {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                SubstateId::ComponentInfo(..) => {
                                    SystemApiCostingEntry::BorrowSubstate {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                SubstateId::KeyValueStoreSpace(_) => {
                                    SystemApiCostingEntry::BorrowSubstate {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                SubstateId::KeyValueStoreEntry(..) => {
                                    SystemApiCostingEntry::BorrowSubstate {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                SubstateId::ResourceManager(..) => {
                                    SystemApiCostingEntry::BorrowSubstate {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                SubstateId::NonFungibleSpace(..) => {
                                    SystemApiCostingEntry::BorrowSubstate {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                SubstateId::NonFungible(..) => {
                                    SystemApiCostingEntry::BorrowSubstate {
                                        // TODO: figure out loaded state and size
                                        loaded: false,
                                        size: 0,
                                    }
                                }
                                SubstateId::Package(..) => SystemApiCostingEntry::BorrowSubstate {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                                SubstateId::System(..) => SystemApiCostingEntry::BorrowSubstate {
                                    // TODO: figure out loaded state and size
                                    loaded: false,
                                    size: 0,
                                },
                            }
                        }),
                        "borrow_substate",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::ReturnSubstateMut { substate_ref } => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table.system_api_cost({
                            match &substate_ref {
                                NativeSubstateRef::Stack(..) => {
                                    SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                }
                                NativeSubstateRef::Track(substate_id, _) => match substate_id {
                                    SubstateId::Vault(_) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::KeyValueStoreSpace(_) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::KeyValueStoreEntry(_, _) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::ResourceManager(_) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::Package(_) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::NonFungibleSpace(_) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::NonFungible(_, _) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::ComponentInfo(..) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::ComponentState(_) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::System(..) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::Bucket(..) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::Proof(..) => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                    SubstateId::Worktop => {
                                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                                    }
                                },
                            }
                        }),
                        "return_substate",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::ReadSubstate { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::ReadSubstate {
                                size: 0, // TODO: get size of the value
                            }),
                        "read_substate",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::WriteSubstate { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::WriteSubstate {
                                size: 0, // TODO: get size of the value
                            }),
                        "write_substate",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::TakeSubstate { .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::TakeSubstate {
                                size: 0, // TODO: get size of the value
                            }),
                        "read_substate",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::ReadTransactionHash => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::ReadTransactionHash),
                        "read_transaction_hash",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::GenerateUuid => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::GenerateUuid),
                        "generate_uuid",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::EmitLog { message, .. } => {
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::EmitLog {
                                size: message.len() as u32,
                            }),
                        "emit_log",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::CheckAccessRule { proof_ids, .. } => {
                // Costing
                track
                    .fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::CheckAccessRule {
                                size: proof_ids.len() as u32,
                            }),
                        "check_access_rule",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
        }

        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        _output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        code: &[u8],
    ) -> Result<(), ModuleError> {
        track
            .fee_reserve
            .consume(
                self.fee_table.wasm_instantiation_per_byte() * code.len() as u32,
                "instantiate_wasm",
            )
            .map_err(ModuleError::CostingError)
    }

    fn on_wasm_costing(
        &mut self,
        track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        units: u32,
    ) -> Result<(), ModuleError> {
        track
            .fee_reserve
            .consume(units, "run_wasm")
            .map_err(ModuleError::CostingError)
    }

    fn on_lock_fee(
        &mut self,
        track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        vault_id: VaultId,
        fee: ResourceContainer,
        contingent: bool,
    ) -> Result<ResourceContainer, ModuleError> {
        track
            .fee_reserve
            .repay(vault_id, fee, contingent)
            .map_err(ModuleError::CostingError)
    }
}
