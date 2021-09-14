use std::time::Instant;

use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::buffer::*;
use scrypto::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::txn::*;

pub fn execute<T: Ledger>(
    ledger: &mut T,
    transaction: Transaction,
    trace: bool,
) -> TransactionReceipt {
    let now = Instant::now();
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
                let target =
                    process.prepare_call_function(package, blueprint.as_str(), function, args);
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
                let target = process.prepare_call_method(component, method, args);
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
                let mut buckets = vec![];
                for (_, bucket) in resource_collector.iter_mut() {
                    if bucket.amount() > 0.into() {
                        let bid = runtime.new_transient_bid();
                        buckets.push(bid);
                        moving_buckets.insert(bid, bucket.take(bucket.amount()).unwrap());
                    }
                }

                if !buckets.is_empty() {
                    let mut process = Process::new(0, trace, &mut runtime);
                    let target = process.prepare_call_method(
                        component,
                        method.clone(),
                        vec![scrypto_encode(&buckets)],
                    );
                    target.and_then(|target| call(&mut process, target, &mut moving_buckets, None))
                } else {
                    Ok(vec![])
                }
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
        execution_time: now.elapsed().as_millis(),
        results,
        logs: runtime.logs().clone(),
    }
}

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
    let (buckets, references) = process.take_resources();
    match resource_collector {
        Some(collector) => {
            for bucket in buckets.values() {
                collector
                    .entry(bucket.resource())
                    .or_insert(Bucket::new(0.into(), bucket.resource()))
                    .put(bucket.clone())
                    .unwrap();
            }
            if !references.is_empty() {
                return Err(RuntimeError::UnexpectedResourceReturn);
            }
        }
        None => {
            if !buckets.is_empty() || !references.is_empty() {
                return Err(RuntimeError::UnexpectedResourceReturn);
            }
        }
    }

    result
}
