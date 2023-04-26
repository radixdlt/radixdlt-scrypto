use radix_engine::blueprints::resource::FungibleResourceManagerSubstate;
use radix_engine::system::bootstrap::{
    Bootstrapper, GenesisDataChunk, GenesisResource, GenesisResourceAllocation,
    GenesisStakeAllocation,
};
use radix_engine::transaction::BalanceChange;
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use radix_engine_stores::interface::SubstateDatabase;
use radix_engine_stores::jmt_support::JmtMapper;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = EcdsaSecp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm);

    let (system_bootstrap_receipt, _, wrap_up_receipt) = bootstrapper
        .bootstrap_with_genesis_data(genesis_data_chunks, 1u64, 100u32, 1u64, 1u64)
        .unwrap();

    assert!(system_bootstrap_receipt
        .commit_result
        .new_package_addresses()
        .contains(&PACKAGE_PACKAGE));

    wrap_up_receipt
        .commit_result
        .next_epoch()
        .expect("There should be a new epoch.");
}

#[test]
fn test_genesis_xrd_allocation_to_accounts() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let account_public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let account_component_address = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(account_public_key.clone()),
    );
    let allocation_amount = dec!("100");
    let genesis_data_chunks = vec![GenesisDataChunk::XrdBalances(vec![(
        account_component_address,
        allocation_amount,
    )])];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm);

    let (_, data_ingestion_receipts, _) = bootstrapper
        .bootstrap_with_genesis_data(genesis_data_chunks, 1u64, 100u32, 1u64, 1u64)
        .unwrap();

    assert!(data_ingestion_receipts[0]
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc| rc.amount == allocation_amount
            && rc.node_id.eq(account_component_address.as_node_id())
            && rc.resource_address == RADIX_TOKEN));
}

#[test]
fn test_genesis_resource_with_initial_allocation() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let token_holder = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key()),
    );
    let address_bytes_without_entity_id = hash(vec![1, 2, 3]).lower_bytes();
    let resource_address = ResourceAddress::new_unchecked(
        NodeId::new(
            EntityType::GlobalFungibleResource as u8,
            &address_bytes_without_entity_id,
        )
        .0,
    );

    let owner = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key(),
    );
    let allocation_amount = dec!("105");
    let genesis_resource = GenesisResource {
        address_bytes_without_entity_id,
        initial_supply: allocation_amount,
        metadata: vec![("symbol".to_string(), "TST".to_string())],
        owner: Some(owner),
    };
    let resource_allocation = GenesisResourceAllocation {
        account_index: 0,
        amount: allocation_amount,
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Resources(vec![genesis_resource]),
        GenesisDataChunk::ResourceBalances {
            accounts: vec![token_holder.clone()],
            allocations: vec![(resource_address.clone(), vec![resource_allocation])],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm);

    let (_, mut data_ingestion_receipts, _) = bootstrapper
        .bootstrap_with_genesis_data(genesis_data_chunks, 1u64, 100u32, 1u64, 1u64)
        .unwrap();

    let resource_manager_substate = substate_db
        .get_mapped_substate::<JmtMapper, FungibleResourceManagerSubstate>(
            &resource_address.as_node_id(),
            SysModuleId::Object.into(),
            ResourceManagerOffset::ResourceManager.into(),
        )
        .unwrap();
    assert_eq!(resource_manager_substate.total_supply, allocation_amount);

    let key = scrypto_encode("symbol").unwrap();
    let entry = substate_db
        .get_mapped_substate::<JmtMapper, Option<MetadataEntry>>(
            &resource_address.as_node_id(),
            SysModuleId::Metadata.into(),
            SubstateKey::Map(key),
        )
        .unwrap();

    if let Some(MetadataEntry::Value(MetadataValue::String(symbol))) = entry {
        assert_eq!(symbol, "TST");
    } else {
        panic!("Resource symbol was not a string");
    }

    let allocation_receipt = data_ingestion_receipts.pop().unwrap();
    let resource_creation_receipt = data_ingestion_receipts.pop().unwrap();

    assert!(resource_creation_receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc|
            // Not an ideal condition, but assuming this is the owner badge
            rc.amount == dec!("1")
                && rc.node_id.eq(owner.as_node_id())));

    assert!(allocation_receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc| rc.amount == allocation_amount
            && rc.node_id.eq(token_holder.as_node_id())
            && rc.resource_address.eq(&resource_address)));
}

#[test]
fn test_genesis_stake_allocation() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();

    // There are two genesis validators
    // - one with two stakers (0 and 1)
    // - one with one staker (just 1)
    let validator_0_key = EcdsaSecp256k1PrivateKey::from_u64(10).unwrap().public_key();
    let validator_1_key = EcdsaSecp256k1PrivateKey::from_u64(11).unwrap().public_key();
    let staker_0 = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );
    let staker_1 = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(5).unwrap().public_key(),
    );
    let validator_0_allocations = vec![
        GenesisStakeAllocation {
            account_index: 0,
            xrd_amount: dec!("10"),
        },
        GenesisStakeAllocation {
            account_index: 1,
            xrd_amount: dec!("50000"),
        },
    ];
    let validator_1_allocations = vec![GenesisStakeAllocation {
        account_index: 1,
        xrd_amount: dec!("1"),
    }];
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![
            validator_0_key.clone().into(),
            validator_1_key.clone().into(),
        ]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_0, staker_1],
            allocations: vec![
                (validator_0_key, validator_0_allocations),
                (validator_1_key, validator_1_allocations),
            ],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm);

    let (_, mut data_ingestion_receipts, _) = bootstrapper
        .bootstrap_with_genesis_data(genesis_data_chunks, 1u64, 100u32, 1u64, 1u64)
        .unwrap();

    let allocate_stakes_receipt = data_ingestion_receipts.pop().unwrap();

    // Staker 0 should have one liquidity balance entry
    {
        let address: GlobalAddress = staker_0.into();
        let balances = allocate_stakes_receipt
            .expect_commit_success()
            .state_update_summary
            .balance_changes
            .get(&address)
            .unwrap();
        assert_eq!(balances.len(), 1);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("10"))));
    }

    // Staker 1 should have two liquidity balance entries
    {
        let address: GlobalAddress = staker_1.into();
        let balances = allocate_stakes_receipt
            .expect_commit_success()
            .state_update_summary
            .balance_changes
            .get(&address)
            .unwrap();
        assert_eq!(balances.len(), 2);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("1"))));
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("50000"))));
    }
}
