use radix_engine::types::{
    require, BTreeMap, Bech32Encoder, Blob, ComponentAddress, Decimal, FromPublicKey,
    NonFungibleAddress, NonFungibleId, ResourceAddress, ResourceMethodAuthKey, ResourceType,
    FAUCET_COMPONENT, RADIX_TOKEN,
};
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::rule;
use scrypto::NonFungibleData;
use scrypto_unit::TestRunner;
use transaction::builder::ManifestBuilder;
use transaction::manifest::compile;
use transaction::signing::EcdsaSecp256k1PrivateKey;
use utils::ContextualDisplay;

/// An example manifest for freeing some funds from the faucet
#[test]
fn free_funds_from_faucet_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            include_str!("../../transaction/examples/faucet/free_funds.rtm"),
            faucet_component_address = FAUCET_COMPONENT.display(bech32_encoder),
            account_component_address = account_component_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for transfer of funds between accounts
#[test]
fn transfer_of_funds_to_another_account_succeeds() {
    test_manifest(|this_account_component_address, bech32_encoder| {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(12).unwrap();
        let public_key = private_key.public_key();
        let other_account_component_address =
            ComponentAddress::virtual_account_from_public_key(&public_key);

        let manifest = format!(
            include_str!("../../transaction/examples/account/resource_transfer.rtm"),
            xrd_resource_address = RADIX_TOKEN.display(bech32_encoder),
            this_account_component_address = this_account_component_address.display(bech32_encoder),
            other_account_component_address =
                other_account_component_address.display(bech32_encoder),
        );
        (manifest, Vec::new())
    });
}

#[test]
fn multi_account_fund_transfer_succeeds() {
    test_manifest_with_additional_accounts(
        3,
        |this_account_component_address, other_accounts, bech32_encoder| {
            let manifest = format!(
                include_str!(
                    "../../transaction/examples/account/multi_account_resource_transfer.rtm"
                ),
                xrd_resource_address = RADIX_TOKEN.display(bech32_encoder),
                this_account_component_address = bech32_encoder
                    .encode_component_address_to_string(&this_account_component_address),
                account_a_component_address = other_accounts[0].display(bech32_encoder),
                account_b_component_address = other_accounts[1].display(bech32_encoder),
                account_c_component_address = other_accounts[2].display(bech32_encoder),
            );
            (manifest, Vec::new())
        },
    )
}

/// An example manifest for creating a new fungible resource with no initial supply
#[test]
fn creating_a_fungible_resource_with_no_initial_supply_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            include_str!(
                "../../transaction/examples/resources/creation/fungible/no_initial_supply.rtm"
            ),
            account_component_address = account_component_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new fungible resource with an initial supply
#[test]
fn creating_a_fungible_resource_with_initial_supply_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let initial_supply = Decimal::from("10000000");

        let manifest = format!(
            include_str!(
                "../../transaction/examples/resources/creation/fungible/with_initial_supply.rtm"
            ),
            initial_supply = initial_supply,
            account_component_address = account_component_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new non-fungible resource with no supply
#[test]
fn creating_a_non_fungible_resource_with_no_initial_supply_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            include_str!(
                "../../transaction/examples/resources/creation/non_fungible/no_initial_supply.rtm"
            ),
            account_component_address = account_component_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new non-fungible resource with an initial supply
#[test]
fn creating_a_non_fungible_resource_with_initial_supply_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            include_str!("../../transaction/examples/resources/creation/non_fungible/with_initial_supply.rtm"),
            account_component_address =
                account_component_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// A sample manifest that publishes a package.
#[test]
fn publish_package_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        // TODO: Update the code.blob and abi.blob files that are used for testing.
        // Using the WASM and ABI from the account blueprint here as they are up to date. The
        // abi.blob and code.blob files from the transaction crate are not.
        let code_blob = include_bytes!("../../assets/account.wasm").to_vec();
        let abi_blob = include_bytes!("../../assets/account.abi").to_vec();

        let manifest = format!(
            include_str!("../../transaction/examples/package/publish.rtm"),
            code_blob_hash = Blob::new(&code_blob),
            abi_blob_hash = Blob::new(&abi_blob),
            account_component_address = account_component_address.display(bech32_encoder)
        );
        (manifest, vec![code_blob, abi_blob])
    });
}

