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
pub struct TransactionExecutor<'l, L: SubstateStore> {
    ledger: &'l mut L,
    trace: bool,
}

impl<'l, L: SubstateStore> AbiProvider for TransactionExecutor<'l, L> {
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

impl<'l, L: SubstateStore> TransactionExecutor<'l, L> {
    pub fn new(ledger: &'l mut L, trace: bool) -> Self {
        Self { ledger, trace }
    }

    /// Returns an immutable reference to the ledger.
    pub fn ledger(&self) -> &L {
        self.ledger
    }

    /// Returns a mutable reference to the ledger.
    pub fn ledger_mut(&mut self) -> &mut L {
        self.ledger
    }

    /// Generates a new public key.
    pub fn new_public_key(&mut self) -> EcdsaPublicKey {
        let mut raw = [0u8; 33];
        raw[1..].copy_from_slice(sha256(self.ledger.get_nonce().to_string()).as_ref());
        self.ledger.increase_nonce();
        raw
    }

    /// Creates an account with 1,000,000 XRD in balance.
    pub fn new_account(&mut self, key: EcdsaPublicKey) -> Address {
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
    pub fn publish_package(&mut self, code: &[u8]) -> Result<Address, RuntimeError> {
        let receipt = self
            .run(
                TransactionBuilder::new(self)
                    .publish_package(code)
                    .build(Vec::new())
                    .unwrap(),
            )
            .unwrap();

        if receipt.result.is_ok() {
            Ok(receipt.package(0).unwrap())
        } else {
            Err(receipt.result.err().unwrap())
        }
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

    pub fn execute(&mut self, transaction: ValidatedTransaction) -> Receipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let transaction_hash = sha256(self.ledger.get_nonce().to_string());
        sha256(self.ledger.get_nonce().to_string());
        let mut track = Track::new(self.ledger, transaction_hash, transaction.signers.clone());
        let mut proc = track.start_process(self.trace);

        let mut error: Option<RuntimeError> = None;
        let mut outputs = vec![];
        for inst in transaction.clone().instructions {
            let result = match inst {
                ValidatedInstruction::TakeFromWorktop {
                    amount,
                    resource_address,
                } => proc.take_from_worktop(Some(amount), resource_address),
                ValidatedInstruction::TakeAllFromWorktop { resource_address } => {
                    proc.take_from_worktop(None, resource_address)
                }
                ValidatedInstruction::ReturnToWorktop { bid } => proc.return_to_worktop(bid),
                ValidatedInstruction::AssertWorktopContains {
                    amount,
                    resource_address,
                } => proc.assert_worktop_contains(amount, resource_address),
                ValidatedInstruction::CreateBucketRef { bid } => proc.create_bucket_ref(bid),
                ValidatedInstruction::CloneBucketRef { rid } => proc.clone_bucket_ref(rid),
                ValidatedInstruction::DropBucketRef { rid } => proc.drop_bucket_ref(rid),
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
                    outputs.push(data);
                }
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }

        // check resource
        error = error.or_else(|| match proc.check_resource() {
            Ok(_) => None,
            Err(e) => Some(e),
        });
        let new_entities = track.new_entities().to_vec();
        let logs = track.logs().clone();

        // commit state updates
        if error.is_none() {
            track.commit();
            self.ledger.increase_nonce();
        }

        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());

        Receipt {
            transaction,
            result: match error {
                Some(error) => Err(error),
                None => Ok(()),
            },
            outputs,
            logs,
            new_entities,
            execution_time,
        }
    }
}
