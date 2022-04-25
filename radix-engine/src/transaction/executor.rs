use scrypto::crypto::hash;
use scrypto::engine::types::*;
use scrypto::resource::*;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::rust::string::ToString;
use scrypto::{abi, rule, access_rule_node};

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

impl<'l, L: SubstateStore> NonceProvider for TransactionExecutor<'l, L> {
    fn get_nonce<PKS: AsRef<[EcdsaPublicKey]>>(&self, _intended_signers: PKS) -> u64 {
        self.substate_store.get_nonce()
    }
}

impl<'l, L: SubstateStore> AbiProvider for TransactionExecutor<'l, L> {
    fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let package: Package = self
            .substate_store
            .get_decoded_substate(&package_address)
            .map(|(package, _)| package)
            .ok_or(RuntimeError::PackageNotFound(package_address))?;

        BasicAbiProvider::new(self.trace)
            .with_package(&package_address, package)
            .export_abi(package_address, blueprint_name)
    }

    fn export_abi_by_component(
        &self,
        component_address: ComponentAddress,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let component: Component = self
            .substate_store
            .get_decoded_substate(&component_address)
            .map(|(component, _)| component)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;
        let package: Package = self
            .substate_store
            .get_decoded_substate(&component.package_address())
            .map(|(package, _)| package)
            .unwrap();
        BasicAbiProvider::new(self.trace)
            .with_package(&component.package_address(), package)
            .export_abi(component.package_address(), component.blueprint_name())
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

    /// Generates a new key pair.
    pub fn new_key_pair(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey) {
        let private_key = EcdsaPrivateKey::from_bytes(
            hash(self.substate_store.get_and_increase_nonce().to_le_bytes()).as_ref(),
        )
        .unwrap();
        let public_key = private_key.public_key();
        (public_key, private_key)
    }

    /// Creates an account with 1,000,000 XRD in balance.
    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &AccessRule) -> ComponentAddress {
        let receipt = self
            .validate_and_execute(
                &TransactionBuilder::new()
                    .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
                    .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                        builder.new_account_with_resource(withdraw_auth, bucket_id)
                    })
                    .build(self.get_nonce([]))
                    .sign([]),
            )
            .unwrap();

        receipt.result.expect("Should be okay");
        receipt.new_component_addresses[0]
    }

    /// Creates a new key and an account which can be accessed using the key.
    pub fn new_account(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) {
        let (public_key, private_key) = self.new_key_pair();
        let id = NonFungibleId::from_bytes(public_key.to_vec());
        let auth_address = NonFungibleAddress::new(ECDSA_TOKEN, id);
        let withdraw_auth = rule!(require(auth_address));
        let account = self.new_account_with_auth_rule(&withdraw_auth);
        (public_key, private_key, account)
    }

    /// Publishes a package.
    pub fn publish_package<T: AsRef<[u8]>>(
        &mut self,
        code: T,
    ) -> Result<PackageAddress, RuntimeError> {
        let receipt = self
            .validate_and_execute(
                &TransactionBuilder::new()
                    .publish_package(code.as_ref())
                    .build(self.get_nonce([]))
                    .sign([]),
            )
            .unwrap();

        if receipt.result.is_ok() {
            Ok(receipt.new_package_addresses[0])
        } else {
            Err(receipt.result.err().unwrap())
        }
    }

    /// Overwrites a package.
    pub fn overwrite_package(
        &mut self,
        package_address: PackageAddress,
        code: Vec<u8>,
    ) -> Result<(), WasmValidationError> {
        let tx_hash = hash(self.substate_store.get_and_increase_nonce().to_le_bytes());
        let mut id_gen = SubstateIdGenerator::new(tx_hash);

        let package = Package::new(code)?;
        self.substate_store
            .put_encoded_substate(&package_address, &package, id_gen.next());
        Ok(())
    }

    pub fn validate_and_execute(
        &mut self,
        signed: &SignedTransaction,
    ) -> Result<Receipt, TransactionValidationError> {
        let validated = signed.validate()?;
        let receipt = self.execute(validated);
        Ok(receipt)
    }

    pub fn execute(&mut self, validated: ValidatedTransaction) -> Receipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let mut track = Track::new(
            self.substate_store,
            validated.raw_hash.clone(),
            validated.signers.clone(),
        );
        let mut proc = track.start_process(self.trace);

        let txn_process = TransactionProcess::new(validated.clone());
        let mut txn_snode = SNodeState::Transaction(txn_process);
        let error = match proc.run(&mut txn_snode, "execute".to_string(), vec![]) {
            Ok(_) => None,
            Err(e) => Some(e),
        };
        let outputs = if let SNodeState::Transaction(txn_process) = txn_snode {
            txn_process.outputs().to_vec()
        } else {
            panic!("Should not get here");
        };

        // prepare data for receipts
        let new_package_addresses = track.new_package_addresses();
        let new_component_addresses = track.new_component_addresses();
        let new_resource_addresses = track.new_resource_addresses();
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
            validated_transaction: validated.clone(),
            result: match error {
                Some(error) => Err(error),
                None => Ok(()),
            },
            outputs,
            logs,
            new_package_addresses,
            new_component_addresses,
            new_resource_addresses,
            execution_time,
        }
    }
}
