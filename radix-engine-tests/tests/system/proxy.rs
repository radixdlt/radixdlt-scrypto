use radix_engine_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn instantiate_package(
    test_runner: &mut DefaultTestRunner,
    package_address: PackageAddress,
    blueprint_name: &str,
    function_name: &str,
) -> ComponentAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            blueprint_name,
            function_name,
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let my_component = receipt.expect_commit(true).new_component_addresses()[0];
    my_component
}

fn create_some_resources(test_runner: &mut DefaultTestRunner) -> Vec<ResourceAddress> {
    let (_public_key, _, account_address) = test_runner.new_account(false);
    let resources = (0..4)
        .into_iter()
        .map(|_| test_runner.create_fungible_resource(dec!(20000), 18, account_address))
        .collect();
    resources
}

fn act_on_oracle(
    test_runner: &mut DefaultTestRunner,
    resources: &[ResourceAddress],
    proxy_component_address: ComponentAddress,
    oracle_component_address: ComponentAddress,
    info: &str,
) {
    // Set Oracle component address in Proxy
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_component_address,
            "set_component_address",
            manifest_args!(oracle_component_address),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_component_address,
            "proxy_set_price",
            manifest_args!(resources[0], resources[1], dec!(20)),
        )
        .call_method(
            proxy_component_address,
            "proxy_set_price",
            manifest_args!(resources[0], resources[2], dec!(30)),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_component_address,
            "proxy_get_price",
            manifest_args!(resources[0], resources[1]),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let price: Option<Decimal> = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(price.unwrap(), dec!(20));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            proxy_component_address,
            "proxy_get_oracle_info",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let oracle_info: String = receipt.expect_commit_success().output(1);

    // Assert
    assert_eq!(&oracle_info, info);
}

#[test]
fn test_proxy() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let resources = create_some_resources(&mut test_runner);

    const ORACLE_PACKAGE_ADDRESS: [u8; NodeId::LENGTH] = [
        13, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
    ];
    let oracle_package_address = PackageAddress::new_or_panic(ORACLE_PACKAGE_ADDRESS);

    // Instantiate Oracle v1 at predefined package address.
    // Oracle package must be published before Proxy package is published, because the Oracle's
    // package address is validated during Proxy package publishing.
    test_runner.publish_package_at_address(PackageLoader::get("oracle_v1"), oracle_package_address);

    let proxy_package_address =
        test_runner.publish_package_simple(PackageLoader::get("oracle_proxy"));
    // Instantiate Oracle Proxy
    let proxy_component_address = instantiate_package(
        &mut test_runner,
        proxy_package_address,
        "Proxy",
        "instantiate_proxy",
    );

    // Instantiate Oracle v1
    let oracle_v1_component_address = instantiate_package(
        &mut test_runner,
        oracle_package_address,
        "Oracle",
        "instantiate_global",
    );

    act_on_oracle(
        &mut test_runner,
        &resources,
        proxy_component_address,
        oracle_v1_component_address,
        "Oracle v1",
    );
}
