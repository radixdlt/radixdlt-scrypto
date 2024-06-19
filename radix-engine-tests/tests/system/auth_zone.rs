use radix_common::prelude::*;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::CreateFrameError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{SystemApi, ACTOR_REF_AUTH_ZONE};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_test::prelude::*;

#[test]
fn should_not_be_able_to_move_auth_zone() {
    // Arrange
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y, V>(
            &mut self,
            export_name: &str,
            input: &IndexedScryptoValue,
            api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
            V: VmApi,
        {
            match export_name {
                "test" => {
                    let auth_zone_id = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE).unwrap();
                    let self_blueprint_id = api.actor_get_blueprint_id()?;
                    api.call_function(
                        self_blueprint_id.package_address,
                        self_blueprint_id.blueprint_name.as_str(),
                        "hi",
                        scrypto_encode(&Own(auth_zone_id)).unwrap(),
                    )?;
                }
                "hi" => {
                    return Ok(input.clone());
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![("test", "test", false), ("hi", "hi", false)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "test", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(..))
            ))
        )
    });
}

#[test]
fn test_auth_zone_create_proof_of_all_for_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, 10)
        .create_proof_from_auth_zone_of_all(XRD, "proof")
        .drop_proof("proof")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_auth_zone_create_proof_of_all_for_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_non_fungibles(
            account,
            resource_address,
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
            ],
        )
        .create_proof_from_auth_zone_of_all(resource_address, "proof")
        .drop_proof("proof")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

fn get_transaction_substates(
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
) -> HashMap<(DbPartitionKey, DbSortKey), Vec<u8>> {
    // Store current DB substate value hashes for comparision after staking execution
    let mut transaction_substates: HashMap<(DbPartitionKey, DbSortKey), Vec<u8>> = HashMap::new();
    let db = ledger.substate_db();

    let old_keys: Vec<DbPartitionKey> = db.list_partition_keys().collect();
    // print_partition_keys(&old_keys);
    for key in old_keys {
        let entries = db.list_entries(&key);
        for (sort_key, value) in entries {
            transaction_substates.insert((key.clone(), sort_key), value);
        }
    }
    transaction_substates
}

#[allow(dead_code)]
fn print_substates(substates: &HashMap<(DbPartitionKey, DbSortKey), Vec<u8>>) {
    for (full_key, value) in substates {
        let address = AddressBech32Encoder::for_simulator()
            .encode(
                &SpreadPrefixKeyMapper::from_db_partition_key(&full_key.0)
                    .0
                     .0,
            )
            .unwrap();

        let (db_partition_key, db_sort_key) = full_key;
        println!(
            "            (
                // {}
                DbPartitionKey {{
                    node_key: unhex({:?}),
                    partition_num: {:?},
                }},
                DbSortKey(unhex({:?}))
            ) => (
                unhex({:?}),
            ),",
            address,
            hex::encode(&db_partition_key.node_key),
            db_partition_key.partition_num,
            hex::encode(&db_sort_key.0),
            hex::encode(value)
        );
    }
}

fn unhex(input: &'static str) -> Vec<u8> {
    hex::decode(input).unwrap()
}

#[test]
fn test_auth_zone_steal() {
    use radix_engine_tests::common::*;

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("steal"));

    let expected_updated_substates = hashmap!(
            (
                // internal_vault_sim1tpsesv77qvw782kknjks9g3x2msg8cc8ldshk28pkf6m6lkhun3sel
                DbPartitionKey {
                    node_key: unhex("f3052b1133393854e7f8ddc613929df4d35c775858619833de031de3aad69cad02a22656e083e307fb617b28e1b275bd7ed7"),
                    partition_num: 64,
                },
                DbSortKey(unhex("00"))
            ) => (
                unhex("5c2200012102220001a0402ce76c20cf153e01000000000000000000000000000000220000")
            ),
            (
                // internal_vault_sim1tz9uaalv8g3ahmwep2trlyj2m3zn7rstm9pwessa3k56me2fcduq2u
                DbPartitionKey {
                    node_key: unhex("06ef5035dba9d29588fa280b760358845b5070f1588bcef7ec3a23dbedd90a963f924adc453f0e0bd942ecc21d8da9ade549"),
                    partition_num: 64,
                },
                DbSortKey(unhex("00"))
            ) => (
                unhex("5c2200012102220001a080a7f173ec277b95614bc772614213000000000000000000220000")
            ),
            (
                // consensusmanager_sim1scxxxxxxxxxxcnsmgrxxxxxxxxx000999665565xxxxxxxxxxc06cl
                DbPartitionKey {
                    node_key: unhex("14a7d055604bf45858649fde5f5ff598e6f99e0e860c6318c6318c6c4e1b40cc6318c6318cf7bca52eb54a6a86318c6318c6"),
                    partition_num: 64,
                },
                DbSortKey(unhex("02"))
            ) => (
                unhex("5c220001210222000121022307a001002096733690e70a9f000000000000000000000000000000009058619833de031de3aad69cad02a22656e083e307fb617b28e1b275bd7ed7220000")
            ),

    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "Steal", "instantiate", manifest_args!())
            .build(),
        vec![],
    );
    let steal_component_address = receipt.expect_commit_success().new_component_addresses()[0];

    let pre_transaction_substates = get_transaction_substates(&mut ledger);

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                steal_component_address,
                "steal_from_account",
                manifest_args!(account),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            err,
        ))) => err.fn_identifier.eq(&FnIdentifier {
            blueprint_id: BlueprintId::new(&ACCOUNT_PACKAGE, "Account"),
            ident: "withdraw".to_owned(),
        }),
        _ => false,
    });

    // Check if updates substates are expected
    let post_transaction_substates = get_transaction_substates(&mut ledger);

    let mut updated_substates: HashMap<(DbPartitionKey, DbSortKey), Vec<u8>> = hashmap!();
    let mut new_substates: HashMap<(DbPartitionKey, DbSortKey), Vec<u8>> = hashmap!();
    for (full_key, post_value) in post_transaction_substates {
        if let Some(pre_value) = pre_transaction_substates.get(&full_key) {
            if !pre_value.eq(&post_value) {
                updated_substates.insert(full_key, post_value);
            }
        } else {
            new_substates.insert(full_key, post_value);
        }
    }

    // println!("Updated substates: ");
    // print_substates(&updated_substates);
    assert_eq!(updated_substates, expected_updated_substates);

    assert_eq!(new_substates.len(), 0);
}
