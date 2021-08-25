use radix_engine::execution::*;
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
    let mut success = true;
    for action in transaction.actions.clone() {
        match action {
            Action::CallBlueprint {
                package,
                blueprint,
                function,
                args,
            } => {
                results.push(process.call_function(package, blueprint, function, args));
            }
            Action::CallComponent {
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

        if let Some(Err(_)) = results.last() {
            success = false;
            break;
        }
    }

    // finalize and flush if success
    if success {
        process
            .finalize()
            .map_err(|e| TransactionError::FinalizationError(e))?;

        runtime.flush();
    }

    Ok(TransactionReceipt {
        transaction,
        success,
        results,
        logs: runtime.logs().to_owned(),
    })
}
