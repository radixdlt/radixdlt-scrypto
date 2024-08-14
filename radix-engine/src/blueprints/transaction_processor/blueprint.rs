use crate::blueprints::account::AccountSubstate;
use crate::blueprints::transaction_processor::TransactionProcessor;
use crate::internal_prelude::declare_native_blueprint_state;
use crate::internal_prelude::*;
use radix_engine_interface::types::*;



declare_native_blueprint_state! {
    blueprint_ident: SubTransactionProcessor,
    blueprint_snake_case: sub_transaction_processor,
    features: {
    },
    fields: {
        execution_state:  {
            ident: ExecutionState,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        }
    },
    collections: {
    }
}


pub type SubTransactionProcessorExecutionStateV1 = TransactionProcessor;