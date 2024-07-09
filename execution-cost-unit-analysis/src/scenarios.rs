use crate::executor::*;
use ignition_testing_framework::prelude::*;
use scrypto_test::prelude::*;
use std::path::PathBuf;

pub fn faucet_lock_fee(
    _: &mut DefaultLedgerSimulator,
    _: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    (
        ManifestBuilder::new().lock_fee_from_faucet().build(),
        Default::default(),
    )
}

pub fn faucet_lock_fee_and_free_xrd(
    ledger: &mut DefaultLedgerSimulator,
    _: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (_, _, account) = ledger.new_account(false);
    (
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        Default::default(),
    )
}

pub fn radiswap_publish_package(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (_, _, account) = ledger.new_account(false);
    let (wasm, definition) = package_loader.get(
        &PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("blueprints")
            .join("radiswap")
            .join("Cargo.toml"),
    );

    (
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package(wasm.clone(), definition.clone())
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        Default::default(),
    )
}

pub fn radiswap_create_pool(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (_, _, account) = ledger.new_account(false);
    let (wasm, definition) = package_loader.get(
        &PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("blueprints")
            .join("radiswap")
            .join("Cargo.toml"),
    );
    let radiswap_package = ledger.publish_package(
        (wasm.clone(), definition.clone()),
        Default::default(),
        Default::default(),
    );
    let resource1 = ledger.create_fungible_resource(100.into(), 18, account);
    let resource2 = ledger.create_fungible_resource(100.into(), 18, account);

    (
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                radiswap_package,
                "Radiswap",
                "new",
                (OwnerRole::None, resource1, resource2),
            )
            .build(),
        Default::default(),
    )
}

pub fn radiswap_add_liquidity(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (pk, _, account) = ledger.new_account(false);
    let (wasm, definition) = package_loader.get(
        &PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("blueprints")
            .join("radiswap")
            .join("Cargo.toml"),
    );
    let radiswap_package = ledger.publish_package(
        (wasm.clone(), definition.clone()),
        Default::default(),
        Default::default(),
    );
    let resource1 = ledger.create_fungible_resource(100.into(), 18, account);
    let resource2 = ledger.create_fungible_resource(100.into(), 18, account);

    let (radiswap_component_address, _) = {
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    radiswap_package,
                    "Radiswap",
                    "new",
                    (OwnerRole::None, resource1, resource2),
                )
                .build(),
            vec![],
        );
        let receipt = receipt.expect_commit_success();

        (
            receipt.new_component_addresses().first().copied().unwrap(),
            receipt.new_resource_addresses().first().copied().unwrap(),
        )
    };

    (
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource1, 10)
            .withdraw_from_account(account, resource2, 10)
            .take_all_from_worktop(resource1, "bucket1")
            .take_all_from_worktop(resource2, "bucket2")
            .then(|builder| {
                let bucket1 = builder.bucket("bucket1");
                let bucket2 = builder.bucket("bucket2");

                builder.call_method(
                    radiswap_component_address,
                    "add_liquidity",
                    (bucket1, bucket2),
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![pk.into()],
    )
}

pub fn radiswap_remove_liquidity(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (pk, _, account) = ledger.new_account(false);
    let (wasm, definition) = package_loader.get(
        &PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("blueprints")
            .join("radiswap")
            .join("Cargo.toml"),
    );
    let radiswap_package = ledger.publish_package(
        (wasm.clone(), definition.clone()),
        Default::default(),
        Default::default(),
    );
    let resource1 = ledger.create_fungible_resource(100.into(), 18, account);
    let resource2 = ledger.create_fungible_resource(100.into(), 18, account);

    let (radiswap_component_address, pool_unit_resource_address) = {
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    radiswap_package,
                    "Radiswap",
                    "new",
                    (OwnerRole::None, resource1, resource2),
                )
                .build(),
            vec![],
        );
        let receipt = receipt.expect_commit_success();

        (
            receipt.new_component_addresses().first().copied().unwrap(),
            receipt.new_resource_addresses().first().copied().unwrap(),
        )
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .withdraw_from_account(account, resource1, 10)
                .withdraw_from_account(account, resource2, 10)
                .take_all_from_worktop(resource1, "bucket1")
                .take_all_from_worktop(resource2, "bucket2")
                .then(|builder| {
                    let bucket1 = builder.bucket("bucket1");
                    let bucket2 = builder.bucket("bucket2");

                    builder.call_method(
                        radiswap_component_address,
                        "add_liquidity",
                        (bucket1, bucket2),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit_success();

    (
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, pool_unit_resource_address, 1)
            .take_all_from_worktop(pool_unit_resource_address, "bucket1")
            .then(|builder| {
                let bucket1 = builder.bucket("bucket1");

                builder.call_method(radiswap_component_address, "remove_liquidity", (bucket1,))
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![pk.into()],
    )
}

pub fn radiswap_single_swap(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (pk, _, account) = ledger.new_account(false);
    let (wasm, definition) = package_loader.get(
        &PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("blueprints")
            .join("radiswap")
            .join("Cargo.toml"),
    );
    let radiswap_package = ledger.publish_package(
        (wasm.clone(), definition.clone()),
        Default::default(),
        Default::default(),
    );
    let resource1 = ledger.create_fungible_resource(100.into(), 18, account);
    let resource2 = ledger.create_fungible_resource(100.into(), 18, account);

    let (radiswap_component_address, _) = {
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    radiswap_package,
                    "Radiswap",
                    "new",
                    (OwnerRole::None, resource1, resource2),
                )
                .build(),
            vec![],
        );
        let receipt = receipt.expect_commit_success();

        (
            receipt.new_component_addresses().first().copied().unwrap(),
            receipt.new_resource_addresses().first().copied().unwrap(),
        )
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .withdraw_from_account(account, resource1, 10)
                .withdraw_from_account(account, resource2, 10)
                .take_all_from_worktop(resource1, "bucket1")
                .take_all_from_worktop(resource2, "bucket2")
                .then(|builder| {
                    let bucket1 = builder.bucket("bucket1");
                    let bucket2 = builder.bucket("bucket2");

                    builder.call_method(
                        radiswap_component_address,
                        "add_liquidity",
                        (bucket1, bucket2),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit_success();

    (
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource1, 1)
            .take_all_from_worktop(resource1, "bucket1")
            .then(|builder| {
                let bucket1 = builder.bucket("bucket1");

                builder.call_method(radiswap_component_address, "swap", (bucket1,))
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![pk.into()],
    )
}

pub fn radiswap_two_swaps(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (pk, _, account) = ledger.new_account(false);
    let (wasm, definition) = package_loader.get(
        &PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("blueprints")
            .join("radiswap")
            .join("Cargo.toml"),
    );
    let radiswap_package = ledger.publish_package(
        (wasm.clone(), definition.clone()),
        Default::default(),
        Default::default(),
    );
    let resource1 = ledger.create_fungible_resource(100.into(), 18, account);
    let resource2 = ledger.create_fungible_resource(100.into(), 18, account);

    let (radiswap_component_address, _) = {
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    radiswap_package,
                    "Radiswap",
                    "new",
                    (OwnerRole::None, resource1, resource2),
                )
                .build(),
            vec![],
        );
        let receipt = receipt.expect_commit_success();

        (
            receipt.new_component_addresses().first().copied().unwrap(),
            receipt.new_resource_addresses().first().copied().unwrap(),
        )
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .withdraw_from_account(account, resource1, 10)
                .withdraw_from_account(account, resource2, 10)
                .take_all_from_worktop(resource1, "bucket1")
                .take_all_from_worktop(resource2, "bucket2")
                .then(|builder| {
                    let bucket1 = builder.bucket("bucket1");
                    let bucket2 = builder.bucket("bucket2");

                    builder.call_method(
                        radiswap_component_address,
                        "add_liquidity",
                        (bucket1, bucket2),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit_success();

    (
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource1, 1)
            .take_all_from_worktop(resource1, "bucket1")
            .then(|builder| {
                let bucket = builder.bucket("bucket1");

                builder.call_method(radiswap_component_address, "swap", (bucket,))
            })
            .withdraw_from_account(account, resource2, 1)
            .take_all_from_worktop(resource2, "bucket2")
            .then(|builder| {
                let bucket = builder.bucket("bucket2");

                builder.call_method(radiswap_component_address, "swap", (bucket,))
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![pk.into()],
    )
}

pub fn ignition_caviarnine_v1_open_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_open_liquidity_position_transaction(ledger, package_loader, |env| env.caviarnine_v1)
}

pub fn ignition_ociswap_v1_open_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_open_liquidity_position_transaction(ledger, package_loader, |env| env.ociswap_v2)
}

pub fn ignition_ociswap_v2_open_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_open_liquidity_position_transaction(ledger, package_loader, |env| env.ociswap_v2)
}

pub fn ignition_defiplaza_v2_open_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_open_liquidity_position_transaction(ledger, package_loader, |env| env.defiplaza_v2)
}

pub fn ignition_caviarnine_v1_close_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_close_liquidity_position_transaction(ledger, package_loader, |env| env.caviarnine_v1)
}

pub fn ignition_ociswap_v1_close_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_close_liquidity_position_transaction(ledger, package_loader, |env| env.ociswap_v2)
}

pub fn ignition_ociswap_v2_close_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_close_liquidity_position_transaction(ledger, package_loader, |env| env.ociswap_v2)
}

pub fn ignition_defiplaza_v2_close_position(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    ignition_close_liquidity_position_transaction(ledger, package_loader, |env| env.defiplaza_v2)
}

fn ignition_open_liquidity_position_transaction(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
    callback: impl FnOnce(&ScryptoUnitEnv) -> DexEntities<ComponentAddress, ComponentAddress>,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (
        ScryptoUnitEnv {
            resources,
            protocol,
            ..
        },
        dex,
    ) = {
        let env = ScryptoUnitEnv::new_with_configuration(
            Configuration {
                maximum_allowed_relative_price_difference: dec!(1),
                ..Default::default()
            },
            move |path| package_loader.get(path),
            ledger,
        );
        let dex = callback(&env);
        (env, dex)
    };
    let (pk, _, account) = ledger.new_account(false);
    ledger
        .execute_manifest_with_enabled_modules(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resources.bitcoin, 100_000)
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            EnabledModules::for_notarized_transaction() & !EnabledModules::AUTH,
        )
        .expect_commit_success();

    (
        ManifestBuilder::new()
            .lock_fee(account, 100)
            .withdraw_from_account(account, resources.bitcoin, 100_000)
            .take_all_from_worktop(resources.bitcoin, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(
                    protocol.ignition,
                    "open_liquidity_position",
                    (
                        bucket,
                        dex.pools.bitcoin,
                        LockupPeriod::from_months(6).unwrap(),
                    ),
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![pk.into()],
    )
}

fn ignition_close_liquidity_position_transaction(
    ledger: &mut DefaultLedgerSimulator,
    package_loader: &mut PackageLoader,
    callback: impl FnOnce(&ScryptoUnitEnv) -> DexEntities<ComponentAddress, ComponentAddress>,
) -> (TransactionManifestV1, Vec<PublicKey>) {
    let (
        ScryptoUnitEnv {
            resources,
            protocol,
            ..
        },
        dex,
    ) = {
        let env = ScryptoUnitEnv::new_with_configuration(
            Configuration {
                maximum_allowed_relative_price_difference: dec!(1),
                ..Default::default()
            },
            move |path| package_loader.get(path),
            ledger,
        );
        let dex = callback(&env);
        (env, dex)
    };
    let (pk, _, account) = ledger.new_account(false);
    ledger
        .execute_manifest_with_enabled_modules(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resources.bitcoin, 100_000)
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            EnabledModules::for_notarized_transaction() & !EnabledModules::AUTH,
        )
        .expect_commit_success();

    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee(account, 100)
                .withdraw_from_account(account, resources.bitcoin, 100_000)
                .take_all_from_worktop(resources.bitcoin, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        protocol.ignition,
                        "open_liquidity_position",
                        (
                            bucket,
                            dex.pools.bitcoin,
                            LockupPeriod::from_months(6).unwrap(),
                        ),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
        )
        .expect_commit_success();

    let current_time = ledger.get_current_time(TimePrecisionV2::Minute);
    let maturity_instant = current_time
        .add_seconds(*LockupPeriod::from_months(6).unwrap().seconds() as i64)
        .unwrap();
    {
        let db = ledger.substate_db_mut();
        let mut writer = SystemDatabaseWriter::new(db);

        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMilliTimestamp.field_index(),
                ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(
                    ProposerMilliTimestampSubstate {
                        epoch_milli: maturity_instant.seconds_since_unix_epoch * 1000,
                    },
                ),
            )
            .unwrap();

        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMinuteTimestamp.field_index(),
                ConsensusManagerProposerMinuteTimestampFieldPayload::from_content_source(
                    ProposerMinuteTimestampSubstate {
                        epoch_minute: i32::try_from(maturity_instant.seconds_since_unix_epoch / 60)
                            .unwrap(),
                    },
                ),
            )
            .unwrap();
    }

    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    protocol.oracle,
                    "set_price",
                    (resources.bitcoin, XRD, dec!(1)),
                )
                .build(),
        )
        .expect_commit_success();

    (
        ManifestBuilder::new()
            .lock_fee(account, 100)
            .withdraw_from_account(account, dex.liquidity_receipt, 1)
            .take_all_from_worktop(dex.liquidity_receipt, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(protocol.ignition, "close_liquidity_position", (bucket,))
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![pk.into()],
    )
}
