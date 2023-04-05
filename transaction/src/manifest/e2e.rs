#[cfg(test)]
mod tests {
    use crate::builder::ManifestBuilder;
    use crate::eddsa_ed25519::EddsaEd25519PrivateKey;
    use crate::manifest::*;
    use crate::model::{TransactionHeader, TransactionIntent};
    use radix_engine_common::data::scrypto::model::{NonFungibleIdType, NonFungibleLocalId};
    use radix_engine_common::{ManifestSbor, ScryptoSbor};
    use radix_engine_interface::blueprints::resource::AccessRule;
    use radix_engine_interface::network::NetworkDefinition;
    use sbor::rust::collections::*;
    use scrypto_derive::NonFungibleData;

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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "lock_fee"
    Decimal("10");
PUBLISH_PACKAGE_ADVANCED
    Blob("a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0")
    Blob("554d6e3a49e90d3be279e7ff394a01d9603cc13aa701c11c1f291f6264aa5791")
    Map<String, Tuple>()
    Map<String, String>()
    Tuple(Map<Tuple, Enum>(Tuple(Enum(1u8), "claim_royalty"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6:#1#")))))), Tuple(Enum(1u8), "set_royalty_config"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6:#1#")))))), Tuple(Enum(2u8), "get"), Enum(0u8, Enum(0u8)), Tuple(Enum(2u8), "set"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6:#1#"))))))), Map<String, Enum>(), Enum(0u8, Enum(1u8)), Map<Tuple, Enum>(Tuple(Enum(1u8), "claim_royalty"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6:#1#")))))), Tuple(Enum(1u8), "set_royalty_config"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6:#1#")))))), Tuple(Enum(2u8), "get"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6:#1#")))))), Tuple(Enum(2u8), "set"), Enum(0u8, Enum(2u8, Enum(0u8, Enum(0u8, Enum(0u8, NonFungibleGlobalId("resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6:#1#"))))))), Map<String, Enum>(), Enum(0u8, Enum(1u8)));
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "withdraw"
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Decimal("5");
TAKE_FROM_WORKTOP_BY_AMOUNT
    Decimal("2")
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Bucket("bucket1");
CALL_METHOD
    Address("component_sim1p8xzs5t032p03afg4p6kzyfuxgllj8uumk7st7dn869qs6vdzq")
    "buy_gumball"
    Bucket("bucket1");
ASSERT_WORKTOP_CONTAINS_BY_AMOUNT
    Decimal("3")
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k");
ASSERT_WORKTOP_CONTAINS
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe");
TAKE_FROM_WORKTOP
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Bucket("bucket2");
RETURN_TO_WORKTOP
    Bucket("bucket2");
TAKE_FROM_WORKTOP_BY_IDS
    Array<NonFungibleLocalId>(NonFungibleLocalId("#1#"))
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Bucket("bucket3");
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "withdraw"
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Decimal("5");
TAKE_FROM_WORKTOP
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "create_proof_by_amount"
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Decimal("5");
POP_FROM_AUTH_ZONE
    Proof("proof3");
DROP_PROOF
    Proof("proof3");
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "create_proof_by_amount"
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Decimal("5");
CREATE_PROOF_FROM_AUTH_ZONE
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Proof("proof4");
CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT
    Decimal("1")
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Proof("proof5");
CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS
    Array<NonFungibleLocalId>(NonFungibleLocalId("#123#"))
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Proof("proof6");
CLEAR_AUTH_ZONE;
CLEAR_SIGNATURE_PROOFS;
DROP_ALL_PROOFS;
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("internal_vault_sim1pcqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsdm5dfg")
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
    Address("package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Bucket("bucket1");
CREATE_PROOF_FROM_AUTH_ZONE
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
    Proof("proof1");
CALL_METHOD
    Address("component_sim1p8xzs5t032p03afg4p6kzyfuxgllj8uumk7st7dn869qs6vdzq")
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
    NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:<value>")
    NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:#123#")
    NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:#456#")
    NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:[031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f]")
    NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:#1234567890#")
    NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:#1#")
    Array<Array>(Bytes("dead"), Bytes("050aff"))
    Array<Array>(Bytes("dead"), Bytes("050aff"))
    Array<Tuple>(NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:<value>"), NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:#1#"))
    Array<Tuple>(NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:<value>"), NonFungibleGlobalId("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:#1#"));
CALL_METHOD
    Address("component_sim1p8xzs5t032p03afg4p6kzyfuxgllj8uumk7st7dn869qs6vdzq")
    "custom_types"
    Address("package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv")
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    Address("epochmanager_sim1qvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqf9zrm9")
    Address("clock_sim1q5qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzxkxrs")
    Address("validator_sim1qsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpskldua9")
    Address("accesscontroller_sim1qcqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpsguupgz")
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
    Address("package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv")
    Map<String, Tuple>("Blueprint", Tuple(Map<String, U32>("method", 1u32), 0u32));
SET_COMPONENT_ROYALTY_CONFIG
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    Tuple(Map<String, U32>("method", 1u32), 0u32);
CLAIM_PACKAGE_ROYALTY
    Address("package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv");
CLAIM_COMPONENT_ROYALTY
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v");
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
    Address("package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv")
    "field_name"
    Enum(0u8, Enum(0u8, "v"));
SET_METADATA
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "field_name"
    Enum(0u8, Enum(0u8, "v"));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(0u8, "v"));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(1u8, true));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(2u8, 123u8));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(3u8, 123u32));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(4u8, 123u64));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(5u8, -123i32));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(6u8, -123i64));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(7u8, Decimal("10.5")));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(8u8, Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(9u8, Enum(0u8, Bytes("0000000000000000000000000000000000000000000000000000000000000000ff"))));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(10u8, NonFungibleGlobalId("resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje:<some_string>")));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(11u8, NonFungibleLocalId("<some_string>")));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(12u8, Tuple(10000i64)));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(0u8, Enum(13u8, "https://radixdlt.com"));
