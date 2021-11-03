use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

fn publish_package(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    code: &[u8],
    publish_address: Option<&str>,
) -> Address {
    if publish_address.is_some() {
        let package_address = Address::from_str(publish_address.unwrap()).unwrap();
        executor.overwrite_package(package_address, code);
        package_address
    } else {
        executor.publish_package(code)
    }
}

fn run_transaction_and_assert_success(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    transaction: Transaction,
) -> Receipt {
    let transaction_receipt = executor.run(transaction, false).unwrap();
    println!("{:?}\n", transaction_receipt);
    assert!(transaction_receipt.success);

    transaction_receipt
}

fn call_blueprint_method_and_assert_success(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    package_address: Address,
    impl_name: &str,
    function_name: &str,
    args: Vec<String>,
    account: Address,
    signers: Vec<Address>,
) -> Receipt {
    run_transaction_and_assert_success(
        executor,
        TransactionBuilder::new(executor)
            .call_function(
                package_address,
                impl_name,
                function_name,
                args,
                Some(account),
            )
            .deposit_all_buckets(account)
            .build(signers)
            .unwrap(),
    )
}

fn instantiate_component(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    package_address: Address,
    impl_name: &str,
    constructor_name: &str,
    args: Vec<String>,
    account: Address,
    signers: Vec<Address>,
) -> Address {
    let receipt = call_blueprint_method_and_assert_success(
        executor,
        package_address,
        impl_name,
        constructor_name,
        args,
        account,
        signers,
    );
    receipt.component(0).unwrap()
}

fn instantiate_component_from_blueprint_code(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    code: &[u8],
    fixed_blueprint_publish_address: Option<&str>,
    impl_name: &str,
    constructor_name: &str,
    args: Vec<String>,
    account: Address,
    signers: Vec<Address>,
) -> (Address, Address) {
    let blueprint_package_address =
        publish_package(executor, code, fixed_blueprint_publish_address);
    let component_address = instantiate_component(
        executor,
        blueprint_package_address,
        impl_name,
        constructor_name,
        args,
        account,
        signers,
    );

    (component_address, blueprint_package_address)
}

/// Runs a method on an instantiated component
/// Any bucket inputs are taken from the given account, and outputs are returned to the account
fn call_method_against_account(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    instantiated_component_address: Address,
    method_name: &str,
    args: Vec<String>,
    account: Address,
    signers: Vec<Address>,
) -> Receipt {
    run_transaction_and_assert_success(
        executor,
        TransactionBuilder::new(executor)
            .call_method(
                instantiated_component_address,
                method_name,
                args,
                Some(account),
            )
            .deposit_all_buckets(account)
            .build(signers)
            .unwrap(),
    )
}

struct MutableTokenDefinition {
    symbol: Option<String>,
    name: Option<String>,
    description: Option<String>,
    url: Option<String>,
    icon_url: Option<String>,
}

fn create_mutable_token_definition(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    definition: MutableTokenDefinition,
    account: Address,
    signers: Vec<Address>,
) -> Address {
    let mut metadata = HashMap::new();
    definition
        .symbol
        .and_then(|d| metadata.insert("symbol".to_owned(), d.to_owned()));
    definition
        .name
        .and_then(|d| metadata.insert("name".to_owned(), d.to_owned()));
    definition
        .description
        .and_then(|d| metadata.insert("description".to_owned(), d.to_owned()));
    definition
        .url
        .and_then(|d| metadata.insert("url".to_owned(), d.to_owned()));
    definition
        .icon_url
        .and_then(|d| metadata.insert("icon_url".to_owned(), d.to_owned()));

    let receipt = run_transaction_and_assert_success(
        executor,
        TransactionBuilder::new(executor)
            .new_badge_fixed(HashMap::new(), 1.into())
            .deposit_all_buckets(account)
            .build(signers.clone())
            .unwrap(),
    );
    let mint_badge_address = receipt.resource_def(0).unwrap();

    let receipt = run_transaction_and_assert_success(
        executor,
        TransactionBuilder::new(executor)
            .new_token_mutable(metadata, mint_badge_address)
            .build(signers)
            .unwrap(),
    );

    receipt.resource_def(0).unwrap()
}

fn prepare_price_oracle(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    account: Address,
    signers: Vec<Address>,
) -> (Address, Address) {
    instantiate_component_from_blueprint_code(
        executor,
        include_code!("../../price-oracle"),
        Some("01806c33ab58c922240ce20a5b697546cc84aaecdf1b460a42c425"),
        "PriceOracle",
        "new",
        vec!["1".to_owned()],
        account,
        signers,
    )
}

fn prepare_synthetic_pool(
    executor: &mut TransactionExecutor<InMemoryLedger>,
    args: Vec<String>,
    account: Address,
    signers: Vec<Address>,
) -> (Address, Address) {
    instantiate_component_from_blueprint_code(
        executor,
        include_code!(),
        None,
        "SyntheticPool",
        "new",
        args,
        account,
        signers,
    )
}

fn bucket_argument_reference(amount: Decimal, resource_def: Address) -> String {
    // See prepare_custom_ty in TransactionBuilder for the format of arguments
    // TODO - output this from arguments - Decimal,ResourceDef
    // TODO - create MethodBuilder
    "1,030000000000000000000000000000000000000000000000000000".to_owned()
}

#[test]
fn test1_via_separate_transactions_to_ledger() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);

    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let collateral_token = RADIX_TOKEN;

    let (price_oracle_address, _) = prepare_price_oracle(&mut executor, account, vec![key]);

    let (synthetic_pool_address, _) = prepare_synthetic_pool(
        &mut executor,
        vec![
            price_oracle_address.to_string(),
            collateral_token.to_string(),
            "03806c33ab58c922240ce20a5b697546cc84aaecdf1b460a42c425".to_owned(),
            "4000000000".to_string(),
        ],
        account,
        vec![key],
    );

    // TODO Call method `mint_synthetic`

    // QUESTION I want to check account state now, but I can't.
}
