use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::prelude::*;
use radix_engine_interface::{metadata, metadata_init, mint_roles};
use radix_engine_tests::common::*;
use radix_rust::ContextualDisplay;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;
use scrypto::prelude::Pow;
use scrypto::NonFungibleData;
use scrypto_test::prelude::LedgerSimulatorBuilder;

macro_rules! replace_variables {
    ($content:expr $(, $a:ident = $b:expr)* ) => {
        $content
            $(.replace(concat!("${", stringify!($a), "}"), &format!("{}", $b)))*
    };
}

#[test]
fn test_allocate_address_and_call_it() {
    run_manifest(|account_address, address_bech32_encoder| {
        let code_blob =
            include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm").to_vec();
        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!("address_allocation/allocate_address.rtm"),
            account_address = account_address.display(address_bech32_encoder),
            package_package_address = PACKAGE_PACKAGE.display(address_bech32_encoder),
            code_blob_hash = hash(&code_blob)
        );
        (manifest, vec![code_blob])
    })
    .expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::AuthTemplateDoesNotExist(..)) => true,
        _ => false,
    });
}

/// An example manifest for transfer of funds between accounts
#[test]
fn transfer_of_funds_to_another_account_succeeds() {
    run_manifest(|this_account_address, address_bech32_encoder| {
        let private_key = Secp256k1PrivateKey::from_u64(12).unwrap();
        let public_key = private_key.public_key();
        let other_account_address =
            ComponentAddress::preallocated_account_from_public_key(&public_key);

        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!("account/resource_transfer.rtm"),
            xrd_resource_address = XRD.display(address_bech32_encoder),
            this_account_address = this_account_address.display(address_bech32_encoder),
            other_account_address = other_account_address.display(address_bech32_encoder)
        );
        (manifest, Vec::new())
    })
    .expect_commit_success();
}

#[test]
fn multi_account_fund_transfer_succeeds() {
    test_manifest_with_additional_accounts(
        3,
        |this_account_address, other_accounts, address_bech32_encoder| {
            let manifest = replace_variables!(
                include_workspace_transaction_examples_str!(
                    "account/multi_account_resource_transfer.rtm"
                ),
                xrd_resource_address = XRD.display(address_bech32_encoder),
                this_account_address = address_bech32_encoder
                    .encode(this_account_address.as_ref())
                    .unwrap(),
                account_a_component_address = other_accounts[0].display(address_bech32_encoder),
                account_b_component_address = other_accounts[1].display(address_bech32_encoder),
                account_c_component_address = other_accounts[2].display(address_bech32_encoder)
            );
            (manifest, Vec::new())
        },
    )
}

/// An example manifest for creating a new fungible resource with no initial supply
#[test]
fn creating_a_fungible_resource_with_no_initial_supply_succeeds() {
    run_manifest(|account_address, address_bech32_encoder| {
        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!(
                "resources/creation/fungible/no_initial_supply.rtm"
            ),
            account_address = account_address.display(address_bech32_encoder)
        );
        (manifest, Vec::new())
    })
    .expect_commit_success();
}

/// An example manifest for creating a new fungible resource with an initial supply
#[test]
fn creating_a_fungible_resource_with_initial_supply_succeeds() {
    run_manifest(|account_address, address_bech32_encoder| {
        let initial_supply = dec!("10000000");

        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!(
                "resources/creation/fungible/with_initial_supply.rtm"
            ),
            initial_supply = initial_supply,
            account_address = account_address.display(address_bech32_encoder)
        );
        (manifest, Vec::new())
    })
    .expect_commit_success();
}

/// An example manifest for creating a new fungible resource with an maximum initial supply
#[test]
fn creating_a_fungible_resource_with_max_initial_supply_succeeds() {
    run_manifest(|account_address, address_bech32_encoder| {
        let initial_supply = Decimal::from_attos(I192::from(2).pow(152));

        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!(
                "resources/creation/fungible/with_initial_supply.rtm"
            ),
            initial_supply = initial_supply,
            account_address = account_address.display(address_bech32_encoder)
        );
        (manifest, Vec::new())
    })
    .expect_commit_success();
}

/// An example manifest for creating a new fungible resource with an exceeded maximum initial supply
#[test]
fn creating_a_fungible_resource_with_exceeded_initial_supply_fails() {
    run_manifest(|account_address, address_bech32_encoder| {
        let initial_supply = Decimal::from_attos(I192::from(2).pow(152) + I192::ONE);

        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!(
                "resources/creation/fungible/with_initial_supply.rtm"
            ),
            initial_supply = initial_supply,
            account_address = account_address.display(address_bech32_encoder)
        );
        (manifest, Vec::new())
    })
    .expect_commit_failure();
}

