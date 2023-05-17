#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;
#[cfg(feature = "libfuzzer-sys")]
use once_cell::sync::Lazy;

#[cfg(feature = "afl")]
use afl::fuzz;
#[cfg(feature = "afl")]
use std::panic::AssertUnwindSafe;

#[cfg(feature = "simple-fuzzer")]
mod simple_fuzzer;

use radix_engine::types::{ComponentAddress, EcdsaSecp256k1PublicKey, ResourceAddress};
use radix_engine_interface::blueprints::resource::{FromPublicKey, NonFungibleGlobalId};
use radix_engine_interface::data::manifest::manifest_decode;
use scrypto_unit::{TestRunner, TestRunnerSnapshot};
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::Instruction;
use transaction::model::TransactionManifest;

struct Account {
    public_key: EcdsaSecp256k1PublicKey,
    _private_key: EcdsaSecp256k1PrivateKey,
    address: ComponentAddress,
}

struct Fuzzer {
    runner: TestRunner,
    snapshot: TestRunnerSnapshot,
    accounts: Vec<Account>,
    resources: Vec<ResourceAddress>,
}

impl Fuzzer {
    fn new() -> Self {
        let mut runner = TestRunner::builder().without_trace().build();
        let accounts: Vec<Account> = (0..2)
            .map(|_| {
                let acc = runner.new_account(false);
                println!("addr = {:?}", acc.2);
                Account {
                    public_key: acc.0,
                    _private_key: acc.1,
                    address: acc.2,
                }
            })
            .collect();
        let resources: Vec<ResourceAddress> = vec![
            runner.create_fungible_resource(1000.into(), 18, accounts[0].address),
            runner.create_non_fungible_resource(accounts[0].address),
        ];

        let snapshot = runner.create_snapshot();

        println!("resources = {:?}", resources);

        Self {
            runner,
            snapshot,
            accounts,
            resources,
        }
    }

    fn reset_runner(&mut self) {
        self.runner.restore_snapshot(self.snapshot.clone());
    }

    // pick account from the preallocated pool basing on the input data
    fn get_account(&mut self, data: &[u8]) -> Option<ComponentAddress> {
        let len = data.len();
        if len >= 2 && data[len - 2] % 2 == 0 {
            let idx = *data.last().unwrap() as usize % self.accounts.len();
            return Some(self.accounts[idx].address);
        }
        None
    }

    // pick resource from the preallocated pool basing on the input data
    fn get_resource(&mut self, data: &[u8]) -> Option<ResourceAddress> {
        let len = data.len();
        if len >= 2 && data[len - 2] % 2 == 0 {
            let idx = *data.last().unwrap() as usize % self.accounts.len();
            return Some(self.resources[idx]);
        }
        None
    }

    // Smartly replace some data in the manifest using some preallocated resources.
    // This is to let fuzzing go "deeper" into the manifest instructions and not to reject the
    // transaction on the very early stage
    fn smart_mutate_manifest(&mut self, manifest: &mut TransactionManifest) {
        for i in &mut manifest.instructions {
            match i {
                Instruction::CallMethod { address, .. } => {
                    if let Some(account) = self.get_account(address.as_ref()) {
                        *address = account.into();
                    }
                }
                Instruction::TakeAllFromWorktop { resource_address }
                | Instruction::TakeFromWorktop {
                    resource_address, ..
                }
                | Instruction::TakeNonFungiblesFromWorktop {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContains {
                    resource_address, ..
                }
                | Instruction::AssertWorktopContainsNonFungibles {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZone { resource_address }
                | Instruction::CreateProofFromAuthZoneOfAmount {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZoneOfNonFungibles {
                    resource_address, ..
                }
                | Instruction::CreateProofFromAuthZoneOfAll {
                    resource_address, ..
                } => {
                    if let Some(address) = self.get_resource(resource_address.as_ref()) {
                        *resource_address = address;
                    }
                }
                _ => {}
            }
        }
    }

    fn fuzz_tx_manifest(&mut self, data: &[u8]) -> TxStatus {
        let result = manifest_decode::<TransactionManifest>(data);
        match result {
            Ok(mut manifest) => {
                self.smart_mutate_manifest(&mut manifest);
                let receipt = self.runner.execute_manifest(
                    manifest,
                    vec![NonFungibleGlobalId::from_public_key(
                        &self.accounts[0].public_key,
                    )],
                );
                if receipt.is_commit_success() {
                    TxStatus::CommitSuccess
                } else {
                    TxStatus::CommitFailure
                }
            }
            Err(_err) => TxStatus::DecodeError,
        }
    }
}

#[derive(Debug)]
pub enum TxStatus {
    // Transaction commit success
    CommitSuccess,
    // Transaction commit failure
    CommitFailure,
    // Transaction manifest parse error
    DecodeError,
}

#[test]
// This test verifies whether it is still possible to parse manifest raw files and execute them.
// If it fails with TxStatus::DecodeError then most likely that manifest format has changed and
// input files shall be recreated.
fn test_fuzz_tx() {
    let mut fuzzer = Fuzzer::new();
    let data = std::fs::read(
        "fuzz_input/transaction/manifest_01995e0d6005c34ad99fba993ebe1443ef55c4db71ed037de12afb3eb28bbfae.raw",
    )
    .unwrap();
    assert!(matches!(
        fuzzer.fuzz_tx_manifest(&data),
        TxStatus::CommitSuccess
    ));

    let data = std::fs::read(
        "fuzz_input/transaction//manifest_0113970c0a72935c8c27ddd97a9396d1839f0173bf9ed091f9706aa61db8417e.raw",
    )
    .unwrap();
    assert!(matches!(
        fuzzer.fuzz_tx_manifest(&data),
        TxStatus::CommitFailure
    ));
}

// Initialize static objects outside the fuzzing loop to assure deterministic instrumentation
// output across runs.
fn init_statics() {
    // Following code initializes secp256k1::SECP256K1 global static context
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(100).unwrap();
    let _public_key = private_key.public_key();
}

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: &[u8]| {
    unsafe {
        static mut FUZZER: Lazy<Fuzzer> = Lazy::new(|| Fuzzer::new());

        FUZZER.reset_runner();
        FUZZER.fuzz_tx_manifest(data);
    }
});

#[cfg(feature = "afl")]
fn main() {
    init_statics();

    // fuzz! uses `catch_unwind` and it requires RefUnwindSafe trait, which is not auto-implemented by
    // Fuzzer members (TestRunner mainly). `AssertUnwindSafe` annotates the variable is indeed
    // unwind safe
    let mut fuzzer = AssertUnwindSafe(Fuzzer::new());

    fuzz!(|data: &[u8]| {
        fuzzer.reset_runner();
        fuzzer.fuzz_tx_manifest(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    init_statics();

    let mut fuzzer = Fuzzer::new();

    simple_fuzzer::fuzz(|data: &[u8]| -> TxStatus {
        fuzzer.reset_runner();
        fuzzer.fuzz_tx_manifest(data)
    });
}
