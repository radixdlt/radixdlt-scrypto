use std::time::Instant;

use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::args;
use scrypto::utils::*;
use uuid::Uuid;

use crate::txn::*;

pub fn execute<T: Ledger>(
    ledger: &mut T,
    transaction: Transaction,
    trace: bool,
) -> TransactionReceipt {
    let now = Instant::now();
    let mut runtime = Runtime::new(sha256(Uuid::new_v4().to_string()), ledger);
    let mut proc = runtime.start_process(trace);

    let mut reserved_bids = vec![];
    let mut results = vec![];
    let mut success = true;
    for inst in transaction.instructions.clone() {
        let res = match inst {
            Instruction::ReserveBuckets { n } => {
                // TODO check if this is the first instruction
                for _ in 0..n {
                    reserved_bids.push(proc.reserve_bucket_id());
                }
                Ok(vec![])
            }
            Instruction::NewBucket {
                offset,
                amount,
                resource,
            } => match reserved_bids.get(offset as usize) {
                Some(bid) => proc
                    .withdraw_buckets_to_reserved(amount, resource, *bid)
                    .map(|()| vec![]),
                None => Err(RuntimeError::BucketNotReserved),
            },
            Instruction::CallFunction {
                package,
                blueprint,
                function,
                args,
            } => proc.call_function(package, blueprint.as_str(), function.as_str(), args),
            Instruction::CallMethod {
                component,
                method,
                args,
            } => proc.call_method(component, method.as_str(), args),
            Instruction::DepositAll { component, method } => {
                let buckets: Vec<_> = proc
                    .owned_buckets()
                    .iter()
                    .map(|bid| scrypto::resource::Bucket::from(*bid))
                    .collect();
                if !buckets.is_empty() {
                    proc.call_method(component, method.as_str(), args!(buckets))
                } else {
                    Ok(vec![])
                }
            }
            Instruction::Finalize => proc.finalize().map(|()| vec![]),
        };
        results.push(res);
        if results.last().unwrap().is_err() {
            success = false;
            break;
        }
    }

    // flush state updates
    if success {
        runtime.flush();
    }

    TransactionReceipt {
        transaction,
        success,
        execution_time: now.elapsed().as_millis(),
        results,
        logs: runtime.logs().clone(),
        new_addresses: runtime.new_addresses().to_vec(),
    }
}
