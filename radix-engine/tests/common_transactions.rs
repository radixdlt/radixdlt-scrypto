use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::{
    hash, Bech32Encoder, Blob, ComponentAddress, Decimal, FromPublicKey, NonFungibleAddress,
    NonFungibleId, ResourceAddress, ACCOUNT_PACKAGE, FAUCET_COMPONENT, RADIX_TOKEN,
};
use radix_engine_interface::core::NetworkDefinition;
use scrypto_unit::TestRunner;
use transaction::manifest::compile;
use transaction::signing::EcdsaSecp256k1PrivateKey;

/// An example manifest for freeing some funds from the faucet
#[test]
fn free_funds_from_faucet_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            r#"
        # Locking 10 XRD in fees from the account component. 
        CALL_METHOD 
            ComponentAddress("{account_component_address}") 
            "lock_fee"
            Decimal("10");
    
        # Calling the "free" method on the faucet component which is the method responsible for 
        # dispensing funds from the faucet.
        CALL_METHOD 
            ComponentAddress("{faucet_component_address}") 
            "free";
    
        # Depositing all of the XRD dispensed from the faucet into our account component.
        CALL_METHOD
            ComponentAddress("{account_component_address}") 
            "deposit_batch"
            Expression("ENTIRE_WORKTOP");
        "#,
            faucet_component_address =
                bech32_encoder.encode_component_address_to_string(&FAUCET_COMPONENT),
            account_component_address =
                bech32_encoder.encode_component_address_to_string(&account_component_address)
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
            r#"
        # Locking 10 XRD in fees from the testnet's faucet
        CALL_METHOD 
            ComponentAddress("{faucet_component_address}") 
            "lock_fee"
            Decimal("10");
    
        # Calling the "free" method on the faucet component which is the method responsible for 
        # dispensing funds from the faucet. 
        CALL_METHOD 
            ComponentAddress("{faucet_component_address}") 
            "free";

        # Take the XRD from the worktop and into a bucket. In this case, we would like to deposit 
        # these funds into the account as soon as it's created (in the same function call).
        TAKE_FROM_WORKTOP 
            ResourceAddress("{xrd_resource_address}") 
            Bucket("bucket1");
    
        # Creating a new account
        CALL_FUNCTION 
            PackageAddress("{account_package_address}") 
            "Account" 
            "new_with_resource" 
            Enum(
                "Protected", 
                Enum(
                    "ProofRule", 
                    Enum(
                        "Require", 
                        Enum(
                            "StaticNonFungible", 
                            NonFungibleAddress(
                                "{virtual_badge_resource_address}", 
                                Bytes("{virtual_badge_non_fungible_id}")
                            )
                        )
                    )
                )
            )
            Bucket("bucket1");
        "#,
            faucet_component_address =
                bech32_encoder.encode_component_address_to_string(&FAUCET_COMPONENT),
            xrd_resource_address = bech32_encoder.encode_resource_address_to_string(&RADIX_TOKEN),
            account_package_address =
                bech32_encoder.encode_package_address_to_string(&ACCOUNT_PACKAGE),
            virtual_badge_resource_address = bech32_encoder.encode_resource_address_to_string(
                &virtual_badge_non_fungible_address.resource_address()
            ),
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
            r#"
        # The account component withdraw methods which have been optimized to also lock a fee in a 
        # single call. In this call, we lock a fee of 10 XRD and also withdraw 100 XRD from the 
        # account.
        CALL_METHOD 
            ComponentAddress("{this_account_component_address}") 
            "lock_fee_and_withdraw_by_amount"
            Decimal("10")                                   # Amount of XRD to lock for fees
            Decimal("100")                                  # Amount of XRD to withdraw
            ResourceAddress("{xrd_resource_address}");
    
        # Depositing all of the XRD withdrawn from the account into the other account
        CALL_METHOD
            ComponentAddress("{other_account_component_address}") 
            "deposit_batch"
            Expression("ENTIRE_WORKTOP");
        "#,
            xrd_resource_address = bech32_encoder.encode_resource_address_to_string(&RADIX_TOKEN),
            this_account_component_address =
                bech32_encoder.encode_component_address_to_string(&this_account_component_address),
            other_account_component_address =
                bech32_encoder.encode_component_address_to_string(&other_account_component_address),
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new fungible resource with no initial supply
#[test]
fn creating_a_fungible_resource_with_no_initial_supply_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            r#"
        # Locking 10 XRD in fees from the account component. 
        CALL_METHOD 
            ComponentAddress("{account_component_address}") 
            "lock_fee"
            Decimal("10");
    
        # Creating a new resource with a divisibility of 18 and a name of `MyResource`. The resource
        # has default resource behavior where it can be withdrawn and deposited by anybody.
        CREATE_RESOURCE 
            Enum("Fungible", 18u8) 
            Array<Tuple>(
                Tuple("name", "MyResource"), 
                Tuple("symbol", "RSRC"),
                Tuple("description", "A very innovative and important resource")
            ) 
            Array<Tuple>(
                Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))),
                Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))
            )
            None;
        "#,
            account_component_address =
                bech32_encoder.encode_component_address_to_string(&account_component_address)
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
            r#"
        # Locking 10 XRD in fees from the account component. 
        CALL_METHOD 
            ComponentAddress("{account_component_address}") 
            "lock_fee"
            Decimal("10");
    
        # Creating a new resource with a divisibility of 18 and a name of `MyResource`. The resource
        # has default resource behavior where it can be withdrawn and deposited by anybody.
        CREATE_RESOURCE 
            Enum("Fungible", 18u8) 
            Array<Tuple>(
                Tuple("name", "MyResource"), 
                Tuple("symbol", "RSRC"),
                Tuple("description", "A very innovative and important resource")
            ) 
            Array<Tuple>(
                Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))),
                Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))
            )
            Some(Enum("Fungible", Decimal("{initial_supply}")));
    
        # Depositing the entirety of the initial supply of the newly created resource into our 
        # account component.
        CALL_METHOD
            ComponentAddress("{account_component_address}") 
            "deposit_batch"
            Expression("ENTIRE_WORKTOP");
        "#,
            initial_supply = initial_supply,
            account_component_address =
                bech32_encoder.encode_component_address_to_string(&account_component_address)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new non-fungible resource with no supply
