use scrypto::abi;
use scrypto::args;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;
use crate::transaction::*;

/// The transaction executor.
pub struct TransactionExecutor<'l, L: Ledger> {
    ledger: &'l mut L,
    current_epoch: u64,
    nonce: u64,
}

#[derive(Debug)]
pub enum TransactionExecutionError {
    MissingEndInstruction,
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

    /// Returns the transaction nonce.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Set the transaction epoch.
    pub fn set_nonce(&self) -> u64 {
        self.nonce
    }

    /// Generates a new public key.
    pub fn new_public_key(&mut self) -> Address {
        let mut raw = [0u8; 33];
        raw[1..].copy_from_slice(sha256(self.nonce.to_string()).as_ref());
        self.nonce += 1;
        Address::PublicKey(raw)
    }

    /// Creates an account with 1,000,000 XRD in balance.
    pub fn create_account(&mut self, key: Address) -> Address {
        self.run(
            TransactionBuilder::new(self)
                .call_method(
                    SYSTEM_COMPONENT,
                    "free_xrd",
                    vec!["1000000".to_owned()],
                    None,
                )
                .create_account_with_resource(key, 1000000.into(), RADIX_TOKEN)
                .build(Vec::new())
                .unwrap(),
            false,
        )
        .unwrap()
        .component(0)
        .unwrap()
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> Address {
        self.run(
            TransactionBuilder::new(self)
                .publish_package(code)
                .build(Vec::new())
                .unwrap(),
            false,
        )
        .unwrap()
        .package(0)
        .unwrap()
    }

    /// Publishes a package to a specified address.
    pub fn publish_package_to(&mut self, code: &[u8], address: Address) {
        self.ledger
            .put_package(address, Package::new(code.to_vec()));
    }

    /// Executes a transaction.
    pub fn run(
        &mut self,
        transaction: Transaction,
        trace: bool,
    ) -> Result<Receipt, TransactionExecutionError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let signers = if let Some(Instruction::End { signers }) = transaction.instructions.last() {
            // TODO: check all signer addresses are public key; eventually should be computed from signature.
            signers.clone()
        } else {
            return Err(TransactionExecutionError::MissingEndInstruction);
        };

        let mut track = Track::new(
            self.ledger,
            self.current_epoch,
            sha256(self.nonce.to_string()),
            signers,
        );
        let mut proc = track.start_process(trace);

        let mut results = vec![];
        let mut success = true;
        for inst in &transaction.instructions {
            let res = match inst {
                Instruction::ReserveBucketId => {
                    proc.reserve_bucket_id();
                    Ok(None)
                }
                Instruction::ReserveBucketRefId => {
                    proc.reserve_bucket_ref_id();
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
                    bucket_ref,
                } => proc
                    .create_temp_bucket_ref(*amount, *resource_def, *bucket_ref)
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
                        args.iter().map(|v| v.encoded.clone()).collect(),
                    )
                    .map(|rtn| Some(SmartValue { encoded: rtn })),
                Instruction::CallMethod {
                    component,
                    method,
                    args,
                } => proc
                    .call_method(
                        *component,
                        method.as_str(),
                        args.iter().map(|v| v.encoded.clone()).collect(),
                    )
                    .map(|rtn| Some(SmartValue { encoded: rtn })),
                Instruction::DepositAll { component, method } => {
                    let buckets = proc.owned_buckets();
                    if !buckets.is_empty() {
                        proc.call_method(*component, method.as_str(), args!(buckets))
                            .map(|rtn| Some(SmartValue { encoded: rtn }))
                    } else {
                        Ok(None)
                    }
                }
                Instruction::End { .. } => proc.check_resource().map(|_| None),
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

        Ok(Receipt {
            transaction,
            success,
            results,
            logs: track.logs().clone(),
            new_entities: if success {
                track.new_entities().to_vec()
            } else {
                Vec::new()
            },
            execution_time,
        })
    }
}
