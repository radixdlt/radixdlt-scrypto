use radix_engine::execution::*;
use radix_engine::model::*;
use scrypto::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::transaction::*;

pub fn execute(transaction: Transaction, trace: bool) -> TransactionReceipt {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    let mut collected_resources = HashMap::<Address, Bucket>::new();
    let mut reserved_bids = vec![];
    let mut prepared_buckets = HashMap::<BID, Bucket>::new();
    let mut results = vec![];
    let mut success = true;
    for inst in transaction.instructions.clone() {
        let res = match inst {
            Instruction::ReserveBuckets { n } => {
                // TODO check if this is the first instruction
                for _ in 0..n {
                    reserved_bids.push(runtime.new_transient_bid());
                }
                Ok(vec![])
            }
            Instruction::NewBucket {
                offset,
                amount,
                resource,
            } => match reserved_bids.get(offset as usize) {
                Some(bid) => {
                    let bucket = collected_resources
                        .get_mut(&resource)
                        .and_then(|b| b.take(amount).ok());

                    match bucket {
                        Some(b) => {
                            prepared_buckets.insert(bid.clone(), b);
                            Ok(vec![])
                        }
                        None => Err(RuntimeError::AccountingError(
                            BucketError::InsufficientBalance,
                        )),
                    }
                }
                None => Err(RuntimeError::BucketNotFound),
            },
            Instruction::CallFunction {
                package,
                blueprint,
                function,
                args,
            } => {
                let mut process = Process::new(0, trace, &mut runtime);

                // move resources
                process.put_resources(prepared_buckets.clone(), HashMap::new());
                prepared_buckets.clear();

                // run
                process
                    .target_function(package, blueprint.as_str(), function, args)
                    .and_then(|target| {
                        let result = process.run(target);

                        // move resources
                        for bucket in process.take_resources().0.values() {
                            collected_resources
                                .entry(bucket.resource())
                                .or_insert(Bucket::new(0.into(), bucket.resource()))
                                .put(bucket.clone())
                                .unwrap();
                        }
                        result
                    })
            }
            Instruction::CallMethod {
                component,
                method,
                args,
            } => {
                let mut process = Process::new(0, trace, &mut runtime);

                // move resources
                process.put_resources(prepared_buckets.clone(), HashMap::new());
                prepared_buckets.clear();

                // run
                process
                    .target_method(component, method, args)
                    .and_then(|target| {
                        let result = process.run(target);

                        // move resources
                        for bucket in process.take_resources().0.values() {
                            collected_resources
                                .entry(bucket.resource())
                                .or_insert(Bucket::new(0.into(), bucket.resource()))
                                .put(bucket.clone())
                                .unwrap();
                        }
                        result
                    })
            }
            Instruction::Finalize => {
                // TODO check if this is the last instruction
                let mut success = true;
                for (_, bucket) in &collected_resources {
                    if bucket.amount() > 0.into() {
                        if trace {
                            println!("Resource leak: {:?}", bucket);
                        }
                        success = false;
                    }
                }
                if success {
                    Ok(vec![])
                } else {
                    Err(RuntimeError::ResourceLeak)
                }
            }
        };

        if res.is_ok() {
            results.push(res);
        } else {
            results.push(res);
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
        results,
        logs: runtime.logs().clone(),
    }
}
