use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

use auto_lend::User;

struct TestEnv<'a, L: SubstateStore> {
    executor: TransactionExecutor<'a, L>,
    key: EcdsaPublicKey,
    account: Address,
    usd: Address,
    lending_pool: Address,
}

fn set_up_test_env<'a, L: SubstateStore>(ledger: &'a mut L) -> TestEnv<'a, L> {
    let mut executor = TransactionExecutor::new(ledger, false);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("auto_lend")).unwrap();

    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .new_token_fixed(HashMap::new(), 1_000_000.into())
                .call_method_with_all_resources(account, "deposit_batch")
                .build(vec![key])
                .unwrap()
        )
        .unwrap();
    let usd = receipt.resource_def(0).unwrap();

    let receipt = executor
        .run(
            TransactionBuilder::new(&executor)
                .call_function(
                    package,
                    "AutoLend",
                    "instantiate_autolend",
                    vec![usd.to_string(), "USD".to_owned()],
                    Some(account),
                )
                .call_method_with_all_resources(account, "deposit_batch")
                .build(vec![key])
                .unwrap()
        )
        .unwrap();
    let lending_pool = receipt.component(0).unwrap();

    TestEnv {
        executor,
        key,
        account,
        usd,
        lending_pool,
    }
}

fn create_user<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>) -> Address {
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(env.lending_pool, "new_user", args![], Some(env.account))
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    assert!(receipt.result.is_ok());
    receipt.resource_def(0).unwrap()
}

fn get_user_state<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>, user_id: Address) -> User {
    let mut receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "get_user",
                    vec![user_id.to_string()],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    assert!(receipt.result.is_ok());
    let encoded = receipt.outputs.swap_remove(0).raw;
    scrypto_decode(&encoded).unwrap()
}

#[test]
fn test_deposit_and_redeem() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user_id = create_user(&mut env);

    // First, deposit 100 USD
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "deposit",
                    vec![format!("{},{}", 1, user_id), format!("{},{}", 100, env.usd)],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "100".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Then, increase deposit interest rate to 5%
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "set_deposit_interest_rate",
                    vec!["0.05".to_string()],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);

    // After that, deposit another 100 USD
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "deposit",
                    vec![format!("{},{}", 1, user_id), format!("{},{}", 100, env.usd)],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "201".parse().unwrap(),
            deposit_interest_rate: "0.02990049751243781".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Finally, redeem with 150 aUSD
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "redeem",
                    vec![format!("{},{}", 1, user_id), "150".to_owned()],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "51".parse().unwrap(),
            deposit_interest_rate: "0.02990049751243781".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );
}

#[test]
fn test_borrow_and_repay() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user_id = create_user(&mut env);

    // First, deposit 1000 USD
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "deposit",
                    vec![
                        format!("{},{}", 1, user_id),
                        format!("{},{}", 1000, env.usd),
                    ],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Then, borrow 100 USD
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "borrow",
                    vec![format!("{},{}", 1, user_id), "100".to_owned()],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "100".parse().unwrap(),
            borrow_interest_rate: "0.02".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Then, increase borrow interest rate to 5%
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "set_borrow_interest_rate",
                    vec!["0.05".to_string()],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);

    // After that, borrow another 100 USD
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "borrow",
                    vec![format!("{},{}", 1, user_id), "100".to_owned()],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "202".parse().unwrap(),
            borrow_interest_rate: "0.034851485148514851".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Finally, repay with 150 USD
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "repay",
                    vec![format!("{},{}", 1, user_id), format!("{},{}", 150, env.usd)],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "59.039999999999999902".parse().unwrap(),
            borrow_interest_rate: "0.034851485148514851".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // F*k it, repay everything
    let receipt = env
        .executor
        .run(
            TransactionBuilder::new(&env.executor)
                .call_method(
                    env.lending_pool,
                    "repay",
                    vec![
                        format!("{},{}", 1, user_id),
                        format!("{},{}", 1000, env.usd),
                    ],
                    Some(env.account),
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(vec![env.key])
                .unwrap()
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );
}
