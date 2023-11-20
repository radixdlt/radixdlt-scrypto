use radix_engine_common::prelude::*;
use transaction::prelude::*;
use scrypto_unit::*;

#[test]
fn test_stake_reconcilation() {
    // Arrange
    // let initial_epoch = Epoch::of(5);
    let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    // let genesis = CustomGenesis::default(
    //     initial_epoch,
    //     CustomGenesis::default_consensus_manager_config(),
    // );
    let mut test_runner = TestRunnerBuilder::new()
        //.with_custom_genesis(genesis)
        .build();
    let (account_pk, _, account) = test_runner.new_account(false);

    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);
    let manifest = ManifestBuilder::new()
    .lock_fee(FAUCET, 100)
    .create_proof_from_account_of_non_fungibles(
        account,
        VALIDATOR_OWNER_BADGE,
        [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
    )
    .register_validator(validator_address)
    .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );
    receipt.expect_commit_success();



    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100)
        .create_proof_from_account_of_non_fungibles(
            account,
            VALIDATOR_OWNER_BADGE,
            [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
        )
        .withdraw_from_account(account, XRD, 100)
        .take_all_from_worktop(XRD, "stake")
        .stake_validator_as_owner(validator_address, "stake")
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_pk)],
    );



    let events = receipt.expect_commit(true).clone().application_events;
    for event in &events {
        let name = test_runner.event_name(&event.0);
        println!("{:?} - {}", event.0, name);
    }
}

