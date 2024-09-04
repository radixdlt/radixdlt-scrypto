use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::prelude::{FromPublicKey, NonFungibleGlobalId};
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::LedgerSimulatorBuilder;

#[test]
fn lock_fee_from_faucet_error_injection() {
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let mut inject_err_after_count = 1u64;

    loop {
        let manifest = ManifestBuilder::new().lock_fee_from_faucet().build();
        let receipt =
            ledger.execute_manifest_with_injected_error(manifest, vec![], inject_err_after_count);
        if receipt.is_commit_success() {
            break;
        }

        inject_err_after_count += 1u64;
    }

    println!("Count: {:?}", inject_err_after_count);
}

#[test]
fn lock_fee_from_faucet_twice_error_injection() {
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let mut inject_err_after_count = 1u64;

    // TODO: Use state from InjectSystemCostingError to tell when we haven't progressed
    for _ in 0..600 {
        let manifest = ManifestBuilder::new().lock_fee_from_faucet().build();
        let receipt =
            ledger.execute_manifest_with_injected_error(manifest, vec![], inject_err_after_count);
        if receipt.is_commit_success() {
            break;
        }

        inject_err_after_count += 1u64;
    }

    println!("Count: {:?}", inject_err_after_count);
}

#[test]
fn lock_fee_from_faucet_and_account_error_injection() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_account(false);

    let mut inject_err_after_count = 1u64;

    loop {
        let manifest = ManifestBuilder::new()
            .lock_fee(account, dec!("500"))
            .build();
        let receipt = ledger.execute_manifest_with_injected_error(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&pub_key)],
            inject_err_after_count,
        );
        if receipt.is_commit_success() {
            break;
        }

        inject_err_after_count += 1u64;
    }

    println!("Count: {:?}", inject_err_after_count);
}
