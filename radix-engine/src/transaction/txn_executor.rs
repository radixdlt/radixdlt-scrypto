use scrypto::args;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::ledger::*;
use crate::transaction::*;

pub fn execute_transaction<T: Ledger>(
    ledger: &mut T,
    tx_hash: H256,
    transaction: Transaction,
    trace: bool,
) -> TransactionReceipt {
    let mut track = Track::new(tx_hash, ledger);
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
                resource_address,
            } => match reserved_bids.get(index as usize) {
                Some(bid) => proc
                    .move_to_bucket(amount, resource_address, *bid)
                    .map(|()| None),
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
        results,
        logs: track.logs().clone(),
        new_addresses: track.new_addresses().to_vec(),
    }
}
