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

use radix_engine::types::{ComponentAddress, EcdsaSecp256k1PublicKey};
use radix_engine_interface::blueprints::resource::{FromPublicKey, NonFungibleGlobalId};
use scrypto_unit::TestRunner;
use transaction::model::TransactionManifest;
use transaction::signing::EcdsaSecp256k1PrivateKey;

struct Account {
    public_key: EcdsaSecp256k1PublicKey,
    private_key: EcdsaSecp256k1PrivateKey,
    address: ComponentAddress,
}

struct Fuzzer {
    runner: TestRunner,
    accounts: Vec<Account>,
    //    fungible_resource: [ResourceAddress; 2],
    //    non_fungible_resource: [ResourceAddress; 2],
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
                    private_key: acc.1,
                    address: acc.2,
                }
            })
            .collect();

        Self { runner, accounts }
    }

    fn fuzz_tx_manifest(&mut self, data: &[u8]) -> TxStatus {
        let result = TransactionManifest::from_slice(data);
        match result {
            Ok(manifest) => {
                let _receipt = self.runner.execute_manifest(
                    manifest,
                    vec![NonFungibleGlobalId::from_public_key(
                        &self.accounts[0].public_key,
                    )],
                );

                TxStatus::Ok
            }
            Err(_err) => {
                //println!("manifest decoding error {:?}", err);
                TxStatus::Error
            }
        }
    }
}

enum TxStatus {
    // TransactionIntent successfully parsed
    Ok,
    // TransactionIntent parse error
    Error,
}

#[test]
fn test_fuzz_tx() {
    let mut fuzzer = Fuzzer::new();
    let data = std::fs::read(
        "afl_in/manifest_e057a3853ccb0e33c8b61f2cde91f655473b202c6c095e2202c2ad93caee4e34.raw",
    )
    .unwrap();
    fuzzer.fuzz_tx_manifest(&data);
}

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: &[u8]| {
    unsafe {
        static mut FUZZER: Lazy<Fuzzer> = Lazy::new(|| Fuzzer::new());

        FUZZER.fuzz_tx_manifest(data);
    }
});

#[cfg(feature = "afl")]
fn main() {
    // fuzz! uses `catch_unwind` and it requires RefUnwindSafe trait, which is not auto-implemented by
    // Fuzzer members (TestRunner mainly). `AssertUnwindSafe` annotates the variable is indeed
    // unwind safe
    let mut fuzzer = AssertUnwindSafe(Fuzzer::new());

    fuzz!(|data: &[u8]| {
        fuzzer.fuzz_tx_manifest(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    let mut fuzzer = Fuzzer::new();

    simple_fuzzer::fuzz(|data: &[u8]| {
        fuzzer.fuzz_tx_manifest(data);
    });
}