/// A sample manifest for minting of a fungible resource
#[test]
fn minting_of_fungible_resource_succeeds() {
    test_manifest_with_restricted_minting_resource(
        ResourceType::Fungible { divisibility: 18 },
        |account_component_address,
         minter_badge_resource_address,
         mintable_resource_address,
         bech32_encoder| {
            let mint_amount = Decimal::from("800");

            let manifest = format!(
                include_str!("../../transaction/examples/resources/mint/fungible/mint.rtm"),
                account_component_address = account_component_address.display(bech32_encoder),
                mintable_resource_address = mintable_resource_address.display(bech32_encoder),
                minter_badge_resource_address =
                    minter_badge_resource_address.display(bech32_encoder),
                mint_amount = mint_amount
            );
            (manifest, Vec::new())
        },
    );
}

/// A sample manifest for minting of a non-fungible resource
#[test]
fn minting_of_non_fungible_resource_succeeds() {
    test_manifest_with_restricted_minting_resource(
        ResourceType::NonFungible {
            id_type: radix_engine::types::NonFungibleIdType::U32,
        },
        |account_component_address,
         minter_badge_resource_address,
         mintable_resource_address,
         bech32_encoder| {
            let manifest = format!(
                include_str!("../../transaction/examples/resources/mint/non_fungible/mint.rtm"),
                account_component_address = account_component_address.display(bech32_encoder),
                mintable_resource_address = mintable_resource_address.display(bech32_encoder),
                minter_badge_resource_address =
                    minter_badge_resource_address.display(bech32_encoder),
                non_fungible_id = "1u32"
            );
            (manifest, Vec::new())
        },
    );
}

fn test_manifest<F>(string_manifest_builder: F)
where
    F: Fn(&ComponentAddress, &Bech32Encoder) -> (String, Vec<Vec<u8>>),
{
    // Creating the test runner and the substate store
    let mut test_runner = TestRunner::new(false);

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let bech32_encoder = Bech32Encoder::new(&network);

    // Run the function and get the manifest string
    let (manifest_string, blobs) = string_manifest_builder(&component_address, &bech32_encoder);
    let manifest = compile(&manifest_string, &network, blobs)
        .expect("Failed to compile manifest from manifest string");

    test_runner
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_address])
        .expect_commit_success();
}

fn test_manifest_with_restricted_minting_resource<F>(
    resource_type: ResourceType,
    string_manifest_builder: F,
) where
    F: Fn(
        &ComponentAddress,
        &ResourceAddress,
        &ResourceAddress,
        &Bech32Encoder,
    ) -> (String, Vec<Vec<u8>>),
{
    // Creating the test runner and the substate store
    let mut test_runner = TestRunner::new(false);

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let bech32_encoder = Bech32Encoder::new(&network);

    // Creating the minter badge and the requested resource
    let minter_badge_resource_address =
        test_runner.create_fungible_resource("1".into(), 0, component_address);

    let access_rules = BTreeMap::from([(
        ResourceMethodAuthKey::Mint,
        (
            rule!(require(minter_badge_resource_address)),
            rule!(deny_all),
        ),
    )]);

    let manifest = match resource_type {
        ResourceType::Fungible { divisibility } => ManifestBuilder::new(&network)
            .create_fungible_resource(divisibility, BTreeMap::new(), access_rules, None)
            .build(),
        ResourceType::NonFungible { id_type } => ManifestBuilder::new(&network)
            .create_non_fungible_resource(
                id_type,
                BTreeMap::new(),
                access_rules,
                None::<BTreeMap<NonFungibleId, SampleNonFungibleData>>,
            )
            .build(),
    };
    let mintable_resource_address = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![])
        .new_resource_addresses()[0]
        .clone();

    // Run the function and get the manifest string
    let (manifest_string, blobs) = string_manifest_builder(
        &component_address,
        &minter_badge_resource_address,
        &mintable_resource_address,
        &bech32_encoder,
    );
    let manifest = compile(&manifest_string, &network, blobs)
        .expect("Failed to compile manifest from manifest string");

    test_runner
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_address])
        .expect_commit_success();
}

fn test_manifest_with_additional_accounts<F>(accounts_required: u16, string_manifest_builder: F)
where
    F: Fn(&ComponentAddress, &[ComponentAddress], &Bech32Encoder) -> (String, Vec<Vec<u8>>),
{
    // Creating the test runner and the substate store
    let mut test_runner = TestRunner::new(false);

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    // Creating the required accounts
    let accounts = (0..accounts_required)
        .map(|_| test_runner.new_account(false).2)
        .collect::<Vec<ComponentAddress>>();

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let bech32_encoder = Bech32Encoder::new(&network);

    // Run the function and get the manifest string
    let (manifest_string, blobs) =
        string_manifest_builder(&component_address, &accounts, &bech32_encoder);
    let manifest = compile(&manifest_string, &network, blobs)
        .expect("Failed to compile manifest from manifest string");

    test_runner
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_address])
        .expect_commit_success();
}

#[derive(NonFungibleData)]
struct SampleNonFungibleData {}
