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
    let mut track = Track::new(sha256(Uuid::new_v4().to_string()), ledger);
    let mut proc = track.start_process(trace);

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
                Ok(None)
            }
            Instruction::MoveToBucket {
                index,
                amount,
                resource,
            } => match reserved_bids.get(index as usize) {
                Some(bid) => proc.move_to_bucket(amount, resource, *bid).map(|()| None),
                None => Err(RuntimeError::BucketNotReserved),
            },
            Instruction::CallFunction {
                blueprint,
                function,
                args,
            } => proc
                .call_function(blueprint, function.as_str(), args.0)
                .map(Option::from),
            Instruction::CallMethod {
                component,
                method,
                args,
            } => proc
                .call_method(component, method.as_str(), args.0)
                .map(Option::from),
            Instruction::DepositAll { component, method } => {
                let buckets: Vec<_> = proc
                    .owned_buckets()
                    .iter()
                    .map(|bid| scrypto::resource::Bucket::from(*bid))
                    .collect();
                if !buckets.is_empty() {
                    proc.call_method(component, method.as_str(), args!(buckets))
                        .map(Option::from)
                } else {
                    Ok(None)
                }
            }
            Instruction::Finalize => proc.finalize().map(|()| None),
        };
        results.push(res);
        if results.last().unwrap().is_err() {
            success = false;
            break;
        }
    }

    // commit state updates
    if success {
        track.commit();
    }

    TransactionReceipt {
        transaction,
        success,
        execution_time: now.elapsed().as_millis(),
        results,
        logs: track.logs().clone(),
        new_addresses: track.new_addresses().to_vec(),
    }
}
