use scrypto::abi;
use scrypto::args;
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

impl<'l, L: Ledger> AbiProvider for TransactionExecutor<'l, L> {
    fn export_abi<A: AsRef<str>>(
        &self,
        package: Address,
        name: A,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let p = self
            .ledger
            .get_package(package)
            .ok_or(RuntimeError::PackageNotFound(package))?;

        BasicAbiProvider::new()
            .with_package(package, p.code().to_vec())
            .export_abi(package, name, trace)
    }

    fn export_abi_component(
        &self,
        component: Address,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let c = self
            .ledger
            .get_component(component)
            .ok_or(RuntimeError::ComponentNotFound(component))?;
        let p = self
            .ledger
            .get_package(c.package())
            .ok_or(RuntimeError::PackageNotFound(c.package()))?;
        BasicAbiProvider::new()
            .with_package(c.package(), p.code().to_vec())
            .export_abi(c.package(), c.name(), trace)
    }
}

impl<'l, L: Ledger> TransactionExecutor<'l, L> {
    pub fn new(ledger: &'l mut L, current_epoch: u64, nonce: u64) -> Self {
        Self {
            ledger,
            current_epoch,
            nonce,
        }
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Sets the current epoch.
    pub fn set_current_epoch(&mut self, current_epoch: u64) {
        self.current_epoch = current_epoch;
    }

    /// Returns the nonce.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Set the transaction epoch.
    pub fn set_nonce(&self) -> u64 {
        self.nonce
    }

    /// Creates a test account with 1000 XRD in balance.
    pub fn new_account(&mut self) -> Address {
        self.run(
            TransactionBuilder::new(self)
                .mint_resource(1000.into(), RADIX_TOKEN)
                .new_account_take_resource(1000.into(), RADIX_TOKEN)
                .build()
                .unwrap(),
            false,
        )
        .component(0)
        .unwrap()
    }

    /// Creates a test account with 1000 XRD in balance.
    pub fn publish_package(&mut self, code: &[u8]) -> Address {
        self.run(
            TransactionBuilder::new(self)
                .publish_package(code)
                .build()
                .unwrap(),
            false,
        )
        .package(0)
        .unwrap()
    }

    /// Executes a transaction.
    pub fn run(&mut self, transaction: Transaction, trace: bool) -> Receipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let mut track = Track::new(
            self.ledger,
            self.current_epoch,
            sha256(self.nonce.to_string()),
        );
        let mut proc = track.start_process(trace);

        let mut results = vec![];
        let mut success = true;
        for inst in &transaction.instructions {
            let res = match inst {
                Instruction::ReserveBid => {
                    proc.reserve_bid();
                    Ok(None)
                }
                Instruction::ReserveRid => {
                    proc.reserve_rid();
                    Ok(None)
                }
                Instruction::CreateTempBucket {
                    amount,
                    resource_def,
                    bucket,
                } => proc
                    .create_temp_bucket(*amount, *resource_def, *bucket)
                    .map(|_| None),
                Instruction::CreateTempBucketRef {
                    amount,
                    resource_def,
                    reference,
                } => proc
                    .create_temp_bucket_ref(*amount, *resource_def, *reference)
                    .map(|_| None),
                Instruction::CallFunction {
                    package,
                    name,
                    function,
                    args,
                } => proc
                    .call_function(
                        *package,
                        name.as_str(),
                        function.as_str(),
                        args.iter().map(|v| v.0.clone()).collect(),
                    )
                    .map(|rtn| Some(SmartValue(rtn))),
                Instruction::CallMethod {
                    component,
                    method,
                    args,
                } => proc
                    .call_method(
                        *component,
                        method.as_str(),
                        args.iter().map(|v| v.0.clone()).collect(),
                    )
                    .map(|rtn| Some(SmartValue(rtn))),
                Instruction::DepositAll { component, method } => {
                    let buckets: Vec<_> = proc
                        .owned_buckets()
                        .iter()
                        .map(|bid| scrypto::resource::Bucket::from(*bid))
                        .collect();
                    if !buckets.is_empty() {
                        proc.call_method(*component, method.as_str(), args!(buckets))
                            .map(|rtn| Some(SmartValue(rtn)))
                    } else {
                        Ok(None)
                    }
                }
                Instruction::End => proc.check_resource().map(|_| None),
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
        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());

        Receipt {
            transaction,
            success,
            results,
            logs: track.logs().clone(),
            new_addresses: if success {
                track.new_addresses().to_vec()
            } else {
                Vec::new()
            },
            execution_time,
        }
    }
}
