use scrypto::crypto::sha256;
use scrypto::engine::types::*;
use scrypto::prelude::NonFungibleAddress;
use scrypto::resource::ProofRule;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::{abi, any_of};

use crate::engine::*;
use crate::errors::*;
use crate::ledger::*;
use crate::model::*;
use crate::transaction::*;

/// An executor that runs transactions.
pub struct TransactionExecutor<'l, L: SubstateStore> {
    substate_store: &'l mut L,
    trace: bool,
}

impl<'l, L: SubstateStore> AbiProvider for TransactionExecutor<'l, L> {
    fn export_abi(
        &self,
        package_id: PackageId,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let package: Package = self
            .substate_store
            .get_decoded_substate(&package_id)
            .map(|(package, _)| package)
            .ok_or(RuntimeError::PackageNotFound(package_id))?;

        BasicAbiProvider::new(self.trace)
            .with_package(package_id, package.code().to_vec())
            .export_abi(package_id, blueprint_name)
    }

    fn export_abi_component(
        &self,
        component_id: ComponentId,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let component: Component = self
            .substate_store
            .get_decoded_substate(&component_id)
            .map(|(component, _)| component)
            .ok_or(RuntimeError::ComponentNotFound(component_id))?;
        let package: Package = self
            .substate_store
            .get_decoded_substate(&component.package_id())
            .map(|(package, _)| package)
            .unwrap();
        BasicAbiProvider::new(self.trace)
            .with_package(component.package_id(), package.code().to_vec())
            .export_abi(component.package_id(), component.blueprint_name())
    }
}

impl<'l, L: SubstateStore> TransactionExecutor<'l, L> {
    pub fn new(substate_store: &'l mut L, trace: bool) -> Self {
        Self {
            substate_store,
            trace,
        }
    }

    /// Returns an immutable reference to the ledger.
    pub fn substate_store(&self) -> &L {
        self.substate_store
    }

    /// Returns a mutable reference to the ledger.
    pub fn substate_store_mut(&mut self) -> &mut L {
        self.substate_store
    }

    /// Generates a new public key.
    pub fn new_public_key(&mut self) -> EcdsaPublicKey {
        let mut raw = [0u8; 33];
        raw[1..].copy_from_slice(&sha256(self.substate_store.get_nonce().to_string()).0);
        self.substate_store.increase_nonce();
        EcdsaPublicKey(raw)
    }