SET_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name"
    Enum(1u8, Array<Enum>(Enum(0u8, "some_string"), Enum(0u8, "another_string"), Enum(0u8, "yet_another_string")));
REMOVE_METADATA
    Address("package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv")
    "field_name";
REMOVE_METADATA
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "field_name";
REMOVE_METADATA
    Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
    "field_name";
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
    Address("resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje")
    Tuple(Enum(1u8), "test")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "lock_fee"
    Decimal("10");
CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
    18u8
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource", "symbol", "RSRC")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Decimal("12");
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "lock_fee"
    Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
    Enum(1u8)
    Tuple(Tuple(Array<Enum>(), Array<Tuple>(), Array<Enum>()), Enum(0u8, 64u8))
    Map<String, String>("description", "A very innovative and important resource", "name", "MyResource")
    Map<Enum, Tuple>(Enum(4u8), Tuple(Enum(0u8), Enum(1u8)), Enum(5u8), Tuple(Enum(0u8), Enum(1u8)))
    Map<NonFungibleLocalId, Array>(NonFungibleLocalId("#12#"), Bytes("5c21020c0b48656c6c6f20576f726c64a00000b0d86b9088a6000000000000000000000000000000000000000000000000"));
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "lock_fee"
    Decimal("10");
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "create_proof_by_amount"
    Address("resource_sim1q9g995jh0x0eaf3672kac6ruq9rr2jvwy4d82qw3cd3qlhgqer")
    Decimal("1");
MINT_FUNGIBLE
    Address("resource_sim1qtvh6xzsalqrfn57w7tsn6n5jhs6h7tvmzc5a6ysypsqjcpftz")
    Decimal("12");
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
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
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "lock_fee"
    Decimal("10");
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "create_proof_by_amount"
    Address("resource_sim1q9g995jh0x0eaf3672kac6ruq9rr2jvwy4d82qw3cd3qlhgqer")
    Decimal("1");
MINT_NON_FUNGIBLE
    Address("resource_sim1qtvh6xzsalqrfn57w7tsn6n5jhs6h7tvmzc5a6ysypsqjcpftz")
    Tuple(Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#12#"), Tuple(Tuple())));
CALL_METHOD
    Address("account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
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
    Tuple(Map<Tuple, Enum>(), Map<Tuple, Enum>(), Map<String, Enum>(), Enum(0u8, Enum(1u8)), Map<Tuple, Enum>(), Map<String, Enum>(), Enum(0u8, Enum(1u8)));
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
    Tuple(Map<Tuple, Enum>(), Map<Tuple, Enum>(), Map<String, Enum>(), Enum(0u8, Enum(1u8)), Map<Tuple, Enum>(), Map<String, Enum>(), Enum(0u8, Enum(1u8)));
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
    Address("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k")
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
                "resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje",
            ),
            (
                "${account_address}",
                "account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v",
            ),
            (
                "${other_account_address}",
                "account_sim1quxmes4pxzvw8mnz5zgsjmv0atudekp9gr2tmf7evlqsp82e9v",
            ),
            (
                "${minter_badge_resource_address}",
                "resource_sim1q9g995jh0x0eaf3672kac6ruq9rr2jvwy4d82qw3cd3qlhgqer",
            ),
            (
                "${mintable_resource_address}",
                "resource_sim1qtvh6xzsalqrfn57w7tsn6n5jhs6h7tvmzc5a6ysypsqjcpftz",
            ),
            (
                "${owner_badge_resource_address}",
                "resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe",
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
                "resource_sim1qgjfp996zpttrx4mcs2zlh5u6rym3q7f596qj9capczqlr3jk6",
            ),
            ("${auth_badge_non_fungible_local_id}", "#1#"),
        ]);
        for (of, with) in replacement_vectors.into_iter() {
            manifest = manifest.replace(of, with);
        }
        manifest
    }

    #[test]
    pub fn decompilation_of_create_non_fungible_resource_with_initial_supply_is_invertible() {
        // Arrange
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                NonFungibleIdType::Integer,
                BTreeMap::new(),
                BTreeMap::<_, (_, AccessRule)>::new(),
                Some([(NonFungibleLocalId::integer(1), EmptyStruct {})]),
            )
            .build();

        // Act
        let inverted_manifest = {
            let network = NetworkDefinition::simulator();
            let decompiled = decompile(&manifest.instructions, &network).unwrap();
            compile(&decompiled, &network, vec![]).unwrap()
        };

        // Assert
        assert_eq!(manifest, inverted_manifest);
    }

    #[derive(ScryptoSbor, NonFungibleData, ManifestSbor)]
    struct EmptyStruct {}
}
