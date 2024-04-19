use radix_common::prelude::*;
use radix_rust::rust::prelude::*;

pub const TEST_UTILS_BLUEPRINT: &str = "TestUtils";

pub const TEST_UTILS_PANIC_IDENT: &str = "panic";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct TestUtilsPanicInput(pub String);
pub type TestUtilsPanicOutput = ();