    /// Creates an account with 1,000,000 XRD in balance.
    pub fn new_account(&mut self, withdraw_auth: &ProofRule) -> ComponentId {
        self.run(
            TransactionBuilder::new(self)
                .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
                .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                    builder.new_account_with_resource(withdraw_auth, bucket_id)
                })
                .build(Vec::new())
                .unwrap(),
        )
        .unwrap()
        .new_component_ids[0]
    }

    /// Creates a new public key and account associated with it
    pub fn new_public_key_with_account(&mut self) -> (EcdsaPublicKey, ComponentId) {
        let key = self.new_public_key();
        let id = NonFungibleId::new(key.to_vec());
        let auth_address = NonFungibleAddress::new(ECDSA_TOKEN, id);
        let withdraw_auth = any_of!(auth_address);
        let account = self.new_account(&withdraw_auth);
        (key, account)
    }

    /// Publishes a package.
    pub fn publish_package<T: AsRef<[u8]>>(&mut self, code: T) -> Result<PackageId, RuntimeError> {
        let receipt = self
            .run(
                TransactionBuilder::new(self)
                    .publish_package(code.as_ref())
                    .build(Vec::new())
                    .unwrap(),
            )
            .unwrap();

        if receipt.result.is_ok() {
            Ok(receipt.new_package_ids[0])
        } else {
            Err(receipt.result.err().unwrap())
        }
    }

    /// Overwrites a package.
    pub fn overwrite_package(&mut self, package_id: PackageId, code: &[u8]) {
        let package = Package::new(code.to_vec());
        self.substate_store.put_encoded_substate(
            &package_id,
            &package,
            self.substate_store.get_nonce(),
        );
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

        let transaction_hash = sha256(self.substate_store.get_nonce().to_string());
        sha256(self.substate_store.get_nonce().to_string());
        let mut track = Track::new(
            self.substate_store,
            transaction_hash,
            transaction.signers.clone(),
        );
        let mut proc = track.start_process(self.trace);

        let mut error: Option<RuntimeError> = None;
        let mut outputs = vec![];
        for inst in transaction.clone().instructions {
            let result = match inst {
                ValidatedInstruction::TakeFromWorktop { resource_def_id } => proc
                    .take_all_from_worktop(resource_def_id)
                    .map(|bucket_id| {
                        ValidatedData::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_def_id,
                } => proc
                    .take_from_worktop(amount, resource_def_id)
                    .map(|bucket_id| {
                        ValidatedData::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_def_id,
                } => proc
                    .take_non_fungibles_from_worktop(ids, resource_def_id)
                    .map(|bucket_id| {
                        ValidatedData::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::AddToWorktop { bucket_id } => proc.add_to_worktop(bucket_id),
                ValidatedInstruction::AssertWorktop { resource_def_id } => {
                    proc.assert_worktop(resource_def_id)
                }
                ValidatedInstruction::AssertWorktopByAmount {
                    amount,
                    resource_def_id,
                } => proc.assert_worktop_by_amount(amount, resource_def_id),
                ValidatedInstruction::AssertWorktopByIds {
                    ids,
                    resource_def_id,
                } => proc.assert_worktop_by_ids(&ids, resource_def_id),
                ValidatedInstruction::TakeFromAuthZone {} => proc
                    .pop_from_auth_zone()
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::ClearAuthZone => proc
                    .drop_all_auth_zone_proofs()
                    .map(|_| ValidatedData::from_value(&())),
                ValidatedInstruction::AddToAuthZone { proof_id } => proc
                    .push_onto_auth_zone(proof_id)
                    .map(|_| ValidatedData::from_value(&())),
                ValidatedInstruction::CreateProofFromAuthZone { resource_def_id } => proc
                    .create_auth_zone_proof(resource_def_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_def_id,
                } => proc
                    .create_auth_zone_proof_by_amount(amount, resource_def_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_def_id,
                } => proc
                    .create_auth_zone_proof_by_ids(&ids, resource_def_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromBucket { bucket_id } => proc
                    .create_bucket_proof(bucket_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CloneProof { proof_id } => proc
                    .clone_proof(proof_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::DropProof { proof_id } => proc
                    .drop_proof(proof_id)
                    .map(|_| ValidatedData::from_value(&())),
                ValidatedInstruction::CallFunction {
                    package_id,
                    blueprint_name,
                    function,
                    args,
                } => proc.call_function(package_id, &blueprint_name, &function, args),
                ValidatedInstruction::CallMethod {
                    component_id,
                    method,
                    args,
                } => proc.call_method(component_id, &method, args),
                ValidatedInstruction::CallMethodWithAllResources {
                    component_id,
                    method,
                } => proc.call_method_with_all_resources(component_id, &method),
                ValidatedInstruction::PublishPackage { code } => proc.publish_package(code),
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

        // drop all dangling proofs
        error = error.or_else(|| match proc.drop_all_proofs() {
            Ok(_) => None,
            Err(e) => Some(e),
        });

        // check resource
        error = error.or_else(|| match proc.check_resource() {
            Ok(_) => None,
            Err(e) => Some(e),
        });

        // prepare data for receipts
        let new_package_ids = track.new_package_ids();
        let new_component_ids = track.new_component_ids();
        let new_resource_def_ids = track.new_resource_def_ids();
        let logs = track.logs().clone();

        // commit state updates
        let commit_receipt = if error.is_none() {
            let receipt = track.commit();
            self.substate_store.increase_nonce();
            Some(receipt)
        } else {
            None
        };

        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());

        Receipt {
            commit_receipt,
            transaction,
            result: match error {
                Some(error) => Err(error),
                None => Ok(()),
            },
            outputs,
            logs,
            new_package_ids,
            new_component_ids,
            new_resource_def_ids,
            execution_time,
        }
    }
}
