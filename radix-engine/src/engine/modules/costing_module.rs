use crate::engine::*;
use crate::fee::{FeeReserve, FeeTable, SystemApiCostingEntry};
use crate::types::*;

pub struct CostingModule<'g, R: FeeReserve> {
    /// Fee reserve
    fee_reserve: &'g mut R,
    /// Fee table
    fee_table: &'g FeeTable,
}

impl<'g, R: FeeReserve> CostingModule<'g, R> {
    pub fn new(fee_reserve: &'g mut R, fee_table: &'g FeeTable) -> Self {
        Self {
            fee_reserve,
            fee_table,
        }
    }
}

impl<'g, R: FeeReserve> Module for CostingModule<'g, R> {
    fn pre_sys_call(
        &mut self,
        _heap: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            SysCallInput::InvokeFunction {
                fn_identifier,
                input,
            } => {
                self.fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::InvokeFunction {
                                fn_identifier: fn_identifier.clone(),
                                input: &input,
                            }),
                        "invoke_function",
                    )
                    .map_err(ModuleError::CostingError)?;
                self.fee_reserve
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
                self.fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::InvokeMethod {
                                receiver: receiver.clone(),
                                input: &input,
                            }),
                        "invoke_method",
                    )
                    .map_err(ModuleError::CostingError)?;

                self.fee_reserve
                    .consume(
                        self.fee_table
                            .run_method_cost(Some(receiver), &fn_identifier, &input),
                        "run_method",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::BorrowNode { node_id } => {
                self.fee_reserve
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
                                RENodeId::System => SystemApiCostingEntry::BorrowNode {
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
                self.fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::DropNode { size: 0 }),
                        "drop_node",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::CreateNode { .. } => {
                // Costing
                self.fee_reserve
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
                self.fee_reserve
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
                self.fee_reserve
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
                                SubstateId::System => SystemApiCostingEntry::BorrowSubstate {
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
                self.fee_reserve
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
                                    SubstateId::System => {
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
                self.fee_reserve
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
                self.fee_reserve
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
                self.fee_reserve
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
                self.fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::ReadTransactionHash),
                        "read_transaction_hash",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::GenerateUuid => {
                self.fee_reserve
                    .consume(
                        self.fee_table
                            .system_api_cost(SystemApiCostingEntry::GenerateUuid),
                        "generate_uuid",
                    )
                    .map_err(ModuleError::CostingError)?;
            }
            SysCallInput::EmitLog { message, .. } => {
                self.fee_reserve
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
                self.fee_reserve
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
        _heap: &mut Vec<CallFrame>,
        _output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }
}
