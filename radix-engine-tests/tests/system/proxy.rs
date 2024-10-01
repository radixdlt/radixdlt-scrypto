use radix_common::prelude::*;
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

fn set_oracle_proxy_component_address(
    ledger: &mut DefaultLedgerSimulator,
    proxy_address: ComponentAddress,
    method_name: &str,
    oracle_address: ComponentAddress,
    proxy_manager_badge: NonFungibleGlobalId,
) {
    // Set Oracle component address in Proxy
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(proxy_address, method_name, manifest_args!(oracle_address))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proxy_manager_badge]);
    receipt.expect_commit_success();
}

fn initialize_oracle_in_oracle_proxy(
    ledger: &mut DefaultLedgerSimulator,
    proxy_address: ComponentAddress,
    oracle_package_address: PackageAddress,
    proxy_manager_badge: NonFungibleGlobalId,
) {
    // Set Oracle package address in Proxy
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "initialize_oracle",
            manifest_args!(oracle_package_address),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proxy_manager_badge]);
    receipt.expect_commit_success();
}

fn set_prices_in_oracle_directly(
    ledger: &mut DefaultLedgerSimulator,
    oracle_address: ComponentAddress,
    resources: &IndexMap<String, ResourceAddress>,
    oracle_manager_badge: NonFungibleGlobalId,
) {
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

fn get_price_in_oracle_directly(
    ledger: &mut DefaultLedgerSimulator,
    oracle_address: ComponentAddress,
    resources: &IndexMap<String, ResourceAddress>,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            oracle_address,
            "get_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("USDT").unwrap(),
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    receipt
}

fn get_price_in_oracle_via_oracle_proxy(
    ledger: &mut DefaultLedgerSimulator,
    proxy_address: ComponentAddress,
    resources: &IndexMap<String, ResourceAddress>,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "get_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("USDT").unwrap(),
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    receipt
}

fn set_prices_in_oracle_via_oracle_proxy(
    ledger: &mut DefaultLedgerSimulator,
    proxy_address: ComponentAddress,
    resources: &IndexMap<String, ResourceAddress>,
    proxy_manager_badge: NonFungibleGlobalId,
) {
    // Set Oracle component address in Proxy
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "set_price",
            manifest_args!(
                resources.get("XRD").unwrap(),
                resources.get("USDT").unwrap(),
                dec!(20)
            ),
        )
        .call_method(
            proxy_address,
            "set_price",
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

fn set_prices_in_oracle_v3_directly(
    ledger: &mut DefaultLedgerSimulator,
    oracle_address: ComponentAddress,
    resources: &IndexMap<String, ResourceAddress>,
    oracle_manager_badge: NonFungibleGlobalId,
) {
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

fn invoke_oracle_via_oracle_proxy(
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
            "get_price",
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
        .call_method(proxy_address, "get_oracle_info", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let oracle_info: String = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(&oracle_info, info);
}

fn get_price_in_oracle_via_generic_proxy(
    ledger: &mut DefaultLedgerSimulator,
    proxy_address: ComponentAddress,
    resources: &IndexMap<String, ResourceAddress>,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_address,
            "call_method",
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
    receipt.expect_commit_success();
    receipt
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
            "call_method",
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
            "call_method",
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
            "call_method",
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
            "call_method",
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
            "call_method",
            manifest_args!("get_oracle_info", to_manifest_value(&()).unwrap()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let oracle_info: String = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(&oracle_info, info);
}

#[test]
fn test_oracle_proxy_with_global() {
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
        "oracle_proxies/oracle_proxy_with_global",
        "OracleProxy",
        "instantiate_and_globalize",
    );

    // Publish and instantiate Oracle v1
    let oracle_v1_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        oracle_manager_badge.clone(),
        "oracles/oracle_v1",
        "Oracle",
        "instantiate_and_globalize",
    );

    set_oracle_proxy_component_address(
        &mut ledger,
        proxy_address,
        "set_oracle_address",
        oracle_v1_address,
        proxy_manager_badge.clone(),
    );

    set_prices_in_oracle_directly(
        &mut ledger,
        oracle_v1_address,
        &resources,
        oracle_manager_badge.clone(),
    );

    // Perform some operations on Oracle v1
    invoke_oracle_via_oracle_proxy(&mut ledger, &resources, proxy_address, "Oracle v1");

    // Publish and instantiate Oracle v2
    let oracle_v2_address = initialize_package(
        &mut ledger,
        owner_badge,
        oracle_manager_badge.clone(),
        "oracles/oracle_v2",
        "Oracle",
        "instantiate_and_globalize",
    );

    set_oracle_proxy_component_address(
        &mut ledger,
        proxy_address,
        "set_oracle_address",
        oracle_v2_address,
        proxy_manager_badge,
    );

    set_prices_in_oracle_directly(
        &mut ledger,
        oracle_v2_address,
        &resources,
        oracle_manager_badge,
    );
    // Perform some operations on Oracle v2
    invoke_oracle_via_oracle_proxy(&mut ledger, &resources, proxy_address, "Oracle v2");
}

#[test]
fn test_oracle_generic_proxy_with_global() {
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
        "oracle_proxies/oracle_generic_proxy_with_global",
        "OracleGenericProxy",
        "instantiate_and_globalize",
    );

    // Publish and instantiate Oracle v1
    let oracle_v1_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        oracle_manager_badge.clone(),
        "oracles/oracle_v1",
        "Oracle",
        "instantiate_and_globalize",
    );

    set_oracle_proxy_component_address(
        &mut ledger,
        proxy_address,
        "set_component_address",
        oracle_v1_address,
        proxy_manager_badge.clone(),
    );

    set_prices_in_oracle_directly(
        &mut ledger,
        oracle_v1_address,
        &resources,
        oracle_manager_badge.clone(),
    );

    // Perform some operations on Oracle v1
    invoke_oracle_via_generic_proxy(&mut ledger, &resources, proxy_address, "Oracle v1");

    // Publish and instantiate Oracle v2
    let oracle_v2_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        oracle_manager_badge.clone(),
        "oracles/oracle_v2",
        "Oracle",
        "instantiate_and_globalize",
    );

    set_oracle_proxy_component_address(
        &mut ledger,
        proxy_address,
        "set_component_address",
        oracle_v2_address,
        proxy_manager_badge.clone(),
    );

    set_prices_in_oracle_directly(
        &mut ledger,
        oracle_v2_address,
        &resources,
        oracle_manager_badge.clone(),
    );

    // Perform some operations on Oracle v2
    invoke_oracle_via_generic_proxy(&mut ledger, &resources, proxy_address, "Oracle v2");

    // Publish and instantiate Oracle v3
    let oracle_v3_address = initialize_package(
        &mut ledger,
        owner_badge,
        oracle_manager_badge.clone(),
        "oracles/oracle_v3",
        "Oracle",
        "instantiate_and_globalize",
    );

    set_oracle_proxy_component_address(
        &mut ledger,
        proxy_address,
        "set_component_address",
        oracle_v3_address,
        proxy_manager_badge,
    );

    set_prices_in_oracle_v3_directly(
        &mut ledger,
        oracle_v3_address,
        &resources,
        oracle_manager_badge,
    );

    // Perform some operations on Oracle v3
    // Note that Oracle v3 has different API than v1 and v2
    invoke_oracle_v3_via_generic_proxy(&mut ledger, &resources, proxy_address, "Oracle v3");
}

