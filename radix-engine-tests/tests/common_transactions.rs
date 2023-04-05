use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::*;
use scrypto::NonFungibleData;
use scrypto_unit::TestRunner;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::manifest::compile;
use utils::ContextualDisplay;

macro_rules! replace_variables {
    ($content:expr $(, $a:ident = $b:expr)* ) => {
        $content
            $(.replace(concat!("${", stringify!($a), "}"), &format!("{}", $b)))*
    };
}

/// An example manifest for transfer of funds between accounts
#[test]
fn transfer_of_funds_to_another_account_succeeds() {
    test_manifest(|this_account_address, bech32_encoder| {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(12).unwrap();
        let public_key = private_key.public_key();
        let other_account_address = ComponentAddress::virtual_account_from_public_key(&public_key);

        let manifest = replace_variables!(
            include_str!("../../transaction/examples/account/resource_transfer.rtm"),
            xrd_resource_address = RADIX_TOKEN.display(bech32_encoder),
            this_account_address = this_account_address.display(bech32_encoder),
            other_account_address = other_account_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

#[test]
fn multi_account_fund_transfer_succeeds() {
    test_manifest_with_additional_accounts(
        3,
        |this_account_address, other_accounts, bech32_encoder| {
            let manifest = replace_variables!(
                include_str!(
                    "../../transaction/examples/account/multi_account_resource_transfer.rtm"
                ),
                xrd_resource_address = RADIX_TOKEN.display(bech32_encoder),
                this_account_address = bech32_encoder
                    .encode(this_account_address.as_ref())
                    .unwrap(),
                account_a_component_address = other_accounts[0].display(bech32_encoder),
                account_b_component_address = other_accounts[1].display(bech32_encoder),
                account_c_component_address = other_accounts[2].display(bech32_encoder)
            );
            (manifest, Vec::new())
        },
    )
}

/// An example manifest for creating a new fungible resource with no initial supply
#[test]
fn creating_a_fungible_resource_with_no_initial_supply_succeeds() {
    test_manifest(|account_address, bech32_encoder| {
        let manifest = replace_variables!(
            include_str!(
                "../../transaction/examples/resources/creation/fungible/no_initial_supply.rtm"
            ),
            account_address = account_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new fungible resource with an initial supply
#[test]
fn creating_a_fungible_resource_with_initial_supply_succeeds() {
    test_manifest(|account_address, bech32_encoder| {
        let initial_supply = dec!("10000000");

        let manifest = replace_variables!(
            include_str!(
                "../../transaction/examples/resources/creation/fungible/with_initial_supply.rtm"
            ),
            initial_supply = initial_supply,
            account_address = account_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new non-fungible resource with no supply
#[test]
fn creating_a_non_fungible_resource_with_no_initial_supply_succeeds() {
    test_manifest(|account_address, bech32_encoder| {
        let manifest = replace_variables!(
            include_str!(
                "../../transaction/examples/resources/creation/non_fungible/no_initial_supply.rtm"
            ),
            account_address = account_address.display(bech32_encoder)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new non-fungible resource with an initial supply
#[test]
fn creating_a_non_fungible_resource_with_initial_supply_succeeds() {
    test_manifest(|account_address, bech32_encoder| {
        let manifest = replace_variables!(
            include_str!("../../transaction/examples/resources/creation/non_fungible/with_initial_supply.rtm"),
            account_address =
                account_address.display(bech32_encoder),
                non_fungible_local_id = "#1#"
        );
        (manifest, Vec::new())
    });
}

/// A sample manifest that publishes a package.
#[test]
fn publish_package_succeeds() {
    test_manifest(|account_address, bech32_encoder| {
        let code_blob = include_bytes!("../../assets/faucet.wasm").to_vec();
        let schema_blob = include_bytes!("../../assets/faucet.schema").to_vec();

        let manifest = replace_variables!(
            include_str!("../../transaction/examples/package/publish.rtm"),
            code_blob_hash = hash(&code_blob),
            schema_blob_hash = hash(&schema_blob),
            account_address = account_address.display(bech32_encoder),
            auth_badge_resource_address = RADIX_TOKEN.display(bech32_encoder),
            auth_badge_non_fungible_local_id = "#1#"
        );
        (manifest, vec![code_blob, schema_blob])
    });
}

/// A sample manifest for minting of a fungible resource
#[test]
fn minting_of_fungible_resource_succeeds() {
    test_manifest_with_restricted_minting_resource(
        ResourceType::Fungible { divisibility: 18 },
        |account_address,
         minter_badge_resource_address,
         mintable_resource_address,
         bech32_encoder| {
            let mint_amount = dec!("800");

            let manifest = replace_variables!(
                include_str!("../../transaction/examples/resources/mint/fungible/mint.rtm"),
                account_address = account_address.display(bech32_encoder),
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
            id_type: NonFungibleIdType::Integer,
        },
        |account_address,
         minter_badge_resource_address,
         mintable_resource_address,
         bech32_encoder| {
            let manifest = replace_variables!(
                include_str!("../../transaction/examples/resources/mint/non_fungible/mint.rtm"),
                account_address = account_address.display(bech32_encoder),
                mintable_resource_address = mintable_resource_address.display(bech32_encoder),
                minter_badge_resource_address =
                    minter_badge_resource_address.display(bech32_encoder),
                non_fungible_local_id = "#1#"
            );
            (manifest, Vec::new())
        },
    );
}

fn test_manifest<F>(string_manifest_builder: F)
where
    F: Fn(&ComponentAddress, &Bech32Encoder) -> (String, Vec<Vec<u8>>),
{
    // Creating a new test runner
    let mut test_runner = TestRunner::builder().without_trace().build();

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let bech32_encoder = Bech32Encoder::new(&network);

    // Run the function and get the manifest string
    let (manifest_string, blobs) = string_manifest_builder(&component_address, &bech32_encoder);
    let manifest = compile(&manifest_string, &network, blobs)
        .expect("Failed to compile manifest from manifest string");

    test_runner
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_global_id])
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
    // Creating a new test runner
    let mut test_runner = TestRunner::builder().without_trace().build();

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let bech32_encoder = Bech32Encoder::new(&network);

    // Creating the minter badge and the requested resource
    let minter_badge_resource_address =
        test_runner.create_fungible_resource(dec!("1"), 0, component_address);

    let access_rules = BTreeMap::from([(
        ResourceMethodAuthKey::Mint,
        (
            rule!(require(minter_badge_resource_address)),
            rule!(deny_all),
        ),
    )]);

    let manifest = match resource_type {
        ResourceType::Fungible { divisibility } => ManifestBuilder::new()
            .create_fungible_resource(divisibility, BTreeMap::new(), access_rules, None)
            .build(),
        ResourceType::NonFungible { id_type } => ManifestBuilder::new()
            .create_non_fungible_resource(
                id_type,
                BTreeMap::new(),
                access_rules,
                None::<BTreeMap<NonFungibleLocalId, SampleNonFungibleData>>,
            )
            .build(),
    };
    let result = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
    let mintable_resource_address = result.expect_commit(true).new_resource_addresses()[0].clone();

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
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_global_id])
        .expect_commit_success();
}

fn test_manifest_with_additional_accounts<F>(accounts_required: u16, string_manifest_builder: F)
where
    F: Fn(&ComponentAddress, &[ComponentAddress], &Bech32Encoder) -> (String, Vec<Vec<u8>>),
{
    // Creating a new test runner
    let mut test_runner = TestRunner::builder().without_trace().build();

    // Creating the account component required for this test
    let (public_key, _, component_address) = test_runner.new_account(false);
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

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
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_global_id])
        .expect_commit_success();
}

#[derive(ScryptoSbor, NonFungibleData, ManifestSbor)]
struct SampleNonFungibleData {}
