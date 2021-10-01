use scrypto::abi;
use scrypto::args;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

use crate::engine::*;
use crate::ledger::*;
use crate::transaction::*;

/// A transaction executor.
pub struct TransactionExecutor<'l, L: Ledger> {
    ledger: &'l mut L,
    current_epoch: u64,
    nonce: u64,
}

impl<'l, L: Ledger> TransactionExecutor<'l, L> {
    pub fn new(ledger: &'l mut L, current_epoch: u64, nonce: u64) -> Self {
        Self {
            ledger,
            current_epoch,
            nonce,
        }
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn set_epoch(&mut self, current_epoch: u64) {
        self.current_epoch = current_epoch;
    }

    pub fn export_abi<A: AsRef<str>>(
        &self,
        package: Address,
        blueprint: A,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        // deterministic ledger, current_epoch and transaction hash
        let mut ledger = InMemoryLedger::new();
        ledger.put_package(
            package,
            self.ledger
                .get_package(package)
                .ok_or(RuntimeError::PackageNotFound(package.to_owned()))?,
        );
        let current_epoch = 0;
        let tx_hash = sha256([]);

        // Start a process and run abi generator
        let mut track = Track::new(&mut ledger, current_epoch, tx_hash);
        let mut proc = track.start_process(trace);
        let output: (Vec<abi::Function>, Vec<abi::Method>) = proc
            .call_abi((package, blueprint.as_ref().to_owned()))
            .and_then(|rtn| scrypto_decode(&rtn).map_err(RuntimeError::InvalidData))?;

        Ok(abi::Blueprint {
            package: package.to_string(),
            name: blueprint.as_ref().to_string(),
            functions: output.0,
            methods: output.1,
        })
    }

    pub fn export_abi_by_component(
        &self,
        component: Address,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let c = self
            .ledger
            .get_component(component)
            .ok_or(RuntimeError::ComponentNotFound(component))?;
        self.export_abi(c.blueprint().0.clone(), c.blueprint().1.clone(), trace)
    }

    pub fn run(&mut self, tx: &Transaction, trace: bool) -> Receipt {
        let mut track = Track::new(
            self.ledger,
            self.current_epoch,
            sha256(self.nonce.to_string()),
        );
        let mut proc = track.start_process(trace);

        let mut results = vec![];
        let mut success = true;
        for inst in &tx.instructions {
            let res = match inst {
                Instruction::ReserveBucket { resource_def } => {
                    proc.reserve_bucket(*resource_def);
                    Ok(None)
                }
                Instruction::BorrowBucket { bucket } => proc.borrow_bucket(*bucket).map(|_| None),
                Instruction::MoveToBucket {
                    amount,
                    resource_def,
                    bucket,
                } => proc
                    .move_to_bucket(*amount, *resource_def, *bucket)
                    .map(|_| None),
                Instruction::CallFunction {
                    blueprint,
                    function,
                    args,
                } => proc
                    .call_function(blueprint.clone(), function.as_str(), args.clone())
                    .map(Option::from),
                Instruction::CallMethod {
                    component,
                    method,
                    args,
                } => proc
                    .call_method(*component, method.as_str(), args.clone())
                    .map(Option::from),
                Instruction::DepositAll { component, method } => {
                    let buckets: Vec<_> = proc
                        .owned_buckets()
                        .iter()
                        .map(|bid| scrypto::resource::Bucket::from(*bid))
                        .collect();
                    if !buckets.is_empty() {
                        proc.call_method(*component, method.as_str(), args!(buckets))
                            .map(Option::from)
                    } else {
                        Ok(None)
                    }
                }
                Instruction::End => proc.finalize().map(|_| None),
            };
            success &= res.is_ok();
            results.push(res);
            if !success {
                break;
            }
        }

        // commit state updates
        if success {
            track.commit();
            self.nonce += 1;
        }

        Receipt {
            success,
            results,
            logs: track.logs().clone(),
            new_addresses: if success {
                track.new_addresses().to_vec()
            } else {
                Vec::new()
            },
        }
    }
}
