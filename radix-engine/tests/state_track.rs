#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::engine::SubstateReceipt;
use radix_engine::ledger::OutputId;
use scrypto::core::Network;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

macro_rules! substate_id {
    ($tx_hash:expr, $idx:expr) => {
        OutputId(Hash::from_str($tx_hash).unwrap(), $idx)
    };
}

#[test]
fn test_state_track_success() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
    let mut expected_downs = HashSet::new();
    expected_downs.extend([
        substate_id!(
            "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
            6
        ),
        substate_id!(
            "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
            7
        ),
        substate_id!(
            "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
            8
        ),
        substate_id!(
            "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
            4
        ),
        substate_id!(
            "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
            5
        ),
        substate_id!(
            "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
            6
        ),
        substate_id!(
            "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
            7
        ),
        substate_id!(
            "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
            8
        ),
    ]);
    let expected_ups = vec![
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            0
        ),
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            1
        ),
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            2
        ),
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            3
        ),
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            4
        ),
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            5
        ),
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            6
        ),
        substate_id!(
            "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            7
        ),
    ];
    assert_eq!(
        receipt.state_updates,
        SubstateReceipt {
            virtual_down_substates: HashSet::new(),
            down_substates: expected_downs,
            virtual_up_substates: Vec::new(),
            up_substates: expected_ups
        }
    )
}

#[test]
fn test_state_track_failure() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .assert_worktop_contains_by_amount(Decimal::from(5), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::WorktopError(_)));
    let expected_downs = HashSet::new();
    let expected_ups = vec![];
    assert_eq!(
        receipt.state_updates,
        SubstateReceipt {
            virtual_down_substates: HashSet::new(),
            down_substates: expected_downs,
            virtual_up_substates: Vec::new(),
            up_substates: expected_ups
        }
    )
}
