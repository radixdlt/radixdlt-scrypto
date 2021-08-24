use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::transaction::*;

pub fn execute(
    transaction: Transaction,
    trace: bool,
) -> Result<TransactionReceipt, TransactionError> {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    let mut process = Process::new(0, trace, &mut runtime);
    let mut results = vec![];
    for action in transaction.actions.clone() {
        match action {
            Action::InvokeBlueprint {
                package,
                blueprint,
                function,
                args,
            } => {
                results.push(process.call_function(package, blueprint, function, args));
            }
            Action::InvokeComponent {
                component,
                method,
                args,
            } => {
                results.push(process.call_method(component, method, args));
            }
            _ => {
                todo!()
            }
        }
    }

    process
        .finalize()
        .map_err(|e| TransactionError::FinalizationError(e))?;

    runtime.flush();

    Ok(TransactionReceipt {
        transaction,
        results,
        logs: runtime.logs().to_owned(),
    })
}
