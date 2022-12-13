use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::{
    hash, require, Bech32Encoder, Blob, ComponentAddress, Decimal, FromPublicKey, HashMap,
    NonFungibleAddress, NonFungibleId, ResourceAddress, ResourceMethodAuthKey, ResourceType,
    ACCOUNT_PACKAGE, FAUCET_COMPONENT, RADIX_TOKEN,
};
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::rule;
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

/// An example manifest for the creation of non-virtual (physical?) accounts
#[test]
fn creating_a_non_virtual_account_succeeds() {
    test_manifest(|_, bech32_encoder| {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(12).unwrap();
        let public_key = private_key.public_key();
        let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

        let manifest = format!(
            include_str!("../../transaction/examples/account/account_creation.rtm"),
            faucet_component_address = FAUCET_COMPONENT.display(bech32_encoder),
            xrd_resource_address = RADIX_TOKEN.display(bech32_encoder),
            account_package_address = ACCOUNT_PACKAGE.display(bech32_encoder),
            virtual_badge_resource_address = virtual_badge_non_fungible_address
                .resource_address()
                .display(bech32_encoder),
            virtual_badge_non_fungible_id =
                hex::encode(&hash(public_key.to_vec()).lower_26_bytes())
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
fn publish_package_with_owner_succeeds() {
    test_manifest_with_owner_badge(
        |account_component_address, owner_badge_non_fungible_address, bech32_encoder| {
            let owner_badge_resource_address = owner_badge_non_fungible_address.resource_address();
            let owner_badge_non_fungible_id = if let NonFungibleId::U32(non_fungible_id) =
                owner_badge_non_fungible_address.non_fungible_id()
            {
                *non_fungible_id
            } else {
                panic!("expected a u32 non-fungible-id");
            };

            // TODO: Update the complex.abi and complex.code files that are used for testing.
            // Using the WASM and ABI from the account blueprint here as they are up to date. The
            // complex.code and complex.abi files from the transaction crate are not.
            let code_blob = include_bytes!("../../assets/account.wasm").to_vec();
            let abi_blob = include_bytes!("../../assets/account.abi").to_vec();

            let manifest = format!(
                include_str!("../../transaction/examples/package/publish_with_owner.rtm"),
                owner_badge_resource_address = owner_badge_resource_address.display(bech32_encoder),
                owner_badge_non_fungible_id = owner_badge_non_fungible_id,
                code_blob_hash = Blob::new(&code_blob),
                abi_blob_hash = Blob::new(&abi_blob),
                account_component_address = account_component_address.display(bech32_encoder)
            );
            (manifest, vec![code_blob, abi_blob])
        },
    );
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(false, &mut store);

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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(false, &mut store);

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let bech32_encoder = Bech32Encoder::new(&network);

    // Creating the minter badge and the requested resource
    let minter_badge_resource_address =
        test_runner.create_fungible_resource("1".into(), 0, component_address);
    let mintable_resource_address = test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new(&network)
                .create_resource(
                    resource_type,
                    HashMap::from([
                        (String::from("name"), String::from("Mintable Resource")),
                        (String::from("symbol"), String::from("MINT")),
                    ]),
                    HashMap::from([(
                        ResourceMethodAuthKey::Mint,
                        (
                            rule!(require(minter_badge_resource_address)),
                            rule!(deny_all),
                        ),
                    )]),
                    None,
                )
                .build(),
            vec![],
        )
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(false, &mut store);

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

fn test_manifest_with_owner_badge<F>(string_manifest_builder: F)
where
    F: Fn(&ComponentAddress, &NonFungibleAddress, &Bech32Encoder) -> (String, Vec<Vec<u8>>),
{
    // Creating the test runner and the substate store
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(false, &mut store);

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_address = NonFungibleAddress::from_public_key(&public_key);

    // Creating a non-fungible resource which we will consider to be the owner badge
    let resource_address = test_runner.create_non_fungible_resource(component_address.clone());
    let owner_badge_non_fungible_address =
        NonFungibleAddress::new(resource_address, NonFungibleId::U32(1));

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let bech32_encoder = Bech32Encoder::new(&network);

    // Run the function and get the manifest string
    let (manifest_string, blobs) = string_manifest_builder(
        &component_address,
        &owner_badge_non_fungible_address,
        &bech32_encoder,
    );
    let manifest = compile(&manifest_string, &network, blobs)
        .expect("Failed to compile manifest from manifest string");

    test_runner
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_address])
        .expect_commit_success();
}
