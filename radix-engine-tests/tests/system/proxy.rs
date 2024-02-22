use radix_engine_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn initialize_package(
    ledger: &mut DefaultLedgerSimulator,
    owner_badge: NonFungibleGlobalId,
    manager_badge: NonFungibleGlobalId,
    package_name: &str,
    blueprint_name: &str,
    function_name: &str,
) -> ComponentAddress {
    let package_address = ledger.publish_package_simple(PackageLoader::get(package_name));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            blueprint_name,
            function_name,
            manifest_args!(owner_badge, manager_badge),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let my_component = receipt.expect_commit(true).new_component_addresses()[0];
    my_component
}

fn create_some_resources(ledger: &mut DefaultLedgerSimulator) -> IndexMap<String, ResourceAddress> {
    let (_public_key, _, account_address) = ledger.new_account(false);
    let mut resources = indexmap!();

    for symbol in ["XRD", "USDT", "ETH"] {
        resources.insert(
            symbol.to_string(),
            ledger.create_fungible_resource(dec!(20000), 18, account_address),
        );
    }
    resources
}

fn oracle_configure_as_global(
    ledger: &mut DefaultLedgerSimulator,
    proxy_manager_badge: NonFungibleGlobalId,
    oracle_manager_badge: NonFungibleGlobalId,
    resources: &IndexMap<String, ResourceAddress>,
    proxy_address: ComponentAddress,
    oracle_address: ComponentAddress,
) {
    // Set Oracle component address in Proxy
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "set_oracle_address",
            manifest_args!(oracle_address),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proxy_manager_badge]);
    receipt.expect_commit_success();

    // "set_price" is a protected method, need to be called directly on the Oracle component
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            oracle_address,
            "set_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("USDT").unwrap(),
                dec!(20)
            ),
        )
        .call_method(
            oracle_address,
            "set_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("ETH").unwrap(),
                dec!(30)
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![oracle_manager_badge.clone()]);
    receipt.expect_commit_success();
}

fn oracle_configure_as_owned(
    ledger: &mut DefaultLedgerSimulator,
    proxy_manager_badge: NonFungibleGlobalId,
    resources: &IndexMap<String, ResourceAddress>,
    proxy_address: ComponentAddress,
    oracle_package_address: PackageAddress,
) {
    // Set Oracle component address in Proxy
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "initialize_oracle",
            manifest_args!(oracle_package_address),
        )
        .call_method(
            proxy_address,
            "proxy_set_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("USDT").unwrap(),
                dec!(20)
            ),
        )
        .call_method(
            proxy_address,
            "proxy_set_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("ETH").unwrap(),
                dec!(30)
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proxy_manager_badge]);
    receipt.expect_commit_success();
}

fn oracle_v3_configure_as_global(
    ledger: &mut DefaultLedgerSimulator,
    proxy_manager_badge: NonFungibleGlobalId,
    oracle_manager_badge: NonFungibleGlobalId,
    resources: &IndexMap<String, ResourceAddress>,
    proxy_address: ComponentAddress,
    oracle_address: ComponentAddress,
) {
    // Set Oracle component address in Proxy
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "set_oracle_address",
            manifest_args!(oracle_address),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proxy_manager_badge]);
    receipt.expect_commit_success();

    // "set_price" and "add_symbol" are protected methods, need to be called directly on the Oracle component
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            oracle_address,
            "add_symbol",
            manifest_args!(resources.get("XRD").unwrap(), "XRD".to_string()),
        )
        .call_method(
            oracle_address,
            "add_symbol",
            manifest_args!(resources.get("USDT").unwrap(), "USDT".to_string()),
        )
        .call_method(
            oracle_address,
            "add_symbol",
            manifest_args!(resources.get("ETH").unwrap(), "ETH".to_string()),
        )
        .call_method(
            oracle_address,
            "set_price",
            manifest_args!("XRD".to_string(), "USDT".to_string(), dec!(20)),
        )
        .call_method(
            oracle_address,
            "set_price",
            manifest_args!("XRD".to_string(), "ETH".to_string(), dec!(30)),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![oracle_manager_badge]);
    receipt.expect_commit_success();
}

