use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

fn fungible_amount() -> ResourceSpecification {
    ResourceSpecification::Fungible {
        amount: Decimal(100),
        resource_def_id: RADIX_TOKEN,
    }
}

fn create_non_fungible_resource(executor: &mut TransactionExecutor<InMemorySubstateStore>, account: ComponentId) -> ResourceSpecification {
    let package = executor.publish_package(&compile("resource_creator")).unwrap();
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
    let non_fungible_resource_def_id = receipt.new_resource_def_ids[0];
    ResourceSpecification::NonFungible {
        keys: BTreeSet::from([NonFungibleId::from(1)]),
        resource_def_id: non_fungible_resource_def_id,
    }
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
    let non_fungible_amount = create_non_fungible_resource(&mut executor, account);

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
    let result = executor.run(transaction);

    // Assert
    assert!(!result.unwrap().result.is_ok());
}

#[test]
fn cannot_withdraw_restricted_transfer_from_my_account_with_no_auth() {
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
