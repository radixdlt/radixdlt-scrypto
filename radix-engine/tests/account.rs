use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

fn fungible_amount() -> ResourceDeterminer {
    ResourceDeterminer::Some(
        Amount::Fungible {
            amount: Decimal(100),
        },
        RADIX_TOKEN,
    )
}

fn create_restricted_transfer_token(
    executor: &mut TransactionExecutor<InMemorySubstateStore>,
    account: ComponentId,
) -> (ResourceDefId, ResourceDefId) {
    let auth_resource_def_id = create_non_fungible_resource(executor, account);

    let package = executor
        .publish_package(&compile("resource_creator"))
        .unwrap();
    let transaction = TransactionBuilder::new(executor)
        .call_function(
            package,
            "ResourceCreator",
            "create_restricted_transfer",
            vec![auth_resource_def_id.to_string()],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    (auth_resource_def_id, receipt.new_resource_def_ids[0])
}

fn create_non_fungible_resource(
    executor: &mut TransactionExecutor<InMemorySubstateStore>,
    account: ComponentId,
) -> ResourceDefId {
    let package = executor
        .publish_package(&compile("resource_creator"))
        .unwrap();
    let transaction = TransactionBuilder::new(executor)
        .call_function(
            package,
            "ResourceCreator",
            "create_non_fungible_fixed",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    receipt.new_resource_def_ids[0]
}

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let (_, other_account) = executor.new_public_key_with_account();

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&fungible_amount(), account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let result = executor.run(transaction);

    // Assert
    assert!(result.unwrap().result.is_ok());
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let (_, other_account) = executor.new_public_key_with_account();
    let resource_def_id = create_non_fungible_resource(&mut executor, account);
    let non_fungible_amount = ResourceDeterminer::Some(
        Amount::NonFungible {
            ids: BTreeSet::from([NonFungibleId::from(1)]),
        },
        resource_def_id,
    );

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&non_fungible_amount, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let result = executor.run(transaction);

    // Assert
    println!("{:?}", result);
    assert!(result.unwrap().result.is_ok());
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, account) = executor.new_public_key_with_account();
    let (other_key, other_account) = executor.new_public_key_with_account();
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&fungible_amount(), account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![other_key])
        .unwrap();

    // Act
    let result = executor.run(transaction).unwrap();

    // Assert
    let runtime_error = result.result.expect_err("Should be runtime error");
    assert_eq!(runtime_error, RuntimeError::NotAuthorized);
}

#[test]
fn cannot_withdraw_restricted_transfer_from_my_account_with_no_auth() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let (_, other_account) = executor.new_public_key_with_account();
    let (_, token_resource_def_id) = create_restricted_transfer_token(&mut executor, account);
    let fungible_amount = ResourceDeterminer::Some(
        Amount::Fungible {
            amount: Decimal::one(),
        },
        token_resource_def_id,
    );

    // Act
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&fungible_amount, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let result = executor.run(transaction).unwrap();

    // Assert
    let err = result.result.expect_err("Should be a runtime error");
    assert_eq!(
        err,
        RuntimeError::ResourceDefError(ResourceDefError::PermissionNotAllowed)
    );
}

#[test]
fn account_to_bucket_to_account() {
    // Arrange
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (key, account) = executor.new_public_key_with_account();
    let amount = fungible_amount();
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&amount, account)
        .take_from_worktop(&amount, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_id: account,
                    method: "deposit".to_owned(),
                    args: vec![scrypto_encode(&scrypto::resource::Bucket(bucket_id))],
                })
                .0
        })
        .build(vec![key])
        .unwrap();

    // Act
    let result = executor.run(transaction);

    // Assert
    assert!(result.unwrap().result.is_ok());
}
