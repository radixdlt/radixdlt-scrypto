use super::*;
use crate::prelude::*;

/// The environment that the tests are run against.
///
/// Each test environment has it's own instance of a [`SelfContainedRadixEngine`] which is exposed
/// through the [`ClientApi`] and [`KernelApi`] and which tests run against.
pub struct TestEnvironment(SelfContainedRadixEngine);
