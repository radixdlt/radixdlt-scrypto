use radix_engine::utils::{validate_call_arguments_to_native_components, ValidationError};
use radix_engine_common::prelude::NetworkDefinition;
use scrypto::prelude::*;
use transaction::{
    manifest::{compile, MockBlobProvider},
    prelude::ManifestBuilder,
    validation::EcdsaSecp256k1PrivateKey,
};
use walkdir::WalkDir;

#[test]
fn validator_sees_valid_transfer_manifest_as_valid() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account1(), RADIX_TOKEN, dec!("10"))
        .try_deposit_batch_or_abort(account2())
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest.instructions);

    // Assert
    assert!(validation_result.is_ok())
}

#[test]
fn validator_sees_invalid_transfer_manifest_as_invalid() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .call_method(account1(), "withdraw", manifest_args!())
        .try_deposit_batch_or_abort(account2())
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest.instructions);

    // Assert
    assert!(is_schema_validation_error(validation_result))
}

#[test]
fn validator_invalidates_calls_to_unknown_methods_on_a_known_blueprint() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .call_method(account1(), "my_made_up_method", manifest_args!())
        .try_deposit_batch_or_abort(account2())
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest.instructions);

    // Assert
    assert!(is_method_not_found(validation_result))
}

#[test]
fn common_manifests_are_all_valid() {
    // Arrange
    let path = "../transaction";
    for entry in WalkDir::new(path) {
        let path = entry.unwrap().path().canonicalize().unwrap();

        if path.extension().and_then(|str| str.to_str()) != Some("rtm") {
            continue;
        }

        let manifest_string = std::fs::read_to_string(&path)
            .map(|str| apply_replacements(&str))
            .unwrap();
        let manifest = compile(
            &manifest_string,
            &NetworkDefinition::simulator(),
            MockBlobProvider::new(),
        )
        .unwrap();

        // Act
        let validation_result =
            validate_call_arguments_to_native_components(&manifest.instructions);

        // Uncomment to see which manifest failed exactly.
        // if validation_result.is_err() {
        //     println!("{path:?}");
        //     println!("{validation_result:?}");
        // }

        // Assert
        assert!(validation_result.is_ok())
    }
}

fn is_schema_validation_error<T>(result: Result<T, ValidationError>) -> bool {
    if let Err(error) = result {
        matches!(error, ValidationError::SchemaValidationError(..))
    } else {
        false
    }
}

fn is_method_not_found<T>(result: Result<T, ValidationError>) -> bool {
    if let Err(error) = result {
        matches!(error, ValidationError::MethodNotFound(..))
    } else {
        false
    }
}

fn private_key1() -> EcdsaSecp256k1PrivateKey {
    EcdsaSecp256k1PrivateKey::from_u64(1).unwrap()
}

fn private_key2() -> EcdsaSecp256k1PrivateKey {
    EcdsaSecp256k1PrivateKey::from_u64(2).unwrap()
}

fn account1() -> ComponentAddress {
    ComponentAddress::virtual_account_from_public_key(&private_key1().public_key())
}

fn account2() -> ComponentAddress {
    ComponentAddress::virtual_account_from_public_key(&private_key2().public_key())
}

fn apply_replacements(string: &str) -> String {
    string
        .replace(
            "${xrd_resource_address}",
            "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3",
        )
        .replace(
            "${fungible_resource_address}",
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        )
        .replace(
            "${resource_address}",
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        )
        .replace(
            "${gumball_resource_address}",
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        )
        .replace(
            "${non_fungible_resource_address}",
            "resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha",
        )
        .replace(
            "${badge_resource_address}",
            "resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha",
        )
        .replace(
            "${account_address}",
            "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
        )
        .replace(
            "${this_account_address}",
            "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
        )
        .replace(
            "${account_a_component_address}",
            "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
        )
        .replace(
            "${account_b_component_address}",
            "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
        )
        .replace(
            "${account_c_component_address}",
            "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
        )
        .replace(
            "${other_account_address}",
            "account_sim1cyzfj6p254jy6lhr237s7pcp8qqz6c8ahq9mn6nkdjxxxat5syrgz9",
        )
        .replace(
            "${component_address}",
            "component_sim1cqvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvemygpmu",
        )
        .replace(
            "${faucet_component_address}",
            "component_sim1cqvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvemygpmu",
        )
        .replace(
            "${package_address}",
            "package_sim1p4r4955skdjq9swg8s5jguvcjvyj7tsxct87a9z6sw76cdfd2jg3zk",
        )
        .replace(
            "${minter_badge_resource_address}",
            "resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha",
        )
        .replace(
            "${mintable_resource_address}",
            "resource_sim1nfhtg7ttszgjwysfglx8jcjtvv8q02fg9s2y6qpnvtw5jsy3wvlhj6",
        )
        .replace(
            "${mintable_fungible_resource_address}",
            "resource_sim1thcgx0f3rwaeetl67cmsssv4p748kd3sjhtge9l4m6ns7cucs97tjv",
        )
        .replace(
            "${second_resource_address}",
            "resource_sim1nfhtg7ttszgjwysfglx8jcjtvv8q02fg9s2y6qpnvtw5jsy3wvlhj6",
        )
        .replace(
            "${mintable_non_fungible_resource_address}",
            "resource_sim1nfhtg7ttszgjwysfglx8jcjtvv8q02fg9s2y6qpnvtw5jsy3wvlhj6",
        )
        .replace(
            "${vault_address}",
            "internal_vault_sim1tqvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvevp72ff",
        )
        .replace("${owner_badge_non_fungible_local_id}", "#1#")
        .replace(
            "${code_blob_hash}",
            "5b4b01a4a3892ea3751793da57f072ae08eec694ddcda872239fc8239e4bcd1b",
        )
        .replace("${initial_supply}", "12")
        .replace("${mint_amount}", "12")
        .replace("${non_fungible_local_id}", "#12#")
        .replace(
            "${auth_badge_resource_address}",
            "resource_sim1n24hvnrgmhj6j8dpjuu85vfsagdjafcl5x4ewc9yh436jh2hpu4qdj",
        )
        .replace("${auth_badge_non_fungible_local_id}", "#1#")
        .replace(
            "${package_address}",
            "package_sim1p4r4955skdjq9swg8s5jguvcjvyj7tsxct87a9z6sw76cdfd2jg3zk",
        )
        .replace(
            "${consensusmanager_address}",
            "consensusmanager_sim1scxxxxxxxxxxcnsmgrxxxxxxxxx000999665565xxxxxxxxxxc06cl",
        )
        .replace(
            "${clock_address}",
            "clock_sim1skxxxxxxxxxxclckxxxxxxxxxxx002253583992xxxxxxxxxx58hk6",
        )
        .replace(
            "${validator_address}",
            "validator_sim1sgvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvedzgr3l",
        )
        .replace(
            "${accesscontroller_address}",
            "accesscontroller_sim1cvvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvexaj7at",
        )
}
