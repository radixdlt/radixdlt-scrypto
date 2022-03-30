use sbor::DecodeError;
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::sha256;
use scrypto::engine::types::*;
use scrypto::prelude::NonFungibleAddress;
use scrypto::resource::ProofRule;
use scrypto::rust::vec;
use scrypto::{abi, this};

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

    /// Generates a new private key.
    pub fn new_private_key(&mut self) -> EcdsaPrivateKey {
        EcdsaPrivateKey(sha256(self.substate_store.next_nonce().to_le_bytes()).0)
    }

    /// Creates an account with 1,000,000 XRD in balance.
    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &ProofRule) -> ComponentId {
        let tx_nonce = self.substate_store.next_nonce();
        self.validate_and_execute(
            &TransactionBuilder::new(self)
                .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
                .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                    builder.new_account_with_resource(withdraw_auth, bucket_id)
                })
                .build(Vec::new(), tx_nonce)
                .unwrap(),
        )
        .unwrap()
        .new_component_ids[0]
    }

    /// Creates a new private key and an account which can be accessed using the private key.
    pub fn new_account(&mut self) -> (EcdsaPrivateKey, EcdsaPublicKey, ComponentId) {
        let private_key = self.new_private_key();
        let public_key = private_key.public_key();
        let id = NonFungibleId::new(public_key.to_vec());
        let auth_address = NonFungibleAddress::new(ECDSA_TOKEN, id);
        let withdraw_auth = this!(auth_address);
        let account = self.new_account_with_auth_rule(&withdraw_auth);
        (private_key, public_key, account)
    }

    /// Publishes a package.
    pub fn publish_package<T: AsRef<[u8]>>(&mut self, code: T) -> Result<PackageId, RuntimeError> {
        let tx_nonce = self.substate_store.next_nonce();
        let receipt = self
            .validate_and_execute(
                &TransactionBuilder::new(self)
                    .publish_package(code.as_ref())
                    .build(Vec::new(), tx_nonce)
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
        let tx_hash = sha256(self.substate_store.next_nonce().to_le_bytes());
        let mut id_gen = SubstateIdGenerator::new(tx_hash);

        let package = Package::new(code.to_vec());
        self.substate_store
            .put_encoded_substate(&package_id, &package, id_gen.next());
    }

    pub fn validate_and_execute(
        &mut self,
        transaction: &Transaction,
    ) -> Result<Receipt, TransactionValidationError> {
        let validated_transaction = self.validate(transaction)?;
        let receipt = self.execute(&validated_transaction);
        Ok(receipt)
    }

    pub fn parse<T: AsRef<[u8]>>(&mut self, transaction: T) -> Result<Transaction, DecodeError> {
        scrypto_decode(transaction.as_ref())
    }

    pub fn validate(
        &mut self,
        transaction: &Transaction,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        validate_transaction(transaction)
    }

    pub fn execute(&mut self, transaction: &ValidatedTransaction) -> Receipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let mut track = Track::new(
            self.substate_store,
            transaction.hash.clone(),
            transaction.signers.clone(),
        );
        let mut proc = track.start_process(self.trace);

        let mut error: Option<RuntimeError> = None;
        let mut outputs = vec![];
        for inst in &transaction.instructions {
            let result = match inst {
                ValidatedInstruction::TakeFromWorktop { resource_def_id } => proc
                    .take_all_from_worktop(*resource_def_id)
                    .map(|bucket_id| {
                        ValidatedData::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_def_id,
                } => proc
                    .take_from_worktop(*amount, *resource_def_id)
                    .map(|bucket_id| {
                        ValidatedData::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_def_id,
                } => proc
                    .take_non_fungibles_from_worktop(ids, *resource_def_id)
                    .map(|bucket_id| {
                        ValidatedData::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::ReturnToWorktop { bucket_id } => {
                    proc.return_to_worktop(*bucket_id)
                }
                ValidatedInstruction::AssertWorktopContains { resource_def_id } => {
                    proc.assert_worktop_contains(*resource_def_id)
                }
                ValidatedInstruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_def_id,
                } => proc.assert_worktop_contains_by_amount(*amount, *resource_def_id),
                ValidatedInstruction::AssertWorktopContainsByIds {
                    ids,
                    resource_def_id,
                } => proc.assert_worktop_contains_by_ids(&ids, *resource_def_id),
                ValidatedInstruction::TakeFromAuthZone {} => proc
                    .take_from_auth_zone()
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::ClearAuthZone => proc
                    .drop_all_auth_zone_proofs()
                    .map(|_| ValidatedData::from_value(&())),
                ValidatedInstruction::MoveToAuthZone { proof_id } => proc
                    .move_to_auth_zone(*proof_id)
                    .map(|_| ValidatedData::from_value(&())),
                ValidatedInstruction::CreateProofFromAuthZone { resource_def_id } => proc
                    .create_auth_zone_proof(*resource_def_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_def_id,
                } => proc
                    .create_auth_zone_proof_by_amount(*amount, *resource_def_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_def_id,
                } => proc
                    .create_auth_zone_proof_by_ids(ids, *resource_def_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromBucket { bucket_id } => proc
                    .create_bucket_proof(*bucket_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CloneProof { proof_id } => proc
                    .clone_proof(*proof_id)
                    .map(|proof_id| ValidatedData::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::DropProof { proof_id } => proc
                    .drop_proof(*proof_id)
                    .map(|_| ValidatedData::from_value(&())),
                ValidatedInstruction::CallFunction {
                    package_id,
                    blueprint_name,
                    function,
                    args,
                } => proc.call_function(*package_id, &blueprint_name, &function, args.clone()),
                ValidatedInstruction::CallMethod {
                    component_id,
                    method,
                    args,
                } => proc.call_method(*component_id, &method, args.clone()),
                ValidatedInstruction::CallMethodWithAllResources {
                    component_id,
                    method,
                } => proc.call_method_with_all_resources(*component_id, &method),
                ValidatedInstruction::PublishPackage { code } => proc.publish_package(code.clone()),
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
            transaction: transaction.clone(),
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
