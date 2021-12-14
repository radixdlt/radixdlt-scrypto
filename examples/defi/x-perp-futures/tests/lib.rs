use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

use x_perp_futures::Position;
use x_perp_futures::PositionType;

struct TestEnv<'a, L: Ledger> {
    executor: TransactionExecutor<'a, L>,
    key: Address,
    account: Address,
    usd: Address,
    clearing_house: Address,
}

fn set_up_test_env<'a, L: Ledger>(ledger: &'a mut L) -> TestEnv<'a, L> {
    let mut executor = TransactionExecutor::new(ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("x_perp_futures"));

    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .new_token_fixed(HashMap::new(), 1_000_000.into())
                .deposit_all_buckets(account)
                .build(vec![key])
                .unwrap(),
            false,
        )
        .unwrap();
    let usd = receipt.resource_def(0).unwrap();

    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(
                    package,
                    "ClearingHouse",
                    "new",
                    vec![usd.to_string(), "1".to_owned(), "99999".to_owned()],
                    Some(account),
                )
                .deposit_all_buckets(account)
                .build(vec![key])
                .unwrap(),
            false,
        )
        .unwrap();
    let clearing_house = receipt.component(0).unwrap();

    TestEnv {
        executor,
        key,
        account,
        usd,
        clearing_house,
    }
}

fn create_user<'a, L: Ledger>(env: &mut TestEnv<'a, L>) -> Address {
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(env.clearing_house, "new_user", args![], Some(env.account))
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    assert!(receipt.success);
    receipt.resource_def(0).unwrap()
}

fn get_position<'a, L: Ledger>(env: &mut TestEnv<'a, L>, user_id: Address, nth: usize) -> Position {
    let mut receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.clearing_house,
                    "get_position",
                    vec![user_id.to_string(), nth.to_string()],
                    Some(env.account),
                )
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    assert!(receipt.success);
    let encoded = receipt.results.swap_remove(0).unwrap().unwrap().encoded;
    scrypto_decode(&encoded).unwrap()
}

#[test]
fn test_long() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user1 = create_user(&mut env);
    let user2 = create_user(&mut env);

    // First, user1 longs BTC with 500 USD x4
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.clearing_house,
                    "new_position",
                    vec![
                        format!("{},{}", 1, user1),
                        format!("{},{}", 500, env.usd),
                        4.to_string(),
                        "Long".to_owned(),
                    ],
                    Some(env.account),
                )
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user1, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Long,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "0.019608035372895813".parse().unwrap()
        }
    );

    // First, user2 longs BTC with 500 USD x1
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.clearing_house,
                    "new_position",
                    vec![
                        format!("{},{}", 1, user2),
                        format!("{},{}", 500, env.usd),
                        4.to_string(),
                        "Long".to_owned(),
                    ],
                    Some(env.account),
                )
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user2, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Long,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "0.018853872914683876".parse().unwrap()
        }
    );

    // user1 settles his position
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.clearing_house,
                    "settle_position",
                    vec![format!("{},{}", 1, user1), "0".to_owned()],
                    Some(env.account),
                )
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    println!("{:?}", receipt);
}

#[test]
fn test_short() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user1 = create_user(&mut env);
    let user2 = create_user(&mut env);

    // First, user1 shorts BTC with 500 USD x4
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.clearing_house,
                    "new_position",
                    vec![
                        format!("{},{}", 1, user1),
                        format!("{},{}", 500, env.usd),
                        4.to_string(),
                        "Short".to_owned(),
                    ],
                    Some(env.account),
                )
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user1, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Short,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "-0.02040837151399504".parse().unwrap()
        }
    );

    // First, user2 shorts BTC with 500 USD x1
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.clearing_house,
                    "new_position",
                    vec![
                        format!("{},{}", 1, user2),
                        format!("{},{}", 500, env.usd),
                        4.to_string(),
                        "Short".to_owned(),
                    ],
                    Some(env.account),
                )
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user2, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Short,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "-0.021258729184970573".parse().unwrap()
        }
    );

    // user1 settles his position
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.clearing_house,
                    "settle_position",
                    vec![format!("{},{}", 1, user1), "0".to_owned()],
                    Some(env.account),
                )
                .deposit_all_buckets(env.account)
                .build(vec![env.key])
                .unwrap(),
            false,
        )
        .unwrap();
    println!("{:?}", receipt);
}
