use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt, TransactionResult,
};
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::data::manifest::model::ManifestExpression;
use radix_engine_interface::rule;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use radix_engine_stores::rocks_db_with_merkle_tree::RocksDBWithMerkleTreeSubstateStore;
use scrypto::api::node_modules::ModuleConfig;
use scrypto::prelude::metadata;
use scrypto::prelude::metadata_init;
use scrypto::prelude::LOCKED;
use std::path::{Path, PathBuf};
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionManifestV1;
use transaction::model::{Executable, TestTransaction};
use transaction::signing::secp256k1::Secp256k1PrivateKey;

use crate::Compile;

// Basic RocksDB test runner for benchmark purpose.
pub struct BasicRocksdbTestRunner {
    scrypto_interpreter: ScryptoVm<DefaultWasmEngine>,
    substate_db: RocksDBWithMerkleTreeSubstateStore,
    next_private_key: u64,
    next_transaction_nonce: u32,
    trace: bool,
}

impl BasicRocksdbTestRunner {
    pub fn new(root: PathBuf, trace: bool) -> Self {
        let scrypto_interpreter = ScryptoVm::default();
        let mut substate_db = RocksDBWithMerkleTreeSubstateStore::standard(root);

        let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false);
        bootstrapper.bootstrap_test_default().unwrap();

        Self {
            scrypto_interpreter,
            substate_db,
            next_private_key: 1,
            next_transaction_nonce: 1,
            trace,
        }
    }

    pub fn faucet_component(&self) -> GlobalAddress {
        FAUCET.clone().into()
    }

    pub fn substate_db(&self) -> &RocksDBWithMerkleTreeSubstateStore {
        &self.substate_db
    }

    pub fn substate_db_mut(&mut self) -> &mut RocksDBWithMerkleTreeSubstateStore {
        &mut self.substate_db
    }

    pub fn next_private_key(&mut self) -> u64 {
        self.next_private_key += 1;
        self.next_private_key - 1
    }

    pub fn next_transaction_nonce(&mut self) -> u32 {
        self.next_transaction_nonce += 1;
        self.next_transaction_nonce - 1
    }

    pub fn new_key_pair(&mut self) -> (Secp256k1PublicKey, Secp256k1PrivateKey) {
        let private_key = Secp256k1PrivateKey::from_u64(self.next_private_key()).unwrap();
        let public_key = private_key.public_key();

        (public_key, private_key)
    }

    pub fn new_key_pair_with_auth_address(
        &mut self,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, NonFungibleGlobalId) {
        let key_pair = self.new_allocated_account();
        (
            key_pair.0,
            key_pair.1,
            NonFungibleGlobalId::from_public_key(&key_pair.0),
        )
    }

    pub fn load_account_from_faucet(&mut self, account_address: ComponentAddress) {
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 500u32.into())
            .call_method(self.faucet_component(), "free", manifest_args!())
            .take_all_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(
                    account_address,
                    ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                    manifest_args!(bucket),
                )
            })
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }

    pub fn new_account_advanced(&mut self, owner_rule: OwnerRole) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 500u32.into())
            .new_account_advanced(owner_rule)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();

        let account = receipt.expect_commit(true).new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 500u32.into())
            .call_method(self.faucet_component(), "free", manifest_args!())
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();

        account
    }

    pub fn new_virtual_account(
        &mut self,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        let (pub_key, priv_key) = self.new_key_pair();
        let account = ComponentAddress::virtual_account_from_public_key(&PublicKey::Secp256k1(
            pub_key.clone(),
        ));
        self.load_account_from_faucet(account);
        (pub_key, priv_key, account)
    }

    pub fn new_allocated_account(
        &mut self,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        let key_pair = self.new_key_pair();
        let withdraw_auth = rule!(require(NonFungibleGlobalId::from_public_key(&key_pair.0)));
        let account = self.new_account_advanced(OwnerRole::Fixed(withdraw_auth));
        (key_pair.0, key_pair.1, account)
    }

    pub fn new_account(
        &mut self,
        is_virtual: bool,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        if is_virtual {
            self.new_virtual_account()
        } else {
            self.new_allocated_account()
        }
    }

    pub fn publish_package(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: BTreeMap<String, MetadataValue>,
        owner_rule: OwnerRole,
    ) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 5000u32.into())
            .publish_package_advanced(None, code, definition, metadata, owner_rule)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        owner_badge: NonFungibleGlobalId,
    ) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 5000u32.into())
            .publish_package_with_owner(code, definition, owner_badge)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    pub fn compile<P: AsRef<Path>>(&mut self, package_dir: P) -> (Vec<u8>, PackageDefinition) {
        Compile::compile(package_dir)
    }

    pub fn compile_and_publish<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        let (code, definition) = Compile::compile(package_dir);
        self.publish_package(code, definition, BTreeMap::new(), OwnerRole::None)
    }

    pub fn compile_and_publish_with_owner<P: AsRef<Path>>(
        &mut self,
        package_dir: P,
        owner_badge: NonFungibleGlobalId,
    ) -> PackageAddress {
        let (code, definition) = Compile::compile(package_dir);
        self.publish_package_with_owner(code, definition, owner_badge)
    }

    pub fn execute_manifest<T>(
        &mut self,
        manifest: TransactionManifestV1,
        initial_proofs: T,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        let nonce = self.next_transaction_nonce();
        self.execute_transaction(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            FeeReserveConfig::default(),
            ExecutionConfig::for_test_transaction(),
        )
    }

    pub fn execute_transaction(
        &mut self,
        executable: Executable,
        fee_reserve_config: FeeReserveConfig,
        mut execution_config: ExecutionConfig,
    ) -> TransactionReceipt {
        // Override the kernel trace config
        execution_config = execution_config.with_kernel_trace(self.trace);

        let transaction_receipt = execute_transaction(
            &mut self.substate_db,
            &self.scrypto_interpreter,
            &fee_reserve_config,
            &execution_config,
            &executable,
        );
        if let TransactionResult::Commit(commit) = &transaction_receipt.transaction_result {
            self.substate_db
                .commit(&commit.state_updates.database_updates);
        }
        transaction_receipt
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceAction::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceAction::Deposit, (rule!(allow_all), LOCKED));
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 500u32.into())
            .create_fungible_resource(
                OwnerRole::None,
                true,
                divisibility,
                metadata!(),
                access_rules,
                Some(amount),
            )
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }
}