fn invoke_oracle_via_proxy_basic(
    ledger: &mut DefaultLedgerSimulator,
    resources: &IndexMap<String, ResourceAddress>,
    proxy_address: ComponentAddress,
    info: &str,
) {
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "proxy_get_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("USDT").unwrap(),
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let price: Option<Decimal> = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(price.unwrap(), dec!(20));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(proxy_address, "proxy_get_oracle_info", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let oracle_info: String = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(&oracle_info, info);
}

fn invoke_oracle_via_generic_proxy(
    ledger: &mut DefaultLedgerSimulator,
    resources: &IndexMap<String, ResourceAddress>,
    proxy_address: ComponentAddress,
    info: &str,
) {
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "proxy_call",
            manifest_args!(
                "get_price",
                to_manifest_value(&(
                    resources.get("XRD").unwrap(),
                    resources.get("USDT").unwrap(),
                ))
                .unwrap()
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let price: Option<Decimal> = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(price.unwrap(), dec!(20));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "proxy_call",
            manifest_args!("get_oracle_info", to_manifest_value(&()).unwrap()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let oracle_info: String = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(&oracle_info, info);
}

fn invoke_oracle_v3_via_generic_proxy(
    ledger: &mut DefaultLedgerSimulator,
    resources: &IndexMap<String, ResourceAddress>,
    proxy_address: ComponentAddress,
    info: &str,
) {
    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "proxy_call",
            manifest_args!(
                "get_address",
                // Note the comma in below tuple reference &(,)
                // Function arguments must be encoded to ManifestValue as a tuple, even if it is
                // just a single argument.
                // Without comma a single argument is encoded with it's native type omitting the
                // tuple.
                to_manifest_value(&("ETH".to_string(),)).unwrap()
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let eth_resource_address: Option<ResourceAddress> = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(
        &eth_resource_address.unwrap(),
        resources.get("ETH").unwrap()
    );

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "proxy_call",
            manifest_args!(
                "get_price",
                to_manifest_value(&("XRD".to_string(), "USDT".to_string())).unwrap()
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let price: Option<Decimal> = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(price.unwrap(), dec!(20));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "proxy_call",
            manifest_args!("get_oracle_info", to_manifest_value(&()).unwrap()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let oracle_info: String = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(&oracle_info, info);
}

#[test]
fn test_proxy_basic_oracle_as_global() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let resources = create_some_resources(&mut ledger);
    let (public_key, _, _account) = ledger.new_account(false);
    let owner_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let proxy_manager_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let oracle_manager_badge = NonFungibleGlobalId::from_public_key(&public_key);

    // Publish and instantiate Oracle Proxy
    let proxy_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        proxy_manager_badge.clone(),
        "oracle_proxy_basic",
        "OracleProxy",
        "instantiate_proxy",
    );

    // Publish and instantiate Oracle v1
    let oracle_v1_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        oracle_manager_badge.clone(),
        "oracle_v1",
        "Oracle",
        "instantiate_global",
    );

    oracle_configure_as_global(
        &mut ledger,
        proxy_manager_badge.clone(),
        oracle_manager_badge.clone(),
        &resources,
        proxy_address,
        oracle_v1_address,
    );

    // Perform some operations on Oracle v1
    invoke_oracle_via_proxy_basic(&mut ledger, &resources, proxy_address, "Oracle v1");

    // Publish and instantiate Oracle v2
    let oracle_v2_address = initialize_package(
        &mut ledger,
        owner_badge,
        oracle_manager_badge.clone(),
        "oracle_v2",
        "Oracle",
        "instantiate_global",
    );

    oracle_configure_as_global(
        &mut ledger,
        proxy_manager_badge,
        oracle_manager_badge,
        &resources,
        proxy_address,
        oracle_v2_address,
    );
    // Perform some operations on Oracle v2
    invoke_oracle_via_proxy_basic(&mut ledger, &resources, proxy_address, "Oracle v2");
}

#[test]
fn test_proxy_generic_oracle_as_global() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let resources = create_some_resources(&mut ledger);
    let (public_key, _, _account) = ledger.new_account(false);
    let owner_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let proxy_manager_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let oracle_manager_badge = NonFungibleGlobalId::from_public_key(&public_key);

    // Publish and instantiate Oracle Proxy
    let proxy_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        proxy_manager_badge.clone(),
        "generic_proxy",
        "GenericProxy",
        "instantiate_proxy",
    );

    // Publish and instantiate Oracle v1
    let oracle_v1_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        oracle_manager_badge.clone(),
        "oracle_v1",
        "Oracle",
        "instantiate_global",
    );

    oracle_configure_as_global(
        &mut ledger,
        proxy_manager_badge.clone(),
        oracle_manager_badge.clone(),
        &resources,
        proxy_address,
        oracle_v1_address,
    );

    // Perform some operations on Oracle v1
    invoke_oracle_via_generic_proxy(&mut ledger, &resources, proxy_address, "Oracle v1");

    // Publish and instantiate Oracle v2
    let oracle_v2_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        oracle_manager_badge.clone(),
        "oracle_v2",
        "Oracle",
        "instantiate_global",
    );

    oracle_configure_as_global(
        &mut ledger,
        proxy_manager_badge.clone(),
        oracle_manager_badge.clone(),
        &resources,
        proxy_address,
        oracle_v2_address,
    );

    // Perform some operations on Oracle v2
    invoke_oracle_via_generic_proxy(&mut ledger, &resources, proxy_address, "Oracle v2");

    // Publish and instantiate Oracle v3
    let oracle_v3_address = initialize_package(
        &mut ledger,
        owner_badge,
        oracle_manager_badge.clone(),
        "oracle_v3",
        "Oracle",
        "instantiate_global",
    );

    oracle_v3_configure_as_global(
        &mut ledger,
        proxy_manager_badge,
        oracle_manager_badge,
        &resources,
        proxy_address,
        oracle_v3_address,
    );

    // Perform some operations on Oracle v3
    // Note that Oracle v3 has different API than v1 and v2
    invoke_oracle_v3_via_generic_proxy(&mut ledger, &resources, proxy_address, "Oracle v3");
}

