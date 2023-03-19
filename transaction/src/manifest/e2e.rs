#[cfg(test)]
mod tests {
    use crate::eddsa_ed25519::EddsaEd25519PrivateKey;
    use crate::manifest::*;
    use crate::model::{TransactionHeader, TransactionIntent};
    use radix_engine_interface::network::NetworkDefinition;
    use sbor::rust::collections::*;

    #[test]
    fn test_publish_package() {
        compile_and_decompile_with_inversion_test(
            "publish_package",
            &apply_replacements_to_manifest(
                include_str!("../../examples/package/publish.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/package/code.blob").to_vec(),
                include_bytes!("../../examples/package/schema.blob").to_vec(),
            ],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
PUBLISH_PACKAGE
    Blob("a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0")
    Blob("554d6e3a49e90d3be279e7ff394a01d9603cc13aa701c11c1f291f6264aa5791")
    Map<String, Tuple>()
    Map<String, String>()
    Tuple(Map<Tuple, Enum>(Tuple(Enum(2u8), "get"), Enum(0u8, Enum(0u8)), Tuple(Enum(2u8), "set"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Tuple(Enum(6u8), "claim_royalty"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Tuple(Enum(6u8), "set_royalty_config"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#"))))))), Map<String, Enum>(), Enum(0u8, Enum(1u8)), Map<Tuple, Enum>(Tuple(Enum(2u8), "get"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Tuple(Enum(2u8), "set"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Tuple(Enum(6u8), "claim_royalty"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Tuple(Enum(6u8), "set_royalty_config"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#"))))))), Map<String, Enum>(), Enum(0u8, Enum(1u8)));
"##,
        );
    }

    #[test]
    fn test_resource_worktop() {
        compile_and_decompile_with_inversion_test(
            "resource_worktop",
            include_str!("../../examples/resources/worktop.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "withdraw"
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Decimal("5");
TAKE_FROM_WORKTOP_BY_AMOUNT
    Decimal("2")
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket1");
CALL_METHOD
    Address("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "buy_gumball"
    Bucket("bucket1");
ASSERT_WORKTOP_CONTAINS_BY_AMOUNT
    Decimal("3")
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
ASSERT_WORKTOP_CONTAINS
    Address("resource_sim1qzhdk7tq68u8msj38r6v6yqa5myc64ejx3ud20zlh9gseqtux6");
TAKE_FROM_WORKTOP
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket2");
RETURN_TO_WORKTOP
    Bucket("bucket2");
TAKE_FROM_WORKTOP_BY_IDS
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket3");
CALL_METHOD
    Address("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_resource_auth_zone() {
        compile_and_decompile_with_inversion_test(
            "resource_auth_zone",
            include_str!("../../examples/resources/auth_zone.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "withdraw"
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Decimal("5");
TAKE_FROM_WORKTOP
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket1");
CREATE_PROOF_FROM_BUCKET
    Bucket("bucket1")
    Proof("proof1");
CLONE_PROOF
    Proof("proof1")
    Proof("proof2");
DROP_PROOF
    Proof("proof1");
DROP_PROOF
    Proof("proof2");
CALL_METHOD
    Address("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "create_proof_by_amount"
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Decimal("5");
POP_FROM_AUTH_ZONE
    Proof("proof3");
DROP_PROOF
    Proof("proof3");
CALL_METHOD
    Address("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "create_proof_by_amount"
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Decimal("5");
CREATE_PROOF_FROM_AUTH_ZONE
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Proof("proof4");
CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT
    Decimal("1")
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Proof("proof5");
CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS
    Array<NonFungibleLocalId>(NonFungibleLocalId("#123#"))
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Proof("proof6");
CLEAR_AUTH_ZONE;
CLEAR_SIGNATURE_PROOFS;
DROP_ALL_PROOFS;
CALL_METHOD
    Address("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_resource_recall() {
        compile_and_decompile_with_inversion_test(
            "resource_recall",
            include_str!("../../examples/resources/recall.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
RECALL_RESOURCE
    Bytes("62b2c217e32e5b4754c08219ef16389761356eaccbf6f6bdbfa44d00000000")
    Decimal("1.2");
"##,
        );
    }

    #[test]
    fn test_call_function() {
        compile_and_decompile_with_inversion_test(
            "call_function",
            include_str!("../../examples/call/call_function.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_FUNCTION
    Address("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe")
    "BlueprintName"
    "f"
    "string";
"##,
        );
    }

    #[test]
    fn test_call_method() {
        compile_and_decompile_with_inversion_test(
            "call_method",
            include_str!("../../examples/call/call_method.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "complicated_method"
    Decimal("1")
    PreciseDecimal("2");
"##,
        );
    }

    #[test]
    fn test_values() {
        compile_and_decompile_with_inversion_test(
            "values",
            include_str!("../../examples/values/values.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/package/code.blob").to_vec(),
                include_bytes!("../../examples/package/schema.blob").to_vec(),
            ],
            r##"
TAKE_FROM_WORKTOP
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket1");
CREATE_PROOF_FROM_AUTH_ZONE
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Proof("proof1");
CALL_METHOD
    Address("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "aliases"
    Enum(0u8)
    Enum(0u8)
    Enum(1u8, "hello")
    Enum(1u8, "hello")
    Enum(0u8, "test")
    Enum(0u8, "test")
    Enum(1u8, "test123")
    Enum(1u8, "test123")
    Enum(0u8)
    Enum(1u8, "a")
    Enum(0u8, "b")
    Enum(1u8, "c")
    Bytes("deadbeef")
    Bytes("050aff")
    NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:<value>")
    NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:#123#")
    NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:#456#")
    NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:[031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f]")
    NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:#1234567890#")
    NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:#1#")
    Array<Array>(Bytes("dead"), Bytes("050aff"))
    Array<Array>(Bytes("dead"), Bytes("050aff"))
    Array<Tuple>(NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:<value>"), NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:#1#"))
    Array<Tuple>(NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:<value>"), NonFungibleGlobalId("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag:#1#"));
CALL_METHOD
    Address("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "custom_types"
    Address("package_sim1qyqzcexvnyg60z7lnlwauh66nhzg3m8tch2j8wc0e70qkydk8r")
    Address("package_sim1qyqzcexvnyg60z7lnlwauh66nhzg3m8tch2j8wc0e70qkydk8r")
    Address("account_sim1q0u9gxewjxj8nhxuaschth2mgencma2hpkgwz30s9wlslthace")
    Address("epochmanager_sim1qsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvygtcq")
    Address("clock_sim1qcqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqagpd30")
    Address("validator_sim1q5qszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsvkh36j")
    Address("accesscontroller_sim1pspqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqq397jz")
    Address("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v")
    Bucket("bucket1")
    Proof("proof1")
    Expression("ENTIRE_WORKTOP")
    Blob("a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0")
    Decimal("1.2")
    PreciseDecimal("1.2")
    NonFungibleLocalId("<SomeId>")
    NonFungibleLocalId("#12#")
    NonFungibleLocalId("[031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f]")
    NonFungibleLocalId("{43968a72-5954-45da-9678-8659dd399faa}");
"##,
        );
    }

    #[test]
    fn test_royalty() {
        compile_and_decompile_with_inversion_test(
            "royalty",
            include_str!("../../examples/royalty/royalty.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
SET_PACKAGE_ROYALTY_CONFIG
    Address("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe")
    Map<String, Tuple>("Blueprint", Tuple(Map<String, U32>("method", 1u32), 0u32));
SET_COMPONENT_ROYALTY_CONFIG
    Address("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt")
    Tuple(Map<String, U32>("method", 1u32), 0u32);
CLAIM_PACKAGE_ROYALTY
    Address("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe");
CLAIM_COMPONENT_ROYALTY
    Address("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt");
"##,
        );
    }

    #[test]
    fn test_metadata() {
        compile_and_decompile_with_inversion_test(
            "metadata",
            include_str!("../../examples/metadata/metadata.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
SET_METADATA
    Address("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe")
    "k"
    Enum(0u8, Enum(0u8, "v"));
SET_METADATA
    Address("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt")
    "k"
    Enum(0u8, Enum(0u8, "v"));
SET_METADATA
    Address("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v")
    "k"
    Enum(0u8, Enum(0u8, "v"));
REMOVE_METADATA
    Address("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe")
    "k";
REMOVE_METADATA
    Address("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt")
    "k";
REMOVE_METADATA
    Address("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v")
    "k";
"##,
        );
    }

    #[test]
    fn test_access_rule() {
        compile_and_decompile_with_inversion_test(
            "access_rule",
            include_str!("../../examples/access_rule/access_rule.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
SET_METHOD_ACCESS_RULE
    Address("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    Tuple(Enum(0u8), "test")
    Enum(0u8);
"##,
        );
    }

    #[test]
    fn test_create_fungible_resource_with_initial_supply() {
        compile_and_decompile_with_inversion_test(
            "create_fungible_resource_with_initial_supply",
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/creation/fungible/with_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
    18u8
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource", "symbol", "RSRC")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Decimal("12");
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_create_fungible_resource_with_no_initial_supply() {
        compile_and_decompile_with_inversion_test(
            "create_fungible_resource_with_no_initial_supply",
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/creation/fungible/no_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_FUNGIBLE_RESOURCE
    18u8
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource", "symbol", "RSRC")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)));
"##,
        );
    }

    //FIXME: this test does not work because of decompiler error:
    // See https://rdxworks.slack.com/archives/C01HK4QFXNY/p1678185923283569?thread_ts=1678184674.780149&cid=C01HK4QFXNY
    #[ignore]
    #[test]
    fn test_create_non_fungible_resource_with_initial_supply() {
        compile_and_decompile_with_inversion_test(
            "create_non_fungible_resource_with_initial_supply",
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/non_fungible/with_initial_supply.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
    Enum(1u8)
    Tuple(Tuple(Array<Enum>(), Array<Tuple>(), Array<Enum>()), Enum(0u8, 64u8))
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Map<NonFungibleLocalId, Array>(NonFungibleLocalId("#12#"), Bytes("5c21020c0b48656c6c6f20576f726c64a00000b0d86b9088a6000000000000000000000000000000000000000000000000"));
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_no_initial_supply() {
        compile_and_decompile_with_inversion_test(
            "create_non_fungible_resource_with_no_initial_supply",
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/non_fungible/no_initial_supply.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE
    Enum(1u8)
    Tuple(Tuple(Array<Enum>(), Array<Tuple>(), Array<Enum>()), Enum(0u8, 64u8), Array<String>())
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)));
"##,
        );
    }

    #[test]
    fn test_mint_fungible() {
        compile_and_decompile_with_inversion_test(
            "mint_fungible",
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/mint/fungible/mint.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "create_proof_by_amount"
    Address("resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02")
    Decimal("1");
MINT_FUNGIBLE
    Address("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx")
    Decimal("12");
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_mint_non_fungible() {
        compile_and_decompile_with_inversion_test(
            "mint_non_fungible",
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/mint/non_fungible/mint.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "create_proof_by_amount"
    Address("resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02")
    Decimal("1");
MINT_NON_FUNGIBLE
    Address("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx")
    Tuple(Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#12#"), Tuple(Tuple())));
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_assert_access_rule() {
        compile_and_decompile_with_inversion_test(
            "assert_access_rule",
            &apply_replacements_to_manifest(
                include_str!("../../examples/access_rule/assert_access_rule.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    Address("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
ASSERT_ACCESS_RULE
    Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))));
"##,
        );
    }

    #[test]
    fn test_create_account() {
        compile_and_decompile_with_inversion_test(
            "create_account",
            &apply_replacements_to_manifest(
                include_str!("../../examples/account/new.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CREATE_ACCOUNT_ADVANCED
    Enum(0u8)
    Enum(0u8);
CREATE_ACCOUNT;
"##,
        );
    }

    #[test]
    fn test_create_identity() {
        compile_and_decompile_with_inversion_test(
            "create_identity",
            &apply_replacements_to_manifest(
                include_str!("../../examples/identity/new.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CREATE_IDENTITY_ADVANCED
    Enum(0u8)
    Enum(0u8);
CREATE_IDENTITY;
"##,
        );
    }

    #[test]
    fn test_create_access_controller() {
        compile_and_decompile_with_inversion_test(
            "create_access_controller",
            &apply_replacements_to_manifest(
                include_str!("../../examples/access_controller/new.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
TAKE_FROM_WORKTOP
    Address("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket1");
CREATE_ACCESS_CONTROLLER
    Bucket("bucket1")
    Tuple(Enum(0u8), Enum(0u8), Enum(0u8))
    Enum(0u8);
"##,
        );
    }

    fn compile_and_decompile_with_inversion_test(
        name: &str,
        manifest: &str,
        network: &NetworkDefinition,
        blobs: Vec<Vec<u8>>,
        expected_canonical: &str,
    ) {
        let compiled1 = compile(manifest, network, blobs.clone()).unwrap();
        let decompiled1 = decompile(&compiled1.instructions, network).unwrap();

        // Whilst we're here - let's test that compile/decompile are inverses...
        let compiled2 = compile(manifest, network, blobs.clone()).unwrap();
        let decompiled2 = decompile(&compiled2.instructions, network).unwrap();

        // The manifest argument is not necessarily in canonical decompiled string representation,
        // therefore we can't assert that decompiled1 == manifest ...
        // So instead we assert that decompiled1 and decompiled2 match :)
        assert_eq!(
            compiled1, compiled2,
            "Compile(Decompile(compiled_manifest)) != compiled_manifest"
        );
        assert_eq!(
            decompiled1, decompiled2,
            "Decompile(Compile(canonical_manifest_str)) != canonical_manifest_str"
        );

        // If you use the following output for test cases, make sure you've checked the diff
        println!("{}", decompiled2);

        assert_eq!(decompiled2.trim(), expected_canonical.trim()); // trim for better view

        let intent = build_intent(&expected_canonical, blobs).to_bytes().unwrap();
        print_blob(name, intent);
    }

    fn print_blob(name: &str, blob: Vec<u8>) {
        print!(
            "const TX_{}: [u8; {}] = [",
            name.clone().to_uppercase(),
            blob.len()
        );

        for &byte in blob.iter() {
            print!("{:#04x}, ", byte);
        }

        println!("];");
    }

    fn build_intent(manifest: &str, blobs: Vec<Vec<u8>>) -> TransactionIntent {
        let sk_notary = EddsaEd25519PrivateKey::from_u64(3).unwrap();

        TransactionIntent::new(
            &NetworkDefinition::simulator(),
            TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 1000,
                nonce: 5,
                notary_public_key: sk_notary.public_key().into(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 3,
            },
            manifest,
            blobs,
        )
        .unwrap()
    }

    fn apply_replacements_to_manifest(mut manifest: String) -> String {
        let replacement_vectors = BTreeMap::from([
            (
                "${xrd_resource_address}",
                "resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr",
            ),
            (
                "${account_component_address}",
                "account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na",
            ),
            (
                "${other_account_component_address}",
                "account_sim1qdy4jqfpehf8nv4n7680cw0vhxqvhgh5lf3ae8jkjz6q5hmzed",
            ),
            (
                "${minter_badge_resource_address}",
                "resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02",
            ),
            (
                "${mintable_resource_address}",
                "resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx",
            ),
            (
                "${owner_badge_resource_address}",
                "resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx",
            ),
            ("${owner_badge_non_fungible_local_id}", "#1#"),
            (
                "${code_blob_hash}",
                "a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0",
            ),
            (
                "${schema_blob_hash}",
                "554d6e3a49e90d3be279e7ff394a01d9603cc13aa701c11c1f291f6264aa5791",
            ),
            ("${initial_supply}", "12"),
            ("${mint_amount}", "12"),
            ("${non_fungible_local_id}", "#12#"),
            (
                "${auth_badge_resource_address}",
                "resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v",
            ),
            ("${auth_badge_non_fungible_local_id}", "#1#"),
        ]);
        for (of, with) in replacement_vectors.into_iter() {
            manifest = manifest.replace(of, with);
        }
        manifest
    }
}
