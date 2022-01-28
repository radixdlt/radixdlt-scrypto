use scrypto::abi;
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
    trace: bool,
}

impl<'l, L: Ledger> AbiProvider for TransactionExecutor<'l, L> {
    fn export_abi<A: AsRef<str>>(
        &self,
        package_address: Address,
        blueprint_name: A,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let p = self
            .ledger
            .get_package(package_address)
            .ok_or(RuntimeError::PackageNotFound(package_address))?;

        BasicAbiProvider::new(self.trace)
            .with_package(package_address, p.code().to_vec())
            .export_abi(package_address, blueprint_name)
    }

    fn export_abi_component(
        &self,
        component_address: Address,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let c = self
            .ledger
            .get_component(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;
        let p = self
            .ledger
            .get_package(c.package_address())
            .ok_or(RuntimeError::PackageNotFound(c.package_address()))?;
        BasicAbiProvider::new(self.trace)
            .with_package(c.package_address(), p.code().to_vec())
            .export_abi(c.package_address(), c.blueprint_name())
    }
}

impl<'l, L: Ledger> TransactionExecutor<'l, L> {
    pub fn new(ledger: &'l mut L, current_epoch: u64, nonce: u64, trace: bool) -> Self {
        Self {
            ledger,
            current_epoch,
            nonce,
            trace,
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
        )
        .unwrap()
        .package(0)
        .unwrap()
    }

    /// Publishes a package to a specified address.
    pub fn overwrite_package(&mut self, address: Address, code: &[u8]) {
        self.ledger
            .put_package(address, Package::new(code.to_vec()));
    }

    /// This is a convenience method that validates and runs a transaction in one shot.
    ///
    /// You might also consider `validate()` and `execute()` in this implementation.
    pub fn run(&mut self, transaction: Transaction) -> Result<Receipt, TransactionValidationError> {
        let validated_transaction = self.validate(transaction)?;
        let receipt = self.execute(validated_transaction);
        Ok(receipt)
    }

    pub fn validate(
        &mut self,
        transaction: Transaction,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        validate_transaction(&transaction)
    }

    pub fn execute(&mut self, validated_transaction: ValidatedTransaction) -> Receipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        // Ledger state updates introduced by this transaction
        let mut track = Track::new(
            self.ledger,
            self.current_epoch,
            sha256(self.nonce.to_string()),
            validated_transaction.signers.clone(),
        );
        let mut proc = track.start_process(self.trace);

        let mut error: Option<RuntimeError> = None;
        let mut returns = vec![];
        for inst in validated_transaction.clone().instructions {
            let result = match inst {
                ValidatedInstruction::CreateTempBucket {
                    amount,
                    resource_address,
                } => proc.create_temp_bucket(amount, resource_address),
                ValidatedInstruction::CreateTempBucketRef { bid } => {
                    proc.create_temp_bucket_ref(bid)
                }
                ValidatedInstruction::CloneTempBucketRef { rid } => proc.clone_temp_bucket_ref(rid),
                ValidatedInstruction::DropTempBucketRef { rid } => proc.drop_temp_bucket_ref(rid),
                ValidatedInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function,
                    args,
                } => proc.call_function(package_address, &blueprint_name, &function, args),
                ValidatedInstruction::CallMethod {
                    component_address,
                    method,
                    args,
                } => proc.call_method(component_address, &method, args),
                ValidatedInstruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => proc.call_method_with_all_resources(component_address, &method),
            };
            match result {
                Ok(data) => {
                    returns.push(data);
                }
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }

        // check resource
        error = error.or(match proc.check_resource() {
            Ok(_) => None,
            Err(e) => Some(e),
        });

        // commit state updates
        if error.is_none() {
            track.commit();
            self.nonce += 1;
        }

        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());

        let new_entities = if error.is_none() {
            track.new_entities().to_vec()
        } else {
            Vec::new()
        };

        let receipt = Receipt {
            transaction: validated_transaction,
            error,
            returns,
            logs: track.logs().clone(),
            new_entities,
            execution_time,
        };
        if self.trace {
            #[cfg(not(feature = "alloc"))]
            println!("{:?}", receipt);
        }
        receipt
    }
}
