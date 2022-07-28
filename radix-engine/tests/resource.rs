#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::model::ResourceManagerError;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_resource_manager() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible",
            to_struct!(),
        )
        .call_function(package_address, "ResourceTest", "query", to_struct!())
        .call_function(package_address, "ResourceTest", "burn", to_struct!())
        .call_function(
            package_address,
            "ResourceTest",
            "update_resource_metadata",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            to_struct!(0u8, dec!("0.1")),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| {
        if let RuntimeError::ResourceManagerError(ResourceManagerError::InvalidAmount(
            amount,
            granularity,
        )) = e
        {
            amount.eq(&Decimal::from("0.1")) && *granularity == 0
        } else {
            false
        }
    });
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            to_struct!(0u8, dec!(100_000_000_001i128)),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| {
        matches!(
            e,
            RuntimeError::ResourceManagerError(ResourceManagerError::MaxMintAmountExceeded)
        )
    })
}

fn test_resource_behavior_internal(
    resource_type: ResourceType,
    behavior_to_add: ResourceMethodAuthKey,
    behavior_access_rule: AccessRule,
    mutability_access_rule: Mutability,
    behavior_to_check: ResourceMethodAuthKey,
    expected_output: (bool, bool),
) {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, _) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "ResourceBehaviorTest",
            "create_and_check_resource_behavior",
            to_struct!(
                resource_type,
                behavior_to_add,
                behavior_access_rule,
                mutability_access_rule,
                behavior_to_check
            ),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    let check_output: (bool, bool) = scrypto_decode(&receipt.result.as_ref().unwrap()[0]).unwrap();

    // Assert
    assert!(check_output == expected_output);
}

#[test]
fn test_non_mintable_resource_with_locked_mintability() {
    let (is_mintable, is_mintable_locked): (bool, bool) = (false, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Mint,
        rule!(deny_all),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Mint,
        (is_mintable, is_mintable_locked),
    )
}

#[test]
fn test_non_mintable_resource_with_mutable_mintability() {
    let (is_mintable, is_mintable_locked): (bool, bool) = (false, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Mint,
        rule!(deny_all),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Mint,
        (is_mintable, is_mintable_locked),
    )
}

#[test]
fn test_mintable_resource_with_locked_mintability() {
    let (is_mintable, is_mintable_locked): (bool, bool) = (true, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Mint,
        rule!(require(RADIX_TOKEN)),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Mint,
        (is_mintable, is_mintable_locked),
    )
}

#[test]
fn test_mintable_resource_with_mutable_mintability() {
    let (is_mintable, is_mintable_locked): (bool, bool) = (true, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Mint,
        rule!(require(RADIX_TOKEN)),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Mint,
        (is_mintable, is_mintable_locked),
    )
}

#[test]
fn test_non_burnable_resource_with_locked_burnability() {
    let (is_burnable, is_burnable_locked): (bool, bool) = (false, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Burn,
        rule!(deny_all),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Burn,
        (is_burnable, is_burnable_locked),
    )
}

#[test]
fn test_non_burnable_resource_with_mutable_burnability() {
    let (is_burnable, is_burnable_locked): (bool, bool) = (false, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Burn,
        rule!(deny_all),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Burn,
        (is_burnable, is_burnable_locked),
    )
}

#[test]
fn test_burnable_resource_with_locked_burnability() {
    let (is_burnable, is_burnable_locked): (bool, bool) = (true, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Burn,
        rule!(require(RADIX_TOKEN)),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Burn,
        (is_burnable, is_burnable_locked),
    )
}

#[test]
fn test_burnable_resource_with_mutable_burnability() {
    let (is_burnable, is_burnable_locked): (bool, bool) = (true, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Burn,
        rule!(require(RADIX_TOKEN)),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Burn,
        (is_burnable, is_burnable_locked),
    )
}

#[test]
fn test_non_restricted_withdraw_resource_with_locked_restricted_withdraw() {
    let (is_restricted_withdraw, is_restricted_withdraw_locked): (bool, bool) = (false, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Withdraw,
        rule!(deny_all),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Withdraw,
        (is_restricted_withdraw, is_restricted_withdraw_locked),
    )
}

#[test]
fn test_non_restricted_withdraw_resource_with_mutable_restricted_withdraw() {
    let (is_restricted_withdraw, is_restricted_withdraw_locked): (bool, bool) = (false, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Withdraw,
        rule!(deny_all),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Withdraw,
        (is_restricted_withdraw, is_restricted_withdraw_locked),
    )
}

#[test]
fn test_restricted_withdraw_resource_with_locked_restricted_withdraw() {
    let (is_restricted_withdraw, is_restricted_withdraw_locked): (bool, bool) = (true, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Withdraw,
        rule!(require(RADIX_TOKEN)),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Withdraw,
        (is_restricted_withdraw, is_restricted_withdraw_locked),
    )
}

#[test]
fn test_restricted_withdraw_resource_with_mutable_restricted_withdraw() {
    let (is_restricted_withdraw, is_restricted_withdraw_locked): (bool, bool) = (true, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Withdraw,
        rule!(require(RADIX_TOKEN)),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Withdraw,
        (is_restricted_withdraw, is_restricted_withdraw_locked),
    )
}

