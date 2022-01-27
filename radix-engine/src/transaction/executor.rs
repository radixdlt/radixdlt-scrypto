use scrypto::abi;
use scrypto::buffer::*;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;
use crate::transaction::*;

/// An executor that runs transactions.
pub struct TransactionExecutor<'l, L: Ledger> {
    ledger: &'l mut L,
    current_epoch: u64,
    nonce: u64,
}

impl<'l, L: Ledger> AbiProvider for TransactionExecutor<'l, L> {
    fn export_abi<A: AsRef<str>>(
        &self,
        package_address: Address,
        blueprint_name: A,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let p = self
            .ledger
            .get_package(package_address)
            .ok_or(RuntimeError::PackageNotFound(package_address))?;

        BasicAbiProvider::new()
            .with_package(package_address, p.code().to_vec())
            .export_abi(package_address, blueprint_name, trace)
    }

    fn export_abi_component(
        &self,
        component_address: Address,
        trace: bool,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let c = self
            .ledger
            .get_component(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;
        let p = self
            .ledger
            .get_package(c.package_address())
            .ok_or(RuntimeError::PackageNotFound(c.package_address()))?;
        BasicAbiProvider::new()
            .with_package(c.package_address(), p.code().to_vec())
            .export_abi(c.package_address(), c.blueprint_name(), trace)
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

    /// Returns the underlying ledger.
    pub fn ledger(&self) -> &L {
        self.ledger
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
    pub fn new_account(&mut self, key: Address) -> Address {
        let free_xrd_amount = Decimal::from(1_000_000);

        self.run(
            TransactionBuilder::new(self)
                .call_method(
                    SYSTEM_COMPONENT,
                    "free_xrd",
                    vec![free_xrd_amount.to_string()],
                    None,
                )
                .new_account_with_resource(key, free_xrd_amount, RADIX_TOKEN)
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
        let receipt = self
            .run(
                TransactionBuilder::new(self)
                    .publish_package(code)
                    .build(Vec::new())
                    .unwrap(),
                false,
            )
            .unwrap();

        if !receipt.success {
            #[cfg(not(feature = "alloc"))]
            println!("{:?}", receipt);
            panic!("Failed to publish package. See receipt above.");
        } else {
            receipt.package(0).unwrap()
        }
    }

    /// Publishes a package to a specified address.
    pub fn overwrite_package(&mut self, address: Address, code: &[u8]) {
        self.ledger
            .put_package(address, Package::new(code.to_vec()));
    }

    /// Executes a transaction.
    pub fn run(
        &mut self,
        transaction: Transaction,
        trace: bool,
    ) -> Result<Receipt, TransactionValidationError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let transaction_validated = validate_transaction(&transaction)?;

        let mut track = Track::new(
            self.ledger,
            self.current_epoch,
            sha256(self.nonce.to_string()),
            transaction_validated.signers.clone(),
        );
        let mut proc = track.start_process(trace);

        let mut results = vec![];
        let mut success = true;
        for inst in &transaction_validated.instructions {
            let res = match inst {
                ValidatedInstruction::DeclareTempBucket => {
                    proc.declare_bucket();
                    Ok(None)
                }
                ValidatedInstruction::DeclareTempBucketRef => {
                    proc.declare_bucket_ref();
                    Ok(None)
                }
                ValidatedInstruction::TakeFromContext {
                    amount,
                    resource_address,
                    to,
                } => proc
                    .take_from_context(*amount, *resource_address, *to)
                    .map(|_| None),
                ValidatedInstruction::BorrowFromContext {
                    amount,
                    resource_address,
                    to,
                } => proc
                    .borrow_from_context(*amount, *resource_address, *to)
                    .map(|_| None),
                ValidatedInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function,
                    args,
                } => proc
                    .call_function(
                        // TODO: update interface
                        *package_address,
                        blueprint_name.as_str(),
                        function.as_str(),
                        args.clone(),
                    )
                    .map(Option::Some),
                ValidatedInstruction::CallMethod {
                    component_address,
                    method,
                    args,
                } => proc
                    .call_method(*component_address, method.as_str(), args.clone())
                    .map(Option::Some),

                ValidatedInstruction::DropAllBucketRefs => {
                    proc.drop_bucket_refs();
                    Ok(None)
                }
                ValidatedInstruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => {
                    let buckets = proc.list_buckets();
                    if !buckets.is_empty() {
                        proc.call_method(
                            *component_address,
                            method,
                            vec![validate_data(&scrypto_encode(&buckets)).unwrap()],
                        )
                        .map(Option::Some)
                    } else {
                        Ok(None)
                    }
                }
            };
            success &= res.is_ok();
            results.push(res);
            if !success {
                break;
            }
        }

        // check resource
        let res = proc.check_resource().map(|_| None);
        success &= res.is_ok();
        results.push(res);

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
            transaction: transaction_validated,
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