#[test]
fn test_proxy_basic_oracle_as_owned() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let resources = create_some_resources(&mut ledger);
    let (public_key, _, _account) = ledger.new_account(false);
    let owner_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let proxy_manager_badge = NonFungibleGlobalId::from_public_key(&public_key);

    // Publish and instantiate Oracle Proxy
    let proxy_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        proxy_manager_badge.clone(),
        "oracle_proxy_basic",
        "OracleProxy",
        "instantiate_proxy",
    );

    let oracle_v1_package_address = ledger.publish_package_simple(PackageLoader::get("oracle_v1"));

    oracle_configure_as_owned(
        &mut ledger,
        proxy_manager_badge.clone(),
        &resources,
        proxy_address,
        oracle_v1_package_address,
    );

    // Perform some operations on Oracle v1
    invoke_oracle_via_proxy_basic(&mut ledger, &resources, proxy_address, "Oracle v1");

    let oracle_v2_package_address = ledger.publish_package_simple(PackageLoader::get("oracle_v2"));

    oracle_configure_as_owned(
        &mut ledger,
        proxy_manager_badge.clone(),
        &resources,
        proxy_address,
        oracle_v2_package_address,
    );

    // Perform some operations on Oracle v2
    invoke_oracle_via_proxy_basic(&mut ledger, &resources, proxy_address, "Oracle v2");
}