#[test]
fn test_oracle_proxy_with_owned() {
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
        "oracle_proxies/oracle_proxy_with_owned",
        "OracleProxy",
        "instantiate_and_globalize",
    );

    let oracle_v1_package_address =
        ledger.publish_package_simple(PackageLoader::get("oracles/oracle_v1"));

    initialize_oracle_in_oracle_proxy(
        &mut ledger,
        proxy_address,
        oracle_v1_package_address,
        proxy_manager_badge.clone(),
    );

    set_prices_in_oracle_via_oracle_proxy(
        &mut ledger,
        proxy_address,
        &resources,
        proxy_manager_badge.clone(),
    );

    // Perform some operations on Oracle v1
    invoke_oracle_via_oracle_proxy(&mut ledger, &resources, proxy_address, "Oracle v1");

    let oracle_v2_package_address =
        ledger.publish_package_simple(PackageLoader::get("oracles/oracle_v2"));

    initialize_oracle_in_oracle_proxy(
        &mut ledger,
        proxy_address,
        oracle_v2_package_address,
        proxy_manager_badge.clone(),
    );

    set_prices_in_oracle_via_oracle_proxy(
        &mut ledger,
        proxy_address,
        &resources,
        proxy_manager_badge,
    );

    // Perform some operations on Oracle v2
    invoke_oracle_via_oracle_proxy(&mut ledger, &resources, proxy_address, "Oracle v2");
}

