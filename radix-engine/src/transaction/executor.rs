use scrypto::abi;
use scrypto::crypto::sha256;
use scrypto::engine::types::*;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::errors::*;
use crate::ledger::*;
use crate::model::*;
use crate::transaction::*;

/// An executor that runs transactions.
pub struct TransactionExecutor<'l, L: SubstateStore> {
    ledger: &'l mut L,
    trace: bool,
}

impl<'l, L: SubstateStore> AbiProvider for TransactionExecutor<'l, L> {
    fn export_abi(
        &self,
        package_ref: PackageRef,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let package = self
            .ledger
            .get_package(package_ref)
            .ok_or(RuntimeError::PackageNotFound(package_ref))?;

        BasicAbiProvider::new(self.trace)
            .with_package(package_ref, package.code().to_vec())
            .export_abi(package_ref, blueprint_name)
    }

    fn export_abi_component(
        &self,
        component_ref: ComponentRef,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let component = self
            .ledger
            .get_component(component_ref)
            .ok_or(RuntimeError::ComponentNotFound(component_ref))?;
        let package = self
            .ledger
            .get_package(component.package_ref())
            .ok_or(RuntimeError::PackageNotFound(component.package_ref()))?;
        BasicAbiProvider::new(self.trace)
            .with_package(component.package_ref(), package.code().to_vec())
            .export_abi(component.package_ref(), component.blueprint_name())
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
        raw[1..].copy_from_slice(&sha256(self.ledger.get_nonce().to_string()).0);
        self.ledger.increase_nonce();
        EcdsaPublicKey(raw)
    }

    /// Creates an account with 1,000,000 XRD in balance.
    pub fn new_account(&mut self, key: EcdsaPublicKey) -> ComponentRef {
        let free_xrd_amount = Decimal::from(1_000_000);

        self.run(
            TransactionBuilder::new(self)
                .call_method(
                    SYSTEM_COMPONENT,
                    "free_xrd",
                    vec![free_xrd_amount.to_string()],
                    None,
                )
                .new_account_with_resource(
                    key,
                    &ResourceSpecification::Fungible {
                        amount: free_xrd_amount,
                        resource_def_ref: RADIX_TOKEN,
                    },
                )
                .build(Vec::new())
                .unwrap(),
        )
        .unwrap()
        .new_component_refs[0]
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> Result<PackageRef, RuntimeError> {
        let receipt = self
            .run(
                TransactionBuilder::new(self)
                    .publish_package(code)
                    .build(Vec::new())
                    .unwrap(),
            )
            .unwrap();

        if receipt.result.is_ok() {
            Ok(receipt.new_package_refs[0])
        } else {
            Err(receipt.result.err().unwrap())
        }
    }

    /// Publishes a package to a specified address.
    pub fn overwrite_package(&mut self, package_ref: PackageRef, code: &[u8]) {
        self.ledger
            .put_package(package_ref, Package::new(code.to_vec()));
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
                    resource_def_ref,
                } => proc.take_from_worktop(ResourceSpecification::Fungible {
                    amount,
                    resource_def_ref,
                }),
                ValidatedInstruction::TakeAllFromWorktop { resource_def_ref } => {
                    proc.take_from_worktop(ResourceSpecification::All { resource_def_ref })
                }
                ValidatedInstruction::TakeNonFungiblesFromWorktop {
                    keys,
                    resource_def_ref,
                } => proc.take_from_worktop(ResourceSpecification::NonFungible {
                    keys,
                    resource_def_ref,
                }),
                ValidatedInstruction::ReturnToWorktop { bucket_id } => {
                    proc.return_to_worktop(bucket_id)
                }
                ValidatedInstruction::AssertWorktopContains {
                    amount,
                    resource_def_ref,
                } => proc.assert_worktop_contains(amount, resource_def_ref),
                ValidatedInstruction::CreateBucketRef { bucket_id } => {
                    proc.create_bucket_ref(bucket_id)
                }
                ValidatedInstruction::CloneBucketRef { bucket_ref_id } => {
                    proc.clone_bucket_ref(bucket_ref_id)
                }
                ValidatedInstruction::DropBucketRef { bucket_ref_id } => {
                    proc.drop_bucket_ref(bucket_ref_id)
                }
                ValidatedInstruction::CallFunction {
                    package_ref,
                    blueprint_name,
                    function,
                    args,
                } => proc.call_function(package_ref, &blueprint_name, &function, args),
                ValidatedInstruction::CallMethod {
                    component_ref,
                    method,
                    args,
                } => proc.call_method(component_ref, &method, args),
                ValidatedInstruction::CallMethodWithAllResources {
                    component_ref,
                    method,
                } => proc.call_method_with_all_resources(component_ref, &method),
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
        let new_package_refs = track.new_package_refs().to_vec();
        let new_component_refs = track.new_component_refs().to_vec();
        let new_resource_def_refs = track.new_resource_def_refs().to_vec();
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
            new_package_refs,
            new_component_refs,
            new_resource_def_refs,
            execution_time,
        }
    }
}
