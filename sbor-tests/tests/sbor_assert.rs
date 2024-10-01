#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_camel_case_types)]

use sbor::*;

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(fixed("INLINE:5b210222000121032022010f012307200100220100010709202101022201010c0a4d7954657374456e756d2201012201012307210100022201010c0548656c6c6f220101220001200c0105737461746520220100002201010a0000000000000000"))]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_FixedSchema_Test1 {
    Hello { state: u32 },
}

const TEST_ENUM_SCHEMA: &'static str = "5b210222000121032022010f012307200100220100010709202101022201010c0a4d7954657374456e756d2201012201012307210100022201010c0548656c6c6f220101220001200c0105737461746520220100002201010a0000000000000000";

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(fixed(TEST_ENUM_SCHEMA))]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_FixedSchema_Test2 {
    Hello { state: u32 },
}

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(fixed("CONST:TEST_ENUM_SCHEMA"))]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_FixedSchema_Test3 {
    Hello { state: u32 },
}

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(fixed("FILE:test_enum_v1_schema.txt"))]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_FixedSchema_Test4 {
    Hello { state: u32 },
}

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(fixed("FILE:test_enum_v1_schema.bin"))]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_FixedSchema_Test5 {
    Hello { state: u32 },
}

const TEST_ENUM_V2_SCHEMA: &'static str = "5b210222000121032022010f012307200200220100010709012200202101022201010c0a4d7954657374456e756d2201012201012307210200022201010c0548656c6c6f220101220001200c0105737461746501022201010c05576f726c6422000020220100002201010a0000000000000000";

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(backwards_compatible(
    v1 = "FILE:test_enum_v1_schema.txt",
    v2 = "CONST:TEST_ENUM_V2_SCHEMA",
))]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_WhichHasBeenExtended_Test1 {
    Hello { state: u32 },
    World, // Extension
}

fn params_builder() -> SingleTypeSchemaCompatibilityParameters<NoCustomSchema> {
    SingleTypeSchemaCompatibilityParameters::new()
        .register_version("version1", TEST_ENUM_SCHEMA)
        .register_version("version2", TEST_ENUM_V2_SCHEMA)
}

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(backwards_compatible("EXPR:params_builder()"))]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_WhichHasBeenExtended_Test2 {
    Hello { state: u32 },
    World, // Extension
}

const TEST_ENUM_V2_RENAMED_SCHEMA: &'static str = "5b210222000121032022010f012307200200220100010709012200202101022201010c0a4d7954657374456e756d2201012201012307210200022201010c0548656c6c6f220101220001200c010d73746174655f72656e616d656401022201010c05576f726c6422000020220100002201010a0000000000000000";

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(
    backwards_compatible(
        v1 = "FILE:test_enum_v1_schema.txt",
        v2 = "CONST:TEST_ENUM_V2_RENAMED_SCHEMA",
    ),
    settings(allow_name_changes)
)]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_WhichHasBeenExtendedAndFieldNameChanged_WorksWithAllowNameChanges {
    Hello { state_renamed: u32 },
    World, // Extension
}

const ALLOW_RENAME_SETTINGS: SchemaComparisonSettings =
    SchemaComparisonSettings::allow_extension().allow_all_name_changes();

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(
    backwards_compatible(
        v1 = "FILE:test_enum_v1_schema.txt",
        v2 = "CONST:TEST_ENUM_V2_RENAMED_SCHEMA",
    ),
    settings(ALLOW_RENAME_SETTINGS)
)]
pub enum MyTestEnum_WhichHasBeenExtendedAndFieldNameChangedAgain_WorksWithOverridenSettings {
    Hello { state_renamed_again: u32 },
    World, // Extension
}

#[derive(BasicSborAssertion, BasicSbor)]
#[sbor_assert(
    backwards_compatible(
        v1 = "FILE:test_enum_v1_schema.txt",
        v2 = "CONST:TEST_ENUM_V2_RENAMED_SCHEMA",
    ),
    settings(
        comparison_between_versions = "EXPR: |s| s.allow_all_name_changes()",
        comparison_between_current_and_latest = "EXPR: |s| s",
    )
)]
#[sbor(type_name = "MyTestEnum")]
pub enum MyTestEnum_WhichHasBeenExtendedAndFieldNameChanged_WorksWithOverridenSettings {
    Hello { state_renamed: u32 },
    World, // Extension
}