#[test]
fn test_oracle_proxy_costing_overhead() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let resources = create_some_resources(&mut ledger);
    let (public_key, _, _account) = ledger.new_account(false);
    let owner_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let proxy_manager_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let oracle_manager_badge = NonFungibleGlobalId::from_public_key(&public_key);

    // Publish and instantiate Oracle v1
    let oracle_v1_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        oracle_manager_badge.clone(),
        "oracles/oracle_v1",
        "Oracle",
        "instantiate_and_globalize",
    );
    set_prices_in_oracle_directly(
        &mut ledger,
        oracle_v1_address,
        &resources,
        oracle_manager_badge,
    );

    // Oracle Proxy Global
    let oracle_proxy_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        proxy_manager_badge.clone(),
        "oracle_proxies/oracle_proxy_with_global",
        "OracleProxy",
        "instantiate_and_globalize",
    );
    set_oracle_proxy_component_address(
        &mut ledger,
        oracle_proxy_address,
        "set_oracle_address",
        oracle_v1_address,
        proxy_manager_badge.clone(),
    );

    // Generic Proxy Global
    let generic_proxy_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        proxy_manager_badge.clone(),
        "oracle_proxies/oracle_generic_proxy_with_global",
        "OracleGenericProxy",
        "instantiate_and_globalize",
    );
    set_oracle_proxy_component_address(
        &mut ledger,
        generic_proxy_address,
        "set_component_address",
        oracle_v1_address,
        proxy_manager_badge.clone(),
    );

    // Oracle Proxy Owned
    let oracle_proxy_owned_address = initialize_package(
        &mut ledger,
        owner_badge.clone(),
        proxy_manager_badge.clone(),
        "oracle_proxies/oracle_proxy_with_owned",
        "OracleProxy",
        "instantiate_and_globalize",
    );
    let oracle_v1_package_address =
        ledger.publish_package_simple(PackageLoader::get("oracles/oracle_v1"));
    initialize_oracle_in_oracle_proxy(
        &mut ledger,
        oracle_proxy_owned_address,
        oracle_v1_package_address,
        proxy_manager_badge.clone(),
    );
    set_prices_in_oracle_via_oracle_proxy(
        &mut ledger,
        oracle_proxy_owned_address,
        &resources,
        proxy_manager_badge.clone(),
    );

    let receipt_oracle_v1 =
        get_price_in_oracle_directly(&mut ledger, oracle_v1_address, &resources);
    let receipt_oracle_proxy =
        get_price_in_oracle_via_oracle_proxy(&mut ledger, oracle_proxy_address, &resources);
    let receipt_generic_proxy =
        get_price_in_oracle_via_generic_proxy(&mut ledger, generic_proxy_address, &resources);
    let receipt_oracle_proxy_owned =
        get_price_in_oracle_via_oracle_proxy(&mut ledger, oracle_proxy_owned_address, &resources);
    println!(
        "get_price Oracle v1 total_cost: {:?}",
        receipt_oracle_v1.fee_summary.total_cost()
    );
    println!(
        "get_price Oracle proxy total_cost: {:?} diff: {:?}",
        receipt_oracle_proxy.fee_summary.total_cost(),
        receipt_oracle_proxy.fee_summary.total_cost() - receipt_oracle_v1.fee_summary.total_cost()
    );
    println!(
        "get_price generic proxy total_cost: {:?} diff: {:?}",
        receipt_generic_proxy.fee_summary.total_cost(),
        receipt_generic_proxy.fee_summary.total_cost() - receipt_oracle_v1.fee_summary.total_cost()
    );
    println!(
        "get_price Oracle proxy owned total_cost: {:?} diff: {:?}",
        receipt_oracle_proxy_owned.fee_summary.total_cost(),
        receipt_oracle_proxy_owned.fee_summary.total_cost()
            - receipt_oracle_v1.fee_summary.total_cost()
    );

    // 2024-02-26: According to above results proxy call cost should be less than 0.19
    assert!(
        (receipt_oracle_proxy.fee_summary.total_cost()
            - receipt_oracle_v1.fee_summary.total_cost())
            < dec!("0.19")
    );
    assert!(
        (receipt_generic_proxy.fee_summary.total_cost()
            - receipt_oracle_v1.fee_summary.total_cost())
            < dec!("0.19")
    );
    assert!(
        (receipt_oracle_proxy_owned.fee_summary.total_cost()
            - receipt_oracle_v1.fee_summary.total_cost())
            < dec!("0.19")
    );
}
