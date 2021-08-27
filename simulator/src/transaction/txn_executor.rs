use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::buffer::*;
use scrypto::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::transaction::*;

fn call<L: Ledger>(
    process: &mut Process<L>,
    target: Target,
    buckets: &mut HashMap<BID, Bucket>,
    resource_collector: Option<&mut HashMap<Address, Bucket>>,
) -> Result<Vec<u8>, RuntimeError> {
    // move resources
    process.put_resources(buckets.clone(), HashMap::new());
    buckets.clear();

    // run
    let result = process.run(target);

    // move resources
    match resource_collector {
        Some(collector) => {
            for bucket in process.take_resources().0.values() {
                collector
                    .entry(bucket.resource())
                    .or_insert(Bucket::new(0.into(), bucket.resource()))
                    .put(bucket.clone())
                    .unwrap();
            }
        }
        None => {
            if !process.take_resources().0.is_empty() {
                return Err(RuntimeError::UnexpectedResourceReturn);
            }
        }
    }

    result
}

pub fn execute<T: Ledger>(
    ledger: &mut T,
    transaction: Transaction,
    trace: bool,
) -> TransactionReceipt {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut runtime = Runtime::new(tx_hash, ledger);

    let mut reserved_bids = vec![];
    let mut resource_collector = HashMap::<Address, Bucket>::new();
    let mut moving_buckets = HashMap::<BID, Bucket>::new();
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
                    let bucket = resource_collector
                        .get_mut(&resource)
                        .and_then(|b| b.take(amount).ok());

                    match bucket {
                        Some(b) => {
                            moving_buckets.insert(bid.clone(), b);
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
                let target = process.target_function(package, blueprint.as_str(), function, args);
                target.and_then(|target| {
                    call(
                        &mut process,
                        target,
                        &mut moving_buckets,
                        Some(&mut resource_collector),
                    )
                })
            }
            Instruction::CallMethod {
                component,
                method,
                args,
            } => {
                let mut process = Process::new(0, trace, &mut runtime);
                let target = process.target_method(component, method, args);
                target.and_then(|target| {
                    call(
                        &mut process,
                        target,
                        &mut moving_buckets,
                        Some(&mut resource_collector),
                    )
                })
            }
            Instruction::DepositAll { component, method } => {
                let mut result = Ok(vec![]);
                for (_, bucket) in resource_collector.iter_mut() {
                    if bucket.amount() > 0.into() {
                        let bid = runtime.new_transient_bid();
                        moving_buckets.insert(bid, bucket.take(bucket.amount()).unwrap());

                        let mut process = Process::new(0, trace, &mut runtime);
                        let target = process.target_method(
                            component,
                            method.clone(),
                            vec![scrypto_encode(&bid)],
                        );
                        result = target.and_then(|target| {
                            call(&mut process, target, &mut moving_buckets, None)
                        });

                        if result.is_err() {
                            break;
                        }
                    }
                }
                result
            }
            Instruction::Finalize => {
                // TODO check if this is the last instruction
                let mut success = true;
                for (_, bucket) in &resource_collector {
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