#[test]
fn test_non_restricted_deposit_resource_with_locked_restricted_deposit() {
    let (is_restricted_deposit, is_restricted_deposit_locked): (bool, bool) = (false, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Deposit,
        rule!(deny_all),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Deposit,
        (is_restricted_deposit, is_restricted_deposit_locked),
    )
}

#[test]
fn test_non_restricted_deposit_resource_with_mutable_restricted_deposit() {
    let (is_restricted_deposit, is_restricted_deposit_locked): (bool, bool) = (false, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Deposit,
        rule!(deny_all),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Deposit,
        (is_restricted_deposit, is_restricted_deposit_locked),
    )
}

#[test]
fn test_restricted_deposit_resource_with_locked_restricted_deposit() {
    let (is_restricted_deposit, is_restricted_deposit_locked): (bool, bool) = (true, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Deposit,
        rule!(require(RADIX_TOKEN)),
        Mutability::LOCKED,
        ResourceMethodAuthKey::Deposit,
        (is_restricted_deposit, is_restricted_deposit_locked),
    )
}

#[test]
fn test_restricted_deposit_resource_with_mutable_restricted_deposit() {
    let (is_restricted_deposit, is_restricted_deposit_locked): (bool, bool) = (true, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::Deposit,
        rule!(require(RADIX_TOKEN)),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::Deposit,
        (is_restricted_deposit, is_restricted_deposit_locked),
    )
}

#[test]
fn test_non_updatable_metadata_resource_with_locked_updatable_metadata() {
    let (is_updatable_metadata, is_updatable_metadata_locked): (bool, bool) = (false, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::UpdateMetadata,
        rule!(deny_all),
        Mutability::LOCKED,
        ResourceMethodAuthKey::UpdateMetadata,
        (is_updatable_metadata, is_updatable_metadata_locked),
    )
}

#[test]
fn test_non_updatable_metadata_resource_with_mutable_updatable_metadata() {
    let (is_updatable_metadata, is_updatable_metadata_locked): (bool, bool) = (false, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::UpdateMetadata,
        rule!(deny_all),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::UpdateMetadata,
        (is_updatable_metadata, is_updatable_metadata_locked),
    )
}

#[test]
fn test_updatable_metadata_resource_with_locked_updatable_metadata() {
    let (is_updatable_metadata, is_updatable_metadata_locked): (bool, bool) = (true, true);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::UpdateMetadata,
        rule!(require(RADIX_TOKEN)),
        Mutability::LOCKED,
        ResourceMethodAuthKey::UpdateMetadata,
        (is_updatable_metadata, is_updatable_metadata_locked),
    )
}

#[test]
fn test_updatable_metadata_resource_with_mutable_updatable_metadata() {
    let (is_updatable_metadata, is_updatable_metadata_locked): (bool, bool) = (true, false);

    test_resource_behavior_internal(
        ResourceType::Fungible { divisibility: 18 },
        ResourceMethodAuthKey::UpdateMetadata,
        rule!(require(RADIX_TOKEN)),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::UpdateMetadata,
        (is_updatable_metadata, is_updatable_metadata_locked),
    )
}

#[test]
fn test_non_non_fungible_data_resource_with_locked_non_fungible_data() {
    let (is_non_fungible_data, is_non_fungible_data_locked): (bool, bool) = (false, true);

    test_resource_behavior_internal(
        ResourceType::NonFungible,
        ResourceMethodAuthKey::UpdateNonFungibleData,
        rule!(deny_all),
        Mutability::LOCKED,
        ResourceMethodAuthKey::UpdateNonFungibleData,
        (is_non_fungible_data, is_non_fungible_data_locked),
    )
}

#[test]
fn test_non_non_fungible_data_resource_with_mutable_non_fungible_data() {
    let (is_non_fungible_data, is_non_fungible_data_locked): (bool, bool) = (false, false);

    test_resource_behavior_internal(
        ResourceType::NonFungible,
        ResourceMethodAuthKey::UpdateNonFungibleData,
        rule!(deny_all),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::UpdateNonFungibleData,
        (is_non_fungible_data, is_non_fungible_data_locked),
    )
}

#[test]
fn test_non_fungible_data_resource_with_locked_non_fungible_data() {
    let (is_non_fungible_data, is_non_fungible_data_locked): (bool, bool) = (true, true);

    test_resource_behavior_internal(
        ResourceType::NonFungible,
        ResourceMethodAuthKey::UpdateNonFungibleData,
        rule!(require(RADIX_TOKEN)),
        Mutability::LOCKED,
        ResourceMethodAuthKey::UpdateNonFungibleData,
        (is_non_fungible_data, is_non_fungible_data_locked),
    )
}

#[test]
fn test_non_fungible_data_resource_with_mutable_non_fungible_data() {
    let (is_non_fungible_data, is_non_fungible_data_locked): (bool, bool) = (true, false);

    test_resource_behavior_internal(
        ResourceType::NonFungible,
        ResourceMethodAuthKey::UpdateNonFungibleData,
        rule!(require(RADIX_TOKEN)),
        Mutability::MUTABLE(rule!(require(RADIX_TOKEN))),
        ResourceMethodAuthKey::UpdateNonFungibleData,
        (is_non_fungible_data, is_non_fungible_data_locked),
    )
}