/// An example manifest for creating a new non-fungible resource with no supply
#[test]
fn creating_a_non_fungible_resource_with_no_initial_supply_succeeds() {
    run_manifest(|account_address, address_bech32_encoder| {
        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!(
                "resources/creation/non_fungible/no_initial_supply.rtm"
            ),
            account_address = account_address.display(address_bech32_encoder)
        );
        (manifest, Vec::new())
    })
    .expect_commit_success();
}

/// An example manifest for creating a new non-fungible resource with an initial supply
#[test]
fn creating_a_non_fungible_resource_with_initial_supply_succeeds() {
    run_manifest(|account_address, address_bech32_encoder| {
        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!(
                "resources/creation/non_fungible/with_initial_supply.rtm"
            ),
            account_address = account_address.display(address_bech32_encoder),
            non_fungible_local_id = "#1#"
        );
        (manifest, Vec::new())
    })
    .expect_commit_success();
}

/// A sample manifest that publishes a package.
#[test]
fn publish_package_succeeds() {
    run_manifest(|account_address, address_bech32_encoder| {
        let code_blob = include_workspace_asset_bytes!("radix-engine", "faucet.wasm").to_vec();

        let manifest = replace_variables!(
            include_workspace_transaction_examples_str!("package/publish.rtm"),
            code_blob_hash = hash(&code_blob),
            account_address = account_address.display(address_bech32_encoder),
            auth_badge_resource_address = XRD.display(address_bech32_encoder),
            auth_badge_non_fungible_local_id = "#1#"
        );
        (manifest, vec![code_blob])
    })
    .expect_commit_success();
}

/// A sample manifest for minting of a fungible resource
#[test]
fn minting_of_fungible_resource_succeeds() {
    test_manifest_with_restricted_minting_resource(
        ResourceType::Fungible { divisibility: 18 },
        |account_address,
         minter_badge_resource_address,
         mintable_fungible_resource_address,
         address_bech32_encoder| {
            let mint_amount = dec!("800");

            let manifest = replace_variables!(
                include_workspace_transaction_examples_str!("resources/mint/fungible/mint.rtm"),
                account_address = account_address.display(address_bech32_encoder),
                mintable_fungible_resource_address =
                    mintable_fungible_resource_address.display(address_bech32_encoder),
                minter_badge_resource_address =
                    minter_badge_resource_address.display(address_bech32_encoder),
                mint_amount = mint_amount
            );
            (manifest, Vec::new())
        },
        true,
    );
}

/// A sample manifest for minting of a fungible resource with maximum mint amount
#[test]
fn minting_of_fungible_resource_max_mint_amount_succeeds() {
    test_manifest_with_restricted_minting_resource(
        ResourceType::Fungible { divisibility: 18 },
        |account_address,
         minter_badge_resource_address,
         mintable_fungible_resource_address,
         address_bech32_encoder| {
            let mint_amount = Decimal::from_attos(I192::from(2).pow(152));

            let manifest = replace_variables!(
                include_workspace_transaction_examples_str!("resources/mint/fungible/mint.rtm"),
                account_address = account_address.display(address_bech32_encoder),
                mintable_fungible_resource_address =
                    mintable_fungible_resource_address.display(address_bech32_encoder),
                minter_badge_resource_address =
                    minter_badge_resource_address.display(address_bech32_encoder),
                mint_amount = mint_amount
            );
            (manifest, Vec::new())
        },
        true,
    );
}

/// A sample manifest for minting of a fungible resource with maximum mint amount
#[test]
fn minting_of_fungible_resource_exceed_max_mint_amount_fails() {
    test_manifest_with_restricted_minting_resource(
        ResourceType::Fungible { divisibility: 18 },
        |account_address,
         minter_badge_resource_address,
         mintable_fungible_resource_address,
         address_bech32_encoder| {
            let mint_amount = Decimal::from_attos(I192::from(2).pow(152) + I192::ONE);

            let manifest = replace_variables!(
                include_workspace_transaction_examples_str!("resources/mint/fungible/mint.rtm"),
                account_address = account_address.display(address_bech32_encoder),
                mintable_fungible_resource_address =
                    mintable_fungible_resource_address.display(address_bech32_encoder),
                minter_badge_resource_address =
                    minter_badge_resource_address.display(address_bech32_encoder),
                mint_amount = mint_amount
            );
            (manifest, Vec::new())
        },
        false,
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
         mintable_non_fungible_resource_address,
         address_bech32_encoder| {
            let manifest = replace_variables!(
                include_workspace_transaction_examples_str!("resources/mint/non_fungible/mint.rtm"),
                account_address = account_address.display(address_bech32_encoder),
                mintable_non_fungible_resource_address =
                    mintable_non_fungible_resource_address.display(address_bech32_encoder),
                minter_badge_resource_address =
                    minter_badge_resource_address.display(address_bech32_encoder),
                non_fungible_local_id = "#1#"
            );
            (manifest, Vec::new())
        },
        true,
    );
}

