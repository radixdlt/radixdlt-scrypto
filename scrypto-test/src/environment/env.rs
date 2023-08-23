use super::*;

/// The environment that the tests are run against.
///
/// Each test environment has it's own instance of a [`SelfContainedRadixEngine`] which is exposed
/// through the [`ClientApi`] and which tests run against.
///
/// [`ClientApi`]: crate::prelude::ClientApi
pub struct TestEnvironment(pub(super) SelfContainedRadixEngine);
