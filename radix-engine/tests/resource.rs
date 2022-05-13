use radix_engine::engine::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::ResourceManagerError;
use radix_engine::transaction::*;
use scrypto::call_data;
use scrypto::prelude::*;

pub fn test_resource_creation(
    resource_auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    expect_err: bool,
) {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut substate_store, false);
    let (pk, sk, account) = executor.new_account();

    // Act
    let metadata: HashMap<String, String> = HashMap::new();
    let new_token_tx = TransactionBuilder::new()
        .call_function(
            SYSTEM_PACKAGE,
            "System",
            call_data![new_resource(
                ResourceType::Fungible { divisibility: 18 },
                metadata,
                resource_auth,
                Some(MintParams::Fungible { amount: dec!("1") })
            )]
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&new_token_tx).unwrap();

    // Assert
    if expect_err {
        receipt.result.expect_err("Should be a runtime error");
    } else {
        receipt.result.expect("Should be okay.");
    }
}

#[test]
fn test_resource_manager() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(package, "ResourceTest", call_data!(create_fungible()))
        .call_function(package, "ResourceTest", call_data!(query()))
        .call_function(package, "ResourceTest", call_data!(burn()))
        .call_function(
            package,
            "ResourceTest",
            call_data!(update_resource_metadata()),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            call_data![create_fungible_and_mint(0u8, dec!("0.1"))],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::InvalidAmount(
            Decimal::from("0.1"),
            0
        ))
    );
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "resource")))
        .unwrap();

    // Act
    let transaction = TransactionBuilder::new()
        .call_function(
            package,
            "ResourceTest",
            call_data![create_fungible_and_mint(0u8, dec!(100_000_000_001i128))],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt = executor.validate_and_execute(&transaction).unwrap();

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::MaxMintAmountExceeded)
    );
}

#[test]
fn resource_creation_with_static_rules_should_succeed() {
    let mut resource_auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)> = HashMap::new();
    resource_auth.insert(ResourceMethodAuthKey::Mint, (rule!(require(RADIX_TOKEN)), LOCKED));

    test_resource_creation(resource_auth, false);
}

#[test]
fn resource_creation_with_dynamic_behavior_rule_should_fail() {
    let mut resource_auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)> = HashMap::new();
    resource_auth.insert(
        ResourceMethodAuthKey::Mint,
        (rule!(require("some_dynamic_badge")), LOCKED),
    );

    test_resource_creation(resource_auth, true);
}

#[test]
fn resource_creation_with_dynamic_mutable_rule_should_fail() {
    let mut resource_auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)> = HashMap::new();
    resource_auth.insert(
        ResourceMethodAuthKey::Mint,
        (
            rule!(require(RADIX_TOKEN)),
            MUTABLE(rule!(require("some_dynamic_badge"))),
        ),
    );

    test_resource_creation(resource_auth, true);
}

#[test]
fn resource_creation_with_dynamic_behavior_and_mutable_rule_should_fail() {
    let mut resource_auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)> = HashMap::new();
    resource_auth.insert(
        ResourceMethodAuthKey::Mint,
        (
            rule!(require("some_dynamic_badge")),
            MUTABLE(rule!(require("some_dynamic_badge"))),
        ),
    );

    test_resource_creation(resource_auth, true);
}