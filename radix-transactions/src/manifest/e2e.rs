use radix_common::prelude::{AddressBech32Encoder, PACKAGE_PACKAGE};
use radix_rust::ContextualDisplay;

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::internal_prelude::*;
    use crate::manifest::*;
    use compiler::*;
    use radix_engine_interface::blueprints::resource::RoleAssignmentInit;
    use radix_engine_interface::blueprints::resource::{NonFungibleResourceRoles, OwnerRole};
    use radix_engine_interface::object_modules::ModuleConfig;
    use radix_engine_interface::{metadata, metadata_init};
    use scrypto_derive::NonFungibleData;

    #[test]
    fn test_address_allocation() {
        compile_and_decompile_with_inversion_test_v1(
            "address_allocation",
            apply_address_replacements(include_str!(
                "../../examples/address_allocation/allocate_address.rtm"
            )),
            &NetworkDefinition::simulator(),
            vec![include_bytes!("../../examples/package/code.wasm").to_vec()],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
ALLOCATE_GLOBAL_ADDRESS
    Address("${package_package_address}")
    "Package"
    AddressReservation("my_reservation")
    NamedAddress("my_package")
;
PUBLISH_PACKAGE_ADVANCED
    Enum<0u8>()
    Tuple(
        Map<String, Tuple>()
    )
    Blob("${code_blob_hash}")
    Map<String, Tuple>()
    Enum<1u8>(
        AddressReservation("my_reservation")
    )
;
CALL_FUNCTION
    NamedAddress("my_package")
    "BlueprintName"
    "no_such_function"
    Decimal("1")
    NamedAddress("my_package")
;
"##,
            ),
        );
    }

    #[test]
    fn test_publish_package() {
        compile_and_decompile_with_inversion_test_v1(
            "publish_package",
            apply_address_replacements(include_str!("../../examples/package/publish.rtm")),
            &NetworkDefinition::simulator(),
            vec![include_bytes!("../../examples/package/code.wasm").to_vec()],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("5000")
;
PUBLISH_PACKAGE_ADVANCED
    Enum<0u8>()
    Tuple(
        Map<String, Tuple>()
    )
    Blob("${code_blob_hash}")
    Map<String, Tuple>()
    Enum<0u8>()
;
"##,
            ),
        );
    }

    #[test]
    fn test_resource_worktop() {
        compile_and_decompile_with_inversion_test_v1(
            "resource_worktop",
            apply_address_replacements(include_str!("../../examples/resources/worktop.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "withdraw"
    Address("${xrd_resource_address}")
    Decimal("5")
;
TAKE_FROM_WORKTOP
    Address("${xrd_resource_address}")
    Decimal("2")
    Bucket("xrd")
;
CALL_METHOD
    Address("${component_address}")
    "buy_gumball"
    Bucket("xrd")
;
ASSERT_WORKTOP_CONTAINS_ANY
    Address("${gumball_resource_address}")
;
ASSERT_WORKTOP_CONTAINS
    Address("${gumball_resource_address}")
    Decimal("3")
;
TAKE_ALL_FROM_WORKTOP
    Address("${xrd_resource_address}")
    Bucket("some_xrd")
;
RETURN_TO_WORKTOP
    Bucket("some_xrd")
;
TAKE_NON_FUNGIBLES_FROM_WORKTOP
    Address("${non_fungible_resource_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#1#")
    )
    Bucket("nfts")
;
CALL_METHOD
    Address("${account_address}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
"##,
            ),
        );
    }

    #[test]
    fn test_resource_auth_zone() {
        compile_and_decompile_with_inversion_test_v1(
            "resource_auth_zone",
            apply_address_replacements(include_str!("../../examples/resources/auth_zone.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q")
    "withdraw"
    Address("resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3")
    Decimal("5")
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3")
    Bucket("some_xrd")
;
CREATE_PROOF_FROM_BUCKET_OF_AMOUNT
    Bucket("some_xrd")
    Decimal("1")
    Proof("proof1a")
;
CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES
    Bucket("some_xrd")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#123#")
    )
    Proof("proof1b")
;
CREATE_PROOF_FROM_BUCKET_OF_ALL
    Bucket("some_xrd")
    Proof("proof1c")
;
CLONE_PROOF
    Proof("proof1c")
    Proof("proof1d")
;
DROP_PROOF
    Proof("proof1d")
;
DROP_PROOF
    Proof("proof1c")
;
DROP_AUTH_ZONE_PROOFS;
CALL_METHOD
    Address("account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q")
    "create_proof_of_amount"
    Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")
    Decimal("5")
;
POP_FROM_AUTH_ZONE
    Proof("proof3")
;
PUSH_TO_AUTH_ZONE
    Proof("proof3")
;
CALL_METHOD
    Address("account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q")
    "create_proof_of_amount"
    Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")
    Decimal("5")
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT
    Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")
    Decimal("1")
    Proof("proof4")
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
    Address("resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#123#")
    )
    Proof("proof5")
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL
    Address("resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha")
    Proof("proof6")
;
DROP_AUTH_ZONE_SIGNATURE_PROOFS;
DROP_AUTH_ZONE_REGULAR_PROOFS;
DROP_AUTH_ZONE_PROOFS;
DROP_NAMED_PROOFS;
DROP_ALL_PROOFS;
CALL_METHOD
    Address("account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
"##,
            ),
        );
    }

    #[test]
    fn test_resource_recall() {
        compile_and_decompile_with_inversion_test_v1(
            "resource_recall",
            apply_address_replacements(include_str!("../../examples/resources/recall.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
RECALL_FROM_VAULT
    Address("${vault_address}")
    Decimal("1.2")
;
"##,
            ),
        );
    }

    #[test]
    fn test_vault_freeze() {
        compile_and_decompile_with_inversion_test_v1(
            "vault_freeze",
            apply_address_replacements(include_str!("../../examples/resources/freeze.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
FREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        1u32
    )
;
FREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        2u32
    )
;
FREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        4u32
    )
;
FREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        7u32
    )
;
UNFREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        1u32
    )
;
UNFREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        2u32
    )
;
UNFREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        4u32
    )
;
UNFREEZE_VAULT
    Address("${vault_address}")
    Tuple(
        7u32
    )
;
"##,
            ),
        );
    }

    #[test]
    fn test_call_function() {
        compile_and_decompile_with_inversion_test_v1(
            "call_function",
            apply_address_replacements(include_str!("../../examples/call/call_function.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_FUNCTION
    Address("${package_address}")
    "BlueprintName"
    "f"
    "string"
;
"##,
            ),
        );
    }

    #[test]
    fn test_resource_recall_nonfungibles() {
        compile_and_decompile_with_inversion_test_v1(
            "resource_recall_nonfungibles",
            apply_address_replacements(include_str!(
                "../../examples/resources/recall_non_fungibles.rtm"
            )),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
RECALL_NON_FUNGIBLES_FROM_VAULT
    Address("${vault_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#123#"),
        NonFungibleLocalId("#456#")
    )
;
"##,
            ),
        );
    }

    #[test]
    fn test_call_method() {
        compile_and_decompile_with_inversion_test_v1(
            "call_method",
            apply_address_replacements(include_str!("../../examples/call/call_method.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${component_address}")
    "complicated_method"
    Decimal("1")
    PreciseDecimal("2")
;
SET_COMPONENT_ROYALTY
    Address("${component_address}")
    "my_method"
    Enum<0u8>()
;
CALL_METADATA_METHOD
    Address("${component_address}")
    "get"
    "HelloWorld"
;
CALL_ROLE_ASSIGNMENT_METHOD
    Address("${component_address}")
    "get"
    Enum<0u8>()
    "hello"
;
CALL_DIRECT_VAULT_METHOD
    Address("${vault_address}")
    "unknown"
    123u8
;
"##,
            ),
        );
    }

    #[test]
    fn test_values() {
        compile_and_decompile_with_inversion_test_v1(
            "values",
            apply_address_replacements(include_str!("../../examples/values/values.rtm")),
            &NetworkDefinition::simulator(),
            vec![include_bytes!("../../examples/package/code.wasm").to_vec()],
            apply_address_replacements(
                r##"
TAKE_ALL_FROM_WORKTOP
    Address("${resource_address}")
    Bucket("temp1")
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL
    Address("${resource_address}")
    Proof("temp2")
;
CALL_METHOD
    Address("${component_address}")
    "aliases"
    Enum<0u8>()
    Enum<0u8>()
    Enum<1u8>(
        "hello"
    )
    Enum<1u8>(
        "hello"
    )
    Enum<0u8>(
        "test"
    )
    Enum<0u8>(
        "test"
    )
    Enum<1u8>(
        "test123"
    )
    Enum<1u8>(
        "test123"
    )
    Enum<0u8>()
    Enum<1u8>(
        "a"
    )
    Enum<0u8>(
        "b"
    )
    Enum<1u8>(
        "c"
    )
    Bytes("deadbeef")
    Bytes("050aff")
    NonFungibleGlobalId("${non_fungible_resource_address}:<value>")
    NonFungibleGlobalId("${non_fungible_resource_address}:#123#")
    NonFungibleGlobalId("${non_fungible_resource_address}:#456#")
    NonFungibleGlobalId("${non_fungible_resource_address}:[031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f]")
    NonFungibleGlobalId("${non_fungible_resource_address}:#1234567890#")
    NonFungibleGlobalId("${non_fungible_resource_address}:#1#")
    Array<Array>(
        Bytes("dead"),
        Bytes("050aff")
    )
    Array<Array>(
        Bytes("dead"),
        Bytes("050aff")
    )
    Array<Tuple>(
        NonFungibleGlobalId("${non_fungible_resource_address}:<value>"),
        NonFungibleGlobalId("${non_fungible_resource_address}:#1#")
    )
    Array<Tuple>(
        NonFungibleGlobalId("${non_fungible_resource_address}:<value>"),
        NonFungibleGlobalId("${non_fungible_resource_address}:#1#")
    )
    Array<Enum>(
        Enum<1u8>(
            "hello"
        )
    )
    Array<Enum>(
        Enum<1u8>(),
        Enum<0u8>()
    )
    Array<Map>(
        Map<U8, U16>()
    )
    Map<U8, U16>(
        1u8 => 5u16
    )
;
CALL_METHOD
    Address("${component_address}")
    "custom_types"
    "normal text"
    "with an escape\""
    "with an escape\r"
    "with an escape\t"
    "with an escape\n"
    "with an escape\""
    "with an escape\\"
    Address("${package_address}")
    Address("${account_address}")
    Address("${consensusmanager_address}")
    Address("${validator_address}")
    Address("${accesscontroller_address}")
    Bucket("temp1")
    Proof("temp2")
    Expression("ENTIRE_WORKTOP")
    Blob("a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0")
    Decimal("1.2")
    PreciseDecimal("1.2")
    NonFungibleLocalId("<SomeId>")
    NonFungibleLocalId("#12#")
    NonFungibleLocalId("[031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f]")
    NonFungibleLocalId("{1111111111111111-1111111111111111-1111111111111111-1111111111111111}")
;
"##,
            ),
        );
    }

    #[test]
    fn test_royalty() {
        compile_and_decompile_with_inversion_test_v1(
            "royalty",
            apply_address_replacements(include_str!("../../examples/royalty/royalty.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
SET_COMPONENT_ROYALTY
    Address("${account_address}")
    "my_method"
    Enum<0u8>()
;
LOCK_COMPONENT_ROYALTY
    Address("${account_address}")
    "my_method"
;
CLAIM_PACKAGE_ROYALTIES
    Address("${package_address}")
;
CLAIM_COMPONENT_ROYALTIES
    Address("${account_address}")
;
"##,
            ),
        );
    }

    #[test]
    fn test_metadata() {
        compile_and_decompile_with_inversion_test_v1(
            "metadata",
            apply_address_replacements(include_str!("../../examples/metadata/metadata.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
SET_METADATA
    Address("${package_address}")
    "field_name"
    Enum<0u8>(
        "Metadata string value, eg description"
    )
;
SET_METADATA
    Address("${account_address}")
    "field_name"
    Enum<0u8>(
        "Metadata string value, eg description"
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<0u8>(
        "Metadata string value, eg description"
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<1u8>(
        true
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<2u8>(
        123u8
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<3u8>(
        123u32
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<4u8>(
        123u64
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<5u8>(
        -123i32
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<6u8>(
        -123i64
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<7u8>(
        Decimal("10.5")
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<8u8>(
        Address("${account_address}")
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<9u8>(
        Enum<0u8>(
            Bytes("0000000000000000000000000000000000000000000000000000000000000000ff")
        )
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<10u8>(
        NonFungibleGlobalId("${non_fungible_resource_address}:<some_string>")
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<11u8>(
        NonFungibleLocalId("<some_string>")
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<12u8>(
        10000i64
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<13u8>(
        "https://radixdlt.com/index.html"
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<14u8>(
        "https://radixdlt.com"
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<15u8>(
        Enum<0u8>(
            Bytes("0000000000000000000000000000000000000000000000000000000000")
        )
    )
;
SET_METADATA
    Address("${resource_address}")
    "field_name"
    Enum<128u8>(
        Array<String>(
            "some_string",
            "another_string",
            "yet_another_string"
        )
    )
;
LOCK_METADATA
    Address("${package_address}")
    "field_name"
;
LOCK_METADATA
    Address("${account_address}")
    "field_name"
;
LOCK_METADATA
    Address("${resource_address}")
    "field_name"
;
REMOVE_METADATA
    Address("${package_address}")
    "field_name"
;
REMOVE_METADATA
    Address("${account_address}")
    "field_name"
;
REMOVE_METADATA
    Address("${resource_address}")
    "field_name"
;
"##,
            ),
        );
    }

    #[test]
    fn test_update_role() {
        compile_and_decompile_with_inversion_test_v1(
            "access_rule",
            apply_address_replacements(include_str!("../../examples/access_rule/access_rule.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
SET_OWNER_ROLE
    Address("${resource_address}")
    Enum<0u8>()
;
LOCK_OWNER_ROLE
    Address("${resource_address}")
;
SET_ROLE
    Address("${resource_address}")
    Enum<0u8>()
    "hello"
    Enum<0u8>()
;
"##,
            ),
        );
    }

    #[test]
    fn test_create_fungible_resource_with_initial_supply() {
        compile_and_decompile_with_inversion_test_v1(
            "create_fungible_resource_with_initial_supply",
            apply_address_replacements(
                include_str!("../../examples/resources/creation/fungible/with_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
    Enum<0u8>()
    true
    18u8
    Decimal("12")
    Tuple(
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<0u8>()
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        ),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>()
    )
    Tuple(
        Map<String, Tuple>(
            "name" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "MyResource"
                    )
                ),
                true
            )
        ),
        Map<String, Enum>(
            "metadata_setter" => Enum<1u8>(
                Enum<0u8>()
            ),
            "metadata_setter_updater" => Enum<0u8>(),
            "metadata_locker" => Enum<1u8>(
                Enum<1u8>()
            ),
            "metadata_locker_updater" => Enum<0u8>()
        )
    )
    Enum<0u8>()
;
CALL_METHOD
    Address("${account_address}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
"##,
            ),
        );
    }

    #[test]
    fn test_create_fungible_resource_with_no_initial_supply() {
        compile_and_decompile_with_inversion_test_v1(
            "create_fungible_resource_with_no_initial_supply",
            apply_address_replacements(
                include_str!("../../examples/resources/creation/fungible/no_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CREATE_FUNGIBLE_RESOURCE
    Enum<0u8>()
    true
    18u8
    Tuple(
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<0u8>()
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        ),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>()
    )
    Tuple(
        Map<String, Tuple>(
            "name" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "MyResource"
                    )
                ),
                true
            )
        ),
        Map<String, Enum>(
            "metadata_setter" => Enum<1u8>(
                Enum<0u8>()
            ),
            "metadata_setter_updater" => Enum<0u8>(),
            "metadata_locker" => Enum<1u8>(
                Enum<1u8>()
            ),
            "metadata_locker_updater" => Enum<0u8>()
        )
    )
    Enum<0u8>()
;
"##,
            ),
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_initial_supply() {
        compile_and_decompile_with_inversion_test_v1(
            "create_non_fungible_resource_with_initial_supply",
            apply_address_replacements(
                include_str!(
                    "../../examples/resources/creation/non_fungible/with_initial_supply.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
    Enum<0u8>()
    Enum<1u8>()
    true
    Enum<0u8>(
        Enum<0u8>(
            Tuple(
                Array<Enum>(),
                Array<Tuple>(),
                Array<Enum>()
            )
        ),
        Enum<0u8>(
            66u8
        ),
        Array<String>()
    )
    Map<NonFungibleLocalId, Tuple>(
        NonFungibleLocalId("#12#") => Tuple(
            Tuple()
        )
    )
    Tuple(
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<0u8>()
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        ),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>()
    )
    Tuple(
        Map<String, Tuple>(
            "name" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "MyResource"
                    )
                ),
                true
            )
        ),
        Map<String, Enum>(
            "metadata_setter" => Enum<1u8>(
                Enum<0u8>()
            ),
            "metadata_setter_updater" => Enum<0u8>(),
            "metadata_locker" => Enum<1u8>(
                Enum<1u8>()
            ),
            "metadata_locker_updater" => Enum<0u8>()
        )
    )
    Enum<0u8>()
;
CALL_METHOD
    Address("${account_address}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
"##,
            ),
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_no_initial_supply() {
        compile_and_decompile_with_inversion_test_v1(
            "create_non_fungible_resource_with_no_initial_supply",
            apply_address_replacements(
                include_str!(
                    "../../examples/resources/creation/non_fungible/no_initial_supply.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CREATE_NON_FUNGIBLE_RESOURCE
    Enum<0u8>()
    Enum<1u8>()
    true
    Enum<0u8>(
        Enum<0u8>(
            Tuple(
                Array<Enum>(),
                Array<Tuple>(),
                Array<Enum>()
            )
        ),
        Enum<0u8>(
            66u8
        ),
        Array<String>()
    )
    Tuple(
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<0u8>()
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        ),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>()
    )
    Tuple(
        Map<String, Tuple>(
            "name" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "MyResource"
                    )
                ),
                true
            )
        ),
        Map<String, Enum>(
            "metadata_setter" => Enum<1u8>(
                Enum<0u8>()
            ),
            "metadata_setter_updater" => Enum<0u8>(),
            "metadata_locker" => Enum<1u8>(
                Enum<1u8>()
            ),
            "metadata_locker_updater" => Enum<0u8>()
        )
    )
    Enum<0u8>()
;
"##,
            ),
        );
    }

    #[test]
    fn test_mint_fungible() {
        compile_and_decompile_with_inversion_test_v1(
            "mint_fungible",
            apply_address_replacements(include_str!(
                "../../examples/resources/mint/fungible/mint.rtm"
            )),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CALL_METHOD
    Address("${account_address}")
    "create_proof_of_amount"
    Address("${minter_badge_resource_address}")
    Decimal("1")
;
MINT_FUNGIBLE
    Address("${mintable_fungible_resource_address}")
    Decimal("12")
;
CALL_METHOD
    Address("${account_address}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
"##,
            ),
        );
    }

    #[test]
    fn test_mint_non_fungible() {
        compile_and_decompile_with_inversion_test_v1(
            "mint_non_fungible",
            apply_address_replacements(include_str!(
                "../../examples/resources/mint/non_fungible/mint.rtm"
            )),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CALL_METHOD
    Address("${account_address}")
    "create_proof_of_amount"
    Address("${minter_badge_resource_address}")
    Decimal("1")
;
MINT_NON_FUNGIBLE
    Address("${mintable_non_fungible_resource_address}")
    Map<NonFungibleLocalId, Tuple>(
        NonFungibleLocalId("${non_fungible_local_id}") => Tuple(
            Tuple()
        )
    )
;
CALL_METHOD
    Address("${account_address}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
"##,
            ),
        );
    }

    #[test]
    fn test_create_account() {
        compile_and_decompile_with_inversion_test_v1(
            "create_account",
            apply_address_replacements(include_str!("../../examples/account/new.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CREATE_ACCOUNT_ADVANCED
    Enum<2u8>(
        Enum<0u8>()
    )
;
CREATE_ACCOUNT;
"##,
            ),
        );
    }

    #[test]
    fn test_create_validator() {
        compile_and_decompile_with_inversion_test_v1(
            "create_validator",
            apply_address_replacements(include_str!("../../examples/validator/new.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CALL_METHOD
    Address("${this_account_address}")
    "withdraw"
    Address("${xrd_resource_address}")
    Decimal("1000")
;
TAKE_FROM_WORKTOP
    Address("${xrd_resource_address}")
    Decimal("1000")
    Bucket("xrd_payment")
;
CREATE_VALIDATOR
    Bytes("02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
    Decimal("1")
    Bucket("xrd_payment")
;
"##,
            ),
        );
    }

    #[test]
    fn test_create_identity() {
        compile_and_decompile_with_inversion_test_v1(
            "create_identity",
            apply_address_replacements(include_str!("../../examples/identity/new.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
CREATE_IDENTITY_ADVANCED
    Enum<0u8>()
;
CREATE_IDENTITY;
"##,
            ),
        );
    }

    #[test]
    fn test_create_access_controller() {
        compile_and_decompile_with_inversion_test_v1(
            "create_access_controller",
            apply_address_replacements(include_str!("../../examples/access_controller/new.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            apply_address_replacements(
                r##"
TAKE_ALL_FROM_WORKTOP
    Address("${badge_resource_address}")
    Bucket("some_xrd")
;
CREATE_ACCESS_CONTROLLER
    Bucket("some_xrd")
    Tuple(
        Enum<1u8>(),
        Enum<1u8>(),
        Enum<1u8>()
    )
    Enum<0u8>()
;
"##,
            ),
        );
    }

    #[test]
    fn test_simple_transfer() {
        // Note - this test is intended for demonstration for the ledger
        // app - but might be better moved to being a transaction scenario
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CALL_METHOD
    Address("${account_address}")
    "withdraw"
    Address("${fungible_resource_address}")
    Decimal("123")
;
TAKE_FROM_WORKTOP
    Address("${fungible_resource_address}")
    Decimal("123")
    Bucket("bucket1")
;
CALL_METHOD
    Address("${other_account_address}")
    "try_deposit_or_abort"
    Bucket("bucket1")
;
        "##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "simple_transfer",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_simple_transfer_new_format() {
        // Note - this test is intended for demonstration for the ledger
        // app - but might be better moved to being a transaction scenario
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee_and_withdraw"
    Decimal("25")
    Address("${xrd_resource_address}")
    Decimal("123")
;
TAKE_FROM_WORKTOP
    Address("${xrd_resource_address}")
    Decimal("123")
    Bucket("bucket1")
;
CALL_METHOD
    Address("${other_account_address}")
    "try_deposit_or_abort"
    Bucket("bucket1")
;
        "##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "simple_transfer_new_format",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_simple_transfer_with_multiple_locked_fees() {
        // Note - this test is intended for demonstration for the ledger
        // app - but might be better moved to being a transaction scenario
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("1.2")
;
CALL_METHOD
    Address("${account_address}")
    "withdraw"
    Address("${xrd_resource_address}")
    Decimal("123")
;
TAKE_FROM_WORKTOP
    Address("${xrd_resource_address}")
    Decimal("123")
    Bucket("bucket1")
;
CALL_METHOD
    Address("${other_account_address}")
    "try_deposit_or_abort"
    Bucket("bucket1")
;
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("3.4")
;
        "##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "simple_transfer_with_multiple_locked_fees",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_simple_transfer_nft() {
        // Note - this test is intended for demonstration for the ledger
        // app - but might be better moved to being a transaction scenario
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CALL_METHOD
    Address("${account_address}")
    "withdraw_non_fungibles"
    Address("${non_fungible_resource_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#1#"),
        NonFungibleLocalId("#2#")
    )
;
TAKE_FROM_WORKTOP
    Address("${non_fungible_resource_address}")
    Decimal("2")
    Bucket("bucket1")
;
CALL_METHOD
    Address("${other_account_address}")
    "try_deposit_or_abort"
    Bucket("bucket1")
;
"##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "simple_transfer_nft",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_simple_transfer_nft_new_format() {
        // Note - this test is intended for demonstration for the ledger
        // app - but might be better moved to being a transaction scenario
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee_and_withdraw_non_fungibles"
    Decimal("500")
    Address("${non_fungible_resource_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#1#"),
        NonFungibleLocalId("#2#")
    )
;
TAKE_FROM_WORKTOP
    Address("${non_fungible_resource_address}")
    Decimal("2")
    Bucket("bucket1")
;
CALL_METHOD
    Address("${other_account_address}")
    "try_deposit_or_abort"
    Bucket("bucket1")
;
"##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "simple_transfer_nft_new_format",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_simple_transfer_nft_by_id() {
        // Note - this test is intended for demonstration for the ledger
        // app - but might be better moved to being a transaction scenario
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee"
    Decimal("500")
;
CALL_METHOD
    Address("${account_address}")
    "withdraw_non_fungibles"
    Address("${non_fungible_resource_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#1#"),
        NonFungibleLocalId("#2#"),
        NonFungibleLocalId("#3#")
    )
;
TAKE_NON_FUNGIBLES_FROM_WORKTOP
    Address("${non_fungible_resource_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#1#"),
        NonFungibleLocalId("#2#"),
        NonFungibleLocalId("#3#")
    )
    Bucket("bucket1")
;
CALL_METHOD
    Address("${other_account_address}")
    "try_deposit_or_abort"
    Bucket("bucket1")
;
"##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "simple_transfer_nft_by_id",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_simple_transfer_nft_by_id_new_format() {
        // Note - this test is intended for demonstration for the ledger
        // app - but might be better moved to being a transaction scenario
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "lock_fee_and_withdraw_non_fungibles"
    Decimal("500")
    Address("${non_fungible_resource_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#1#"),
        NonFungibleLocalId("#2#"),
        NonFungibleLocalId("#3#")
    )
;
TAKE_NON_FUNGIBLES_FROM_WORKTOP
    Address("${non_fungible_resource_address}")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#1#"),
        NonFungibleLocalId("#2#"),
        NonFungibleLocalId("#3#")
    )
    Bucket("bucket1")
;
CALL_METHOD
    Address("${other_account_address}")
    "try_deposit_or_abort"
    Bucket("bucket1")
;
"##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "simple_transfer_nft_by_id_new_format",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_subintent_v2_with_children() {
        let canonical_manifest = apply_address_replacements(
            r##"
USE_CHILD
    NamedIntent("my_child")
    Intent("subtxid_sim1ree59h2u2sguzl6g72pn7q9hpe3r28l95c05f2rfe7cgfp4sgmwqx5l3mu")
;
ASSERT_WORKTOP_IS_EMPTY;
TAKE_ALL_FROM_WORKTOP
    Address("${resource_address}")
    Bucket("bucket")
;
ASSERT_WORKTOP_RESOURCES_ONLY
    Map<Address, Enum>(
        Address("${resource_address}") => Enum<0u8>()
    )
;
ASSERT_WORKTOP_RESOURCES_INCLUDE
    Map<Address, Enum>(
        Address("${fungible_resource_address}") => Enum<1u8>(
            Decimal("1")
        ),
        Address("${non_fungible_resource_address}") => Enum<2u8>(
            Decimal("2")
        )
    )
;
ASSERT_NEXT_CALL_RETURNS_ONLY
    Map<Address, Enum>(
        Address("${non_fungible_resource_address}") => Enum<3u8>(
            Array<NonFungibleLocalId>(
                NonFungibleLocalId("#234#")
            )
        )
    )
;
ASSERT_NEXT_CALL_RETURNS_INCLUDE
    Map<Address, Enum>(
        Address("${non_fungible_resource_address}") => Enum<4u8>(
            Array<NonFungibleLocalId>(
                NonFungibleLocalId("<My_Id>")
            )
        )
    )
;
ASSERT_BUCKET_CONTENTS
    Bucket("bucket")
    Enum<5u8>(
        Tuple(
            Array<NonFungibleLocalId>(),
            Enum<0u8>(),
            Enum<0u8>(
                Decimal("123")
            ),
            Enum<1u8>()
        )
    )
;
VERIFY_PARENT
    Enum<0u8>()
;
YIELD_TO_PARENT
    Tuple()
    Tuple()
;
YIELD_TO_CHILD
    NamedIntent("my_child")
;
YIELD_TO_PARENT;
"##,
        );
        compile_and_decompile_with_inversion_test_subintent_v2(
            "v2_subintent_commands",
            &apply_address_replacements(include_str!("../../examples/subintents/child.rtm")),
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    #[test]
    fn test_decompilation_of_unicode_characters() {
        let canonical_manifest = apply_address_replacements(
            r##"
CALL_METHOD
    Address("${account_address}")
    "hello! CRLF: \r\n, backslash: \\, tab: \t, \b, \f, double quote: \", control character:\u202e, emoji: 🌍"
;
CALL_METHOD
    Address("${account_address}")
    "\t\"goodbye\"\u202c"
;
"##,
        );
        compile_and_decompile_with_inversion_test_v1(
            "unicode_weird_encodings",
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
            &canonical_manifest,
        );
    }

    fn compile_and_decompile_with_inversion_test_v1(
        name: &str,
        manifest: impl AsRef<str>,
        network: &NetworkDefinition,
        blobs: Vec<Vec<u8>>,
        expected_canonical: impl AsRef<str>,
    ) {
        let blob_provider = BlobProvider::new_with_blobs(blobs);
        let manifest = manifest.as_ref();
        let expected_canonical = expected_canonical.as_ref();

        compile_and_decompile_with_inversion_test::<TransactionManifestV1>(
            manifest,
            network,
            blob_provider.clone(),
            expected_canonical,
        );

        println!("{}", expected_canonical.trim());

        let intent = build_intent_v1(
            expected_canonical.as_ref(),
            network,
            blob_provider.blobs().into_values().collect(),
        )
        .expect("Canonical manifest could not be compiled")
        .to_raw()
        .unwrap();

        let intent_hash = PreparedIntentV1::prepare(&intent, PreparationSettings::latest_ref())
            .unwrap()
            .transaction_intent_hash();

        print_blob(name, intent.as_slice());
        print_blob(&format!("{}_HASH", name), intent_hash.0.as_slice());
    }

    fn compile_and_decompile_with_inversion_test_subintent_v2(
        _name: &str,
        manifest: impl AsRef<str>,
        network: &NetworkDefinition,
        blobs: Vec<Vec<u8>>,
        expected_canonical: impl AsRef<str>,
    ) {
        let blob_provider = BlobProvider::new_with_blobs(blobs);
        let manifest = manifest.as_ref();
        let expected_canonical = expected_canonical.as_ref();

        compile_and_decompile_with_inversion_test::<SubintentManifestV2>(
            manifest,
            network,
            blob_provider.clone(),
            expected_canonical,
        );

        println!("{}", expected_canonical.trim());
    }

    fn compile_and_decompile_with_inversion_test<M: BuildableManifest>(
        manifest: &str,
        network: &NetworkDefinition,
        blob_provider: BlobProvider,
        expected_canonical: &str,
    ) {
        let error_style = CompileErrorDiagnosticsStyle::PlainText;
        let original_string = manifest;
        let original_compiled = compile_manifest_with_pretty_error::<M>(
            original_string,
            network,
            blob_provider.clone(),
            error_style,
        )
        .unwrap_or_else(|err| {
            panic!("Manifest string could not be compiled. Error message:\n\n{err}\n")
        });
        let original_binary =
            manifest_encode(&original_compiled).expect("Compiled manifest could not be encoded");

        let original_decompiled =
            decompile(&original_compiled, network).expect("Manifest could not be decompiled");
        let recompiled = compile_manifest_with_pretty_error::<M>(
            &original_decompiled,
            network,
            blob_provider.clone(),
            error_style,
        )
        .expect("Decompiled manifest could not be recompiled");
        let recompiled_binary =
            manifest_encode(&recompiled).expect("Recompiled manifest could not be encoded");

        let recompiled_decompiled =
            decompile(&recompiled, network).expect("Recompiled manifest could not be decompiled");
        let re_recompiled = compile_manifest_with_pretty_error::<M>(
            &recompiled_decompiled,
            network,
            blob_provider.clone(),
            error_style,
        )
        .expect("Decompiled recompiled manifest could not be re-recompiled");
        let re_recompiled_binary =
            manifest_encode(&re_recompiled).expect("Re-recompiled manifest could not be encoded");

        // Check round-trip property
        assert_eq!(original_compiled, recompiled);
        assert_eq!(original_binary, recompiled_binary);
        assert_eq!(recompiled, re_recompiled);
        assert_eq!(recompiled_binary, re_recompiled_binary);

        // Check both canonical decompilations are identical
        assert_eq!(original_decompiled, recompiled_decompiled);

        // Assert that the decompiled matches the expected canonical encoding
        assert_eq!(original_decompiled.trim(), expected_canonical.trim());
    }

    pub fn print_blob(name: &str, blob: &[u8]) {
        std::env::var("PRINT_TEST_VECTORS").ok().map(|_| {
            print!(
                "pub const TX_{}: [u8; {}] = [",
                name.to_uppercase(),
                blob.len()
            );

            for &byte in blob.iter() {
                print!("{:#04x}, ", byte);
            }

            println!("];");
        });
    }

    fn build_intent_v1(
        manifest: &str,
        network: &NetworkDefinition,
        blobs: Vec<Vec<u8>>,
    ) -> Result<IntentV1, CompileError> {
        let blob_provider = BlobProvider::new_with_blobs(blobs);

        let sk_notary = Ed25519PrivateKey::from_u64(3).unwrap();

        let (instructions, blobs) =
            compile_manifest_v1(manifest, &network, blob_provider)?.for_intent();

        Ok(IntentV1 {
            header: TransactionHeaderV1 {
                network_id: network.id,
                start_epoch_inclusive: Epoch::zero(),
                end_epoch_exclusive: Epoch::of(1000),
                nonce: 5,
                notary_public_key: sk_notary.public_key().into(),
                notary_is_signatory: false,
                tip_percentage: 3,
            },
            instructions,
            blobs,
            message: MessageV1::None,
        })
    }

    #[test]
    pub fn decompilation_of_create_non_fungible_resource_with_initial_supply_is_invertible() {
        // Arrange
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                NonFungibleResourceRoles::default(),
                metadata!(),
                Some([(NonFungibleLocalId::integer(1), EmptyStruct {})]),
            )
            .build();

        // Act
        let inverted_manifest = {
            let network = NetworkDefinition::simulator();
            let decompiled = decompile(&manifest, &network).unwrap();
            compile_manifest_v1(&decompiled, &network, BlobProvider::new()).unwrap()
        };

        // Assert
        assert_eq!(manifest, inverted_manifest);
    }

    #[derive(ScryptoSbor, NonFungibleData, ManifestSbor)]
    struct EmptyStruct {}
}

pub fn apply_address_replacements(input: impl ToString) -> String {
    let mut input = input.to_string();
    // Can generate some from resim, eg resim new-account, resim publish examples/hello-world etc
    // For other addresses, uncomment the below:;
    // {
    //     // Generate addresses
    //     use radix_common::address::{AddressBech32Decoder, AddressBech32Encoder};
    //     use radix_common::types::EntityType;
    //     use radix_common::constants::*;

    //     // Random address from resim new-account
    //     let account_address = "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q";

    //     println!("{}", AddressBech32Encoder::for_simulator().encode(CONSENSUS_MANAGER.as_node_id().as_bytes()).unwrap());

    //     let (_, mut pseudo_random_bytes) = AddressBech32Decoder::for_simulator().validate_and_decode(account_address).unwrap();
    //     pseudo_random_bytes[0] = EntityType::InternalFungibleVault as u8;
    //     println!("{}", AddressBech32Encoder::for_simulator().encode(pseudo_random_bytes.as_ref()).unwrap());
    //     pseudo_random_bytes[0] = EntityType::GlobalValidator as u8;
    //     println!("{}", AddressBech32Encoder::for_simulator().encode(pseudo_random_bytes.as_ref()).unwrap());
    //     pseudo_random_bytes[0] = EntityType::GlobalAccessController as u8;
    //     println!("{}", AddressBech32Encoder::for_simulator().encode(pseudo_random_bytes.as_ref()).unwrap());
    //     pseudo_random_bytes[0] = EntityType::GlobalGenericComponent as u8;
    //     println!("{}", AddressBech32Encoder::for_simulator().encode(pseudo_random_bytes.as_ref()).unwrap());
    // };
    let package_package_address = PACKAGE_PACKAGE.to_string(&AddressBech32Encoder::for_simulator());
    let replacement_vectors = sbor::prelude::BTreeMap::from([
        (
            "${xrd_resource_address}",
            "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3",
        ),
        (
            "${fungible_resource_address}",
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        ),
        (
            "${resource_address}",
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        ),
        (
            "${gumball_resource_address}",
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        ),
        (
            "${non_fungible_resource_address}",
            "resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha",
        ),
        (
            "${badge_resource_address}",
            "resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha",
        ),
        (
            "${account_address}",
            "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
        ),
        (
            "${this_account_address}",
            "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
        ),
        (
            "${other_account_address}",
            "account_sim1cyzfj6p254jy6lhr237s7pcp8qqz6c8ahq9mn6nkdjxxxat5syrgz9",
        ),
        (
            "${component_address}",
            "component_sim1cqvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvemygpmu",
        ),
        (
            "${account_a_component_address}",
            "account_sim1c8mulhl5yrk6hh4jsyldps5sdrp08r5v9wusupvzxgqvhlp4c4nwjz",
        ),
        (
            "${account_b_component_address}",
            "account_sim1c8s2hass5g62ckwpv78y8ykdqljtetv4ve6etcz64gveykxznj36tr",
        ),
        (
            "${account_c_component_address}",
            "account_sim1c8ct6jdcwqrg3gzskyxuy0z933fe55fyjz6p56730r95ulzwl3ppva",
        ),
        (
            "${package_address}",
            "package_sim1p4r4955skdjq9swg8s5jguvcjvyj7tsxct87a9z6sw76cdfd2jg3zk",
        ),
        (
            "${minter_badge_resource_address}",
            "resource_sim1ngktvyeenvvqetnqwysevcx5fyvl6hqe36y3rkhdfdn6uzvt5366ha",
        ),
        (
            "${mintable_fungible_resource_address}",
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        ),
        (
            "${mintable_non_fungible_resource_address}",
            "resource_sim1nfhtg7ttszgjwysfglx8jcjtvv8q02fg9s2y6qpnvtw5jsy3wvlhj6",
        ),
        (
            "${vault_address}",
            "internal_vault_sim1tqvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvevp72ff",
        ),
        ("${owner_badge_non_fungible_local_id}", "#1#"),
        (
            "${code_blob_hash}",
            "a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0",
        ),
        ("${initial_supply}", "12"),
        ("${mint_amount}", "12"),
        ("${non_fungible_local_id}", "#12#"),
        (
            "${auth_badge_resource_address}",
            "resource_sim1n24hvnrgmhj6j8dpjuu85vfsagdjafcl5x4ewc9yh436jh2hpu4qdj",
        ),
        ("${auth_badge_non_fungible_local_id}", "#1#"),
        (
            "${package_address}",
            "package_sim1p4r4955skdjq9swg8s5jguvcjvyj7tsxct87a9z6sw76cdfd2jg3zk",
        ),
        (
            "${consensusmanager_address}",
            "consensusmanager_sim1scxxxxxxxxxxcnsmgrxxxxxxxxx000999665565xxxxxxxxxxc06cl",
        ),
        (
            "${validator_address}",
            "validator_sim1svzs2pg9q5zs2pg9q5zs2pg9q5zs2pg9q5zs2pg9q5zs2pg9wr6hj0",
        ),
        (
            "${accesscontroller_address}",
            "accesscontroller_sim1cvvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvexaj7at",
        ),
        (
            "${faucet_component_address}",
            "component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh",
        ),
        (
            "${second_resource_address}",
            "resource_sim1thcgx0f3rwaeetl67cmsssv4p748kd3sjhtge9l4m6ns7cucs97tjv",
        ),
        (
            "${package_package_address}",
            package_package_address.as_str(),
        ),
    ]);
    for (of, with) in replacement_vectors.into_iter() {
        input = input.replace(of, with);
    }
    input
}
