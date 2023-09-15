use radix_engine::vm::NoExtension;
use radix_engine_common::dec;
use radix_engine_interface::prelude::{FromPublicKey, NonFungibleGlobalId};
use scrypto_unit::{InjectSystemCostingError, TestRunnerBuilder};
use transaction::builder::ManifestBuilder;

#[test]
fn lock_fee_from_faucet_error_injection() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let mut inject_err_after_count = 1u64;

    loop {
        let manifest = ManifestBuilder::new().lock_fee_from_faucet().build();
        let receipt = test_runner
            .execute_manifest_with_system::<_, InjectSystemCostingError<'_, NoExtension>>(
                manifest,
                vec![],
                inject_err_after_count,
            );
        if receipt.is_commit_success() {
            break;
        }

        inject_err_after_count += 1u64;
    }

    println!("Count: {:?}", inject_err_after_count);
}

#[test]
fn lock_fee_from_faucet_twice_error_injection() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let mut inject_err_after_count = 1u64;

    // TODO: Use state from InjectSystemCostingError to tell when we haven't progressed
    for _ in 0..600 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .lock_fee_from_faucet()
            .build();
        let receipt = test_runner
            .execute_manifest_with_system::<_, InjectSystemCostingError<'_, NoExtension>>(
                manifest,
                vec![],
                inject_err_after_count,
            );
        if receipt.is_commit_success() {
            break;
        }

        inject_err_after_count += 1u64;
    }

    println!("Count: {:?}", inject_err_after_count);
}

#[test]
fn lock_fee_from_faucet_and_account_error_injection() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pub_key, _, account) = test_runner.new_account(false);

    let mut inject_err_after_count = 1u64;

    loop {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .lock_fee(account, dec!("500"))
            .build();
        let receipt = test_runner
            .execute_manifest_with_system::<_, InjectSystemCostingError<'_, NoExtension>>(
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