#[test]
fn creating_a_non_fungible_resource_with_no_initial_supply_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            r#"
        # Locking 10 XRD in fees from the account component. 
        CALL_METHOD 
            ComponentAddress("{account_component_address}") 
            "lock_fee"
            Decimal("10");
    
        # Creating a new resource 
        CREATE_RESOURCE 
            Enum(
                "NonFungible", 
                Enum("U32")
            ) 
            Array<Tuple>(
                Tuple("name", "MyResource"), 
                Tuple("symbol", "RSRC"),
                Tuple("description", "A very innovative and important resource"), 
            ) 
            Array<Tuple>(
                Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))),
                Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))
            )
            None;
        "#,
            account_component_address =
                bech32_encoder.encode_component_address_to_string(&account_component_address)
        );
        (manifest, Vec::new())
    });
}

/// An example manifest for creating a new non-fungible resource with an initial supply
#[test]
fn creating_a_non_fungible_resource_with_initial_supply_succeeds() {
    test_manifest(|account_component_address, bech32_encoder| {
        let manifest = format!(
            r#"
        # Locking 10 XRD in fees from the account component. 
        CALL_METHOD 
            ComponentAddress("{account_component_address}") 
            "lock_fee"
            Decimal("10");
    
        # Creating a new resource 
        CREATE_RESOURCE 
            Enum(
                "NonFungible", 
                Enum("U32")
            ) 
            Array<Tuple>(
                Tuple("name", "MyResource"), 
                Tuple("symbol", "RSRC"),
                Tuple("description", "A very innovative and important resource"), 
            ) 
            Array<Tuple>(
                Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))),
                Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))
            )
            Some(
                Enum(
                    "NonFungible", 
                    Array<Tuple>(
                        Tuple(NonFungibleId(1u32), Tuple(Bytes("5c2100"), Bytes("5c2100")))
                    )
                )
            );
    
        # Depositing the entirety of the initial supply of the newly created resource into our 
        # account component.
        CALL_METHOD
            ComponentAddress("{account_component_address}") 
            "deposit_batch"
            Expression("ENTIRE_WORKTOP");
        "#,
            account_component_address =
                bech32_encoder.encode_component_address_to_string(&account_component_address)
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
                r#"
            # Locking 10 XRD in fees from the account component. 
            CALL_METHOD 
                ComponentAddress("{account_component_address}") 
                "lock_fee"
                Decimal("10");
        
            # Publishing a new package with an owner badge
            PUBLISH_PACKAGE_WITH_OWNER 
                Blob("{code_blob_hash}")
                Blob("{abi_blob_hash}")
                NonFungibleAddress("{owner_badge_resource_address}", {owner_badge_non_fungible_id}u32);
            "#,
                owner_badge_resource_address =
                    bech32_encoder.encode_resource_address_to_string(&owner_badge_resource_address),
                owner_badge_non_fungible_id = owner_badge_non_fungible_id,
                code_blob_hash = Blob::new(&code_blob),
                abi_blob_hash = Blob::new(&abi_blob),
                account_component_address =
                    bech32_encoder.encode_component_address_to_string(&account_component_address)
            );
            (manifest, vec![code_blob, abi_blob])
        },
    );
}

/// A sample manifest for minting of a fungible resource
#[test]
fn minting_of_fungible_resource_succeeds() {
    test_manifest_with_mintable_resource(
        |account_component_address, mintable_resource_address, bech32_encoder| {
            let mint_amount = Decimal::from("800");

            let manifest = format!(
                r#"
            # Locking 10 XRD in fees from the account component. 
            CALL_METHOD 
                ComponentAddress("{account_component_address}") 
                "lock_fee"
                Decimal("10");
        
            # Minting 800 tokens from the mintable fungible resource
            MINT_FUNGIBLE 
                ResourceAddress("{mintable_resource_address}")
                Decimal("{mint_amount}");

            # Depositing the entirety of the newly minted tokens into out account
            CALL_METHOD
                ComponentAddress("{account_component_address}") 
                "deposit_batch"
                Expression("ENTIRE_WORKTOP");
            "#,
                account_component_address =
                    bech32_encoder.encode_component_address_to_string(&account_component_address),
                mintable_resource_address =
                    bech32_encoder.encode_resource_address_to_string(&mintable_resource_address),
                mint_amount = mint_amount
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

fn test_manifest_with_mintable_resource<F>(string_manifest_builder: F)
where
    F: Fn(&ComponentAddress, &ResourceAddress, &Bech32Encoder) -> (String, Vec<Vec<u8>>),
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

    // Creating a new mintable resource.
    let mintable_resource_address =
        test_runner.create_mintable_fungible_resource(Decimal::from("0"), 18, component_address);

    // Run the function and get the manifest string
    let (manifest_string, blobs) = string_manifest_builder(
        &component_address,
        &mintable_resource_address,
        &bech32_encoder,
    );
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
