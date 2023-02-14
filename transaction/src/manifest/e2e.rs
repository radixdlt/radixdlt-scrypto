#[cfg(test)]
mod tests {
    use crate::manifest::*;
    use radix_engine_interface::node::NetworkDefinition;
    use sbor::rust::collections::*;

    #[test]
    fn test_publish_package() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/package/publish.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/package/code.blob").to_vec(),
                include_bytes!("../../examples/package/abi.blob").to_vec(),
            ],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
PUBLISH_PACKAGE
    Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618")
    Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d")
    Map<String, Tuple>()
    Map<String, String>()
    Tuple(Map<Enum, Enum>(Enum(1u8, Enum(2u8, Enum(1u8))), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Enum(1u8, Enum(2u8, Enum(2u8))), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Enum(1u8, Enum(3u8, Enum(0u8))), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Enum(1u8, Enum(3u8, Enum(1u8))), Enum(0u8, Enum(0u8))), Map<String, Enum>(), Enum(1u8), Map<Enum, Enum>(Enum(1u8, Enum(2u8, Enum(1u8))), Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#"))))), Enum(1u8, Enum(2u8, Enum(2u8))), Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#"))))), Enum(1u8, Enum(3u8, Enum(0u8))), Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#"))))), Enum(1u8, Enum(3u8, Enum(1u8))), Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))))), Map<String, Enum>(), Enum(1u8));
"##,
        );
    }

    #[test]
    fn test_publish_package_with_owner() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/package/publish_with_owner.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/package/code.blob").to_vec(),
                include_bytes!("../../examples/package/abi.blob").to_vec(),
            ],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
PUBLISH_PACKAGE_WITH_OWNER
    Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618")
    Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d")
    NonFungibleGlobalId("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx:#1#");
