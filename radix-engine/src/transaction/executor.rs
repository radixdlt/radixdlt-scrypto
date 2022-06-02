use sbor::rust::marker::PhantomData;
use sbor::rust::vec::Vec;
use sbor::rust::string::ToString;
use scrypto::buffer::*;
use scrypto::component::Package;
use scrypto::crypto::hash;
use scrypto::engine::types::*;
use scrypto::resource::*;
use scrypto::values::ScryptoValue;
use scrypto::{abi, access_rule_node, rule, to_struct};

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;
use crate::transaction::abi_extractor::{export_abi, export_abi_by_component};
use crate::transaction::*;
use crate::wasm::*;

/// An executor that runs transactions.
pub struct TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    trace: bool,
    phantom: PhantomData<I>,
}

impl<'s, 'w, S, W, I> NonceProvider for TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn get_nonce<PKS: AsRef<[EcdsaPublicKey]>>(&self, _intended_signers: PKS) -> u64 {
        self.substate_store.get_nonce()
    }
}

impl<'s, 'w, S, W, I> TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(
        substate_store: &'s mut S,
        wasm_engine: &'w mut W,
        trace: bool,
    ) -> TransactionExecutor<'s, 'w, S, W, I> {
        Self {
            substate_store,
            wasm_engine,
            trace,
            phantom: PhantomData,
        }
    }

    /// Returns an immutable reference to the ledger.
    pub fn substate_store(&self) -> &S {
        self.substate_store
    }

    /// Returns a mutable reference to the ledger.
    pub fn substate_store_mut(&mut self) -> &mut S {
        self.substate_store
    }

    pub fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<abi::BlueprintAbi, RuntimeError> {
        export_abi(self.substate_store, package_address, blueprint_name)
    }

    pub fn export_abi_by_component(
        &self,
        component_address: ComponentAddress,
    ) -> Result<abi::BlueprintAbi, RuntimeError> {
        export_abi_by_component(self.substate_store, component_address)
    }

    /// Generates a new key pair.
    pub fn new_key_pair(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey) {
        let nonce = self.substate_store.get_nonce();
        self.substate_store.increase_nonce();
        let private_key = EcdsaPrivateKey::from_bytes(hash(nonce.to_le_bytes()).as_ref()).unwrap();
        let public_key = private_key.public_key();
        (public_key, private_key)
    }

    /// Creates an account with 1,000,000 XRD in balance.
    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &AccessRule) -> ComponentAddress {
        let receipt = self
            .validate_and_execute(
                &TransactionBuilder::new()
                    .call_method(SYSTEM_COMPONENT, "free_xrd", to_struct!())
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
    pub fn publish_package(&mut self, package: Package) -> Result<PackageAddress, RuntimeError> {
        let receipt = self
            .validate_and_execute(
                &TransactionBuilder::new()
                    .publish_package(package)
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

        // Start state track
        let mut track = Track::new(self.substate_store, validated.raw_hash.clone());

        // Create root call frame.
        let mut root_frame = CallFrame::new_root(
            self.trace,
            validated.raw_hash.clone(),
            validated.signers.clone(),
            &mut track,
            self.wasm_engine,
        );

        // Invoke the transaction processor
        // TODO: may consider moving transaction parsing to `TransactionProcessor` as well.
        let result = root_frame.invoke_snode(
            scrypto::core::SNodeRef::TransactionProcessor,
            "run".to_string(),
            ScryptoValue::from_value(&TransactionProcessorRunInput {
                transaction: validated.clone(),
            }),
        );

        let (outputs, error) = match result {
            Ok(o) => (scrypto_decode::<Vec<ScryptoValue>>(&o.raw).unwrap(), None),
            Err(e) => (Vec::<ScryptoValue>::new(), Some(e)),
        };

        let track_receipt = track.to_receipt();

        // commit state updates
        let commit_receipt = if error.is_none() {
            if !track_receipt.borrowed.is_empty() {
                panic!("There should be nothing borrowed by end of transaction.");
            }
            let commit_receipt = track_receipt.substates.commit(self.substate_store);
            self.substate_store.increase_nonce();
            Some(commit_receipt)
        } else {
            None
        };

        let mut new_component_addresses = Vec::new();
        let mut new_resource_addresses = Vec::new();
        let mut new_package_addresses = Vec::new();
        for address in track_receipt.new_addresses {
            match address {
                Address::Component(component_address) => {
                    new_component_addresses.push(component_address)
                }
                Address::Resource(resource_address) => {
                    new_resource_addresses.push(resource_address)
                }
                Address::Package(package_address) => new_package_addresses.push(package_address),
                _ => {}
            }
        }

        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());

        Receipt {
            commit_receipt,
            validated_transaction: validated,
            result: match error {
                Some(error) => Err(error),
                None => Ok(()),
            },
            outputs,
            logs: track_receipt.logs,
            new_package_addresses,
            new_component_addresses,
            new_resource_addresses,
            execution_time,
        }
    }
}