#[test]
fn changing_default_deposit_rule_succeeds() {
    test_manifest_with_restricted_minting_resource(
        ResourceType::Fungible { divisibility: 18 },
        |account_address,
         minter_badge_resource_address,
         mintable_resource_address,
         address_bech32_encoder| {
            let manifest = replace_variables!(
                include_workspace_transaction_examples_str!("account/deposit_modes.rtm"),
                account_address = account_address.display(address_bech32_encoder),
                first_resource_address = mintable_resource_address.display(address_bech32_encoder),
                second_resource_address =
                    minter_badge_resource_address.display(address_bech32_encoder)
            );
            (manifest, Vec::new())
        },
        true,
    );
}

fn run_manifest<F>(string_manifest_builder: F) -> TransactionReceipt
where
    F: Fn(&ComponentAddress, &AddressBech32Encoder) -> (String, Vec<Vec<u8>>),
{
    // Creating a new test runner
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Creating the account component required for this test
    let (public_key, _, component_address) = ledger.new_account(false);
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let address_bech32_encoder = AddressBech32Encoder::new(&network);

    // Run the function and get the manifest string
    let (manifest_string, blobs) =
        string_manifest_builder(&component_address, &address_bech32_encoder);
    let manifest = compile_manifest_v1(
        &manifest_string,
        &network,
        BlobProvider::new_with_blobs(blobs),
    )
    .expect("Failed to compile manifest from manifest string");

    ledger.execute_manifest(manifest, vec![virtual_badge_non_fungible_global_id])
}

fn test_manifest_with_restricted_minting_resource<F>(
    resource_type: ResourceType,
    string_manifest_builder: F,
    expect_success: bool,
) where
    F: Fn(
        &ComponentAddress,
        &ResourceAddress,
        &ResourceAddress,
        &AddressBech32Encoder,
    ) -> (String, Vec<Vec<u8>>),
{
    // Creating a new test runner
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Creating the account component required for this test
    let (public_key, _, component_address) = ledger.new_account(false);
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let address_bech32_encoder = AddressBech32Encoder::new(&network);

    // Creating the minter badge and the requested resource
    let minter_badge_resource_address =
        ledger.create_fungible_resource(dec!(1), 0, component_address);

    let manifest = match resource_type {
        ResourceType::Fungible { divisibility } => ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                false,
                divisibility,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(require(minter_badge_resource_address));
                        minter_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                None,
            )
            .build(),
        ResourceType::NonFungible { id_type } => ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                id_type,
                false,
                NonFungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(require(minter_badge_resource_address));
                        minter_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                None::<BTreeMap<NonFungibleLocalId, SampleNonFungibleData>>,
            )
            .build(),
    };
    let result = ledger.execute_manifest(manifest, vec![]);
    let mintable_non_fungible_resource_address =
        result.expect_commit(true).new_resource_addresses()[0].clone();

    // Run the function and get the manifest string
    let (manifest_string, blobs) = string_manifest_builder(
        &component_address,
        &minter_badge_resource_address,
        &mintable_non_fungible_resource_address,
        &address_bech32_encoder,
    );
    let manifest = compile_manifest_v1(
        &manifest_string,
        &network,
        BlobProvider::new_with_blobs(blobs),
    )
    .expect("Failed to compile manifest from manifest string");

    ledger
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_global_id])
        .expect_commit(expect_success);
}

fn test_manifest_with_additional_accounts<F>(accounts_required: u16, string_manifest_builder: F)
where
    F: Fn(&ComponentAddress, &[ComponentAddress], &AddressBech32Encoder) -> (String, Vec<Vec<u8>>),
{
    // Creating a new test runner
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Creating the account component required for this test
    let (public_key, _, component_address) = ledger.new_account(false);
    let virtual_badge_non_fungible_global_id = NonFungibleGlobalId::from_public_key(&public_key);

    // Creating the required accounts
    let accounts = (0..accounts_required)
        .map(|_| ledger.new_account(false).2)
        .collect::<Vec<ComponentAddress>>();

    // Defining the network and the bech32 encoder to use
    let network = NetworkDefinition::simulator();
    let address_bech32_encoder = AddressBech32Encoder::new(&network);

    // Run the function and get the manifest string
    let (manifest_string, blobs) =
        string_manifest_builder(&component_address, &accounts, &address_bech32_encoder);
    let manifest = compile_manifest_v1(
        &manifest_string,
        &network,
        BlobProvider::new_with_blobs(blobs),
    )
    .expect("Failed to compile manifest from manifest string");

    ledger
        .execute_manifest(manifest, vec![virtual_badge_non_fungible_global_id])
        .expect_commit_success();
}

#[derive(ScryptoSbor, NonFungibleData, ManifestSbor)]
struct SampleNonFungibleData {}