"##,
        );
    }

    #[test]
    fn test_resource_worktop() {
        compile_and_decompile_with_inversion_test(
            include_str!("../../examples/resources/worktop.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "withdraw_by_amount"
    Decimal("5")
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
TAKE_FROM_WORKTOP_BY_AMOUNT
    Decimal("2")
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket1");
CALL_METHOD
    ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "buy_gumball"
    Bucket("bucket1");
ASSERT_WORKTOP_CONTAINS_BY_AMOUNT
    Decimal("3")
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
ASSERT_WORKTOP_CONTAINS
    ResourceAddress("resource_sim1qzhdk7tq68u8msj38r6v6yqa5myc64ejx3ud20zlh9gseqtux6");
TAKE_FROM_WORKTOP
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket2");
CREATE_PROOF_FROM_BUCKET
    Bucket("bucket2")
    Proof("proof1");
CLONE_PROOF
    Proof("proof1")
    Proof("proof2");
DROP_PROOF
    Proof("proof1");
DROP_PROOF
    Proof("proof2");
CALL_METHOD
    ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "create_proof_by_amount"
    Decimal("5")
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
POP_FROM_AUTH_ZONE
    Proof("proof3");
DROP_PROOF
    Proof("proof3");
RETURN_TO_WORKTOP
    Bucket("bucket2");
TAKE_FROM_WORKTOP_BY_IDS
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket3");
DROP_ALL_PROOFS;
CALL_METHOD
    ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_resource_recall() {
        compile_and_decompile_with_inversion_test(
            include_str!("../../examples/resources/recall.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
RECALL_RESOURCE
    Bytes("49cd9235ba62b2c217e32e5b4754c08219ef16389761356eaccbf6f6bdbfa44d00000000")
    Decimal("1.2");
"##,
        );
    }

    #[test]
    fn test_invocation() {
        compile_and_decompile_with_inversion_test(
            include_str!("../../examples/call/invocation.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_FUNCTION
    PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe")
    "BlueprintName"
    "f"
    "string";
CALL_METHOD
    ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "complicated_method"
    Decimal("1")
    PreciseDecimal("2");
"##,
        );
    }

    #[test]
    fn test_values() {
        compile_and_decompile_with_inversion_test(
            include_str!("../../examples/call/values.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/package/code.blob").to_vec(),
                include_bytes!("../../examples/package/abi.blob").to_vec(),
            ],
            r##"
TAKE_FROM_WORKTOP
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Bucket("bucket1");
CREATE_PROOF_FROM_AUTH_ZONE
    ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag")
    Proof("proof1");
CALL_METHOD
    ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "with_some_basic_types"
    Tuple();
CALL_METHOD
    ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "with_aliases"
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
    ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    "with_all_scrypto_custom_types"
    PackageAddress("package_sim1qyqzcexvnyg60z7lnlwauh66nhzg3m8tch2j8wc0e70qkydk8r")
    ComponentAddress("account_sim1q0u9gxewjxj8nhxuaschth2mgencma2hpkgwz30s9wlslthace")
    ComponentAddress("epochmanager_sim1qsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvygtcq")
    ComponentAddress("clock_sim1qcqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqagpd30")
    ComponentAddress("validator_sim1q5qszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsvkh36j")
    ComponentAddress("accesscontroller_sim1pspqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqq397jz")
    ResourceAddress("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v")
    Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618")
    Bucket("bucket1")
    Proof("proof1")
    Expression("ENTIRE_WORKTOP")
    Hash("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
    EcdsaSecp256k1PublicKey("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
    EcdsaSecp256k1Signature("0079224ea514206706298d8d620f660828f7987068d6d02757e6f3cbbf4a51ab133395db69db1bc9b2726dd99e34efc252d8258dcb003ebaba42be349f50f7765e")
    EddsaEd25519PublicKey("4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29")
    EddsaEd25519Signature("ce993adc51111309a041faa65cbcf1154d21ed0ecdc2d54070bc90b9deb744aa8605b3f686fa178fba21070b4a4678e54eee3486a881e0e328251cd37966de09")
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
            include_str!("../../examples/royalty/royalty.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
SET_PACKAGE_ROYALTY_CONFIG
    PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe")
    Map<String, Tuple>("Blueprint", Tuple(Map<String, U32>("method", 1u32), 0u32));
SET_COMPONENT_ROYALTY_CONFIG
    ComponentAddress("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt")
    Tuple(Map<String, U32>("method", 1u32), 0u32);
CLAIM_PACKAGE_ROYALTY
    PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe");
CLAIM_COMPONENT_ROYALTY
    ComponentAddress("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt");
"##,
        );
    }

    #[test]
    fn test_metadata() {
        compile_and_decompile_with_inversion_test(
            include_str!("../../examples/metadata/metadata.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
SET_METADATA PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe")
    "k"
    "v";
SET_METADATA ComponentAddress("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt")
    "k"
    "v";
SET_METADATA ResourceAddress("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v")
    "k"
    "v";
"##,
        );
    }

    #[test]
    fn test_access_rule() {
        compile_and_decompile_with_inversion_test(
            include_str!("../../examples/access_rule/access_rule.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
SET_METHOD_ACCESS_RULE ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")
    0u32
    Enum(0u8, "test")
    Enum(0u8);
"##,
        );
    }

    #[test]
    fn test_create_fungible_resource_with_initial_supply() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/creation/fungible/with_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_FUNGIBLE_RESOURCE
    18u8
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource", "symbol", "RSRC")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Enum(1u8, Decimal("12"));
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_create_fungible_resource_with_initial_supply_with_owner() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/fungible/with_initial_supply_with_owner.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_FUNGIBLE_RESOURCE_WITH_OWNER
    18u8
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource", "symbol", "RSRC")
    NonFungibleGlobalId("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx:#1#")
    Enum(1u8, Decimal("12"));
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_create_fungible_resource_with_no_initial_supply() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/creation/fungible/no_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_FUNGIBLE_RESOURCE
    18u8
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource", "symbol", "RSRC")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Enum(0u8);
"##,
        );
    }

    #[test]
    fn test_create_fungible_resource_with_no_initial_supply_with_owner() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/fungible/no_initial_supply_with_owner.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_FUNGIBLE_RESOURCE_WITH_OWNER
    18u8
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource", "symbol", "RSRC")
    NonFungibleGlobalId("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx:#1#")
    Enum(0u8);
"##,
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_initial_supply() {
        compile_and_decompile_with_inversion_test(
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
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE
    Enum(1u8)
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Enum(1u8, Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#12#"), Tuple(Tuple("Hello World", Decimal("12")), Tuple(12u8, 19u128))));
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_initial_supply_with_owner() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/non_fungible/with_initial_supply_with_owner.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
             vec![], r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE_WITH_OWNER
    Enum(1u8)
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource")
    NonFungibleGlobalId("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx:#1#")
    Enum(1u8, Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#12#"), Tuple(Tuple("Hello World", Decimal("12")), Tuple(12u8, 19u128))));
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_no_initial_supply() {
        compile_and_decompile_with_inversion_test(
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
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE
    Enum(1u8)
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Enum(0u8);
"##,
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_no_initial_supply_with_owner() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/non_fungible/no_initial_supply_with_owner.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
             vec![], r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE_WITH_OWNER
    Enum(1u8)
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource")
    NonFungibleGlobalId("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx:#1#")
    Enum(0u8);
"##,
        );
    }

    #[test]
    fn test_mint_fungible() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/mint/fungible/mint.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "create_proof_by_amount"
    Decimal("1")
    ResourceAddress("resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02");
MINT_FUNGIBLE
    ResourceAddress("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx")
    Decimal("12");
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_mint_non_fungible() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/mint/non_fungible/mint.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "create_proof_by_amount"
    Decimal("1")
    ResourceAddress("resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02");
MINT_NON_FUNGIBLE
    ResourceAddress("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx")
    Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#12#"), Tuple(Tuple("Hello World", Decimal("12")), Tuple(12u8, 19u128)));
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
"##,
        );
    }

    #[test]
    fn test_assert_access_rule() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/access_rule/assert_access_rule.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CALL_METHOD
    ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na")
    "lock_fee"
    Decimal("10");
ASSERT_ACCESS_RULE
    Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qpflrslzpnprsd27ywcpmm9mqzncshp2sfjg6h59n48smx5k0v:#1#")))));
"##,
        );
    }

    #[test]
    fn test_create_access_controller() {
        compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/access_controller/create.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
            r##"
CREATE_ACCESS_CONTROLLER
    Bucket(1u32)
    Enum(0u8)
    Enum(0u8)
    Enum(0u8)
    Enum(1u8, 1u32);
"##,
        );
    }

    fn compile_and_decompile_with_inversion_test(
        manifest: &str,
        network: &NetworkDefinition,
        blobs: Vec<Vec<u8>>,
        expected_canonical: &str,
    ) {
        let compiled1 = compile(manifest, network, blobs.clone()).unwrap();
        let decompiled1 = decompile(&compiled1.instructions, network).unwrap();

        // Whilst we're here - let's test that compile/decompile are inverses...
        let compiled2 = compile(manifest, network, blobs).unwrap();
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
                "36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618",
            ),
            (
                "${abi_blob_hash}",
                "15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d",
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
