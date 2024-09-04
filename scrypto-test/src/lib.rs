//! This crate is an implementation of for Scrypto-Test, a unit testing framework for Scrypto that
//! follows an invocation-based approach instead of a transaction-based approach to testing,
//! allowing Scrypto developers to write tests that look and feel like Scrypto. Scrypto-Test is not
//! a replacement for transaction-based tests offered by the LedgerSimulator, it would just be an
//! addition and another way for Scrypto developers to test their code where Scrypto-Test could be
//! classified as a unit-testing framework while the LedgerSimulator could be classified as an
//! integration-testing framework.
//!
//! # Why
//!
//! We already have a way to test Scrypto blueprints in the from of the scrypto_test::prelude::LedgerSimulator
//! which is essentially an in-memory ledger that we can run transactions against, get back
//! transaction receipts, and determine-based on the TransactionReceipt-if the behavior of the
//! blueprint or component is as we expect or not. This approach is tried and tested and has been
//! proven to work as evident by the hundreds and thousands of tests in the radix_engine_tests crate
//! that make it clear that the transaction-based approach works. However, it has a number of
//! issues, especially when we think about the target audience of our testing framework: DeFi
//! developers.
//!
//! In the current (transaction-based) model, there is a lot of boilerplate code involved to write
//! what should be a straightforward test. As an example, to test that a contribution of X and Y
//! resources to Radiswap results in Z pool units minted the test author needs to:
//!
//! * Create these two resources to use for testing.
//! * Decide on whether these resources should just be mintable on demand or if the supply of these
//!   resources should be stored in some account that will be used in the test to withdraw from.
//! * If an account will hold those resources, then the author needs to create that account.
//! * Split out the instructions into multiple manifests as needed such as in cases where one
//!   instruction depends on the output of a previous instruction.
//! * Ensure that the execution of the previous transactions did indeed succeed and extract out the
//!   information required from receipts either through the worktop changes, balance changes, or by
//!   other means.
//! * Manage and ensure that the worktop by the end of the transaction is empty of all resources and
//!   that the accounts that the resources will be deposited into sign the transaction.
//!
//! Most if not all of the items listed above are not core to what the developer wishes to test,
//! recall that they wished to test whether a contribution of X and Y resources returns Z pool
//! units. However, they spent a majority of their time thinking about completely different
//! problems. Thus, there is not only a large amount of boilerplate code, but there is also large
//! mental overhead for a test should be conceptually easy and simple to write. As you can see from
//! the description above, most of the time that is spent writing tests is **not spent writing
//! tests**, but spend initializing and creating the environment and managing side effects just to
//! then write a simple test in the form of a method call to some node id.
//!
//! # Scrypto-Test Model
//!
//! This model differs from the transaction based model of writing tests in that we do not have
//! transaction instructions, processor, and worktop at all. In fact, **nothing** related to
//! transactions exists in this model. Instead, there exists a [`TestEnvironment`] struct that each
//! test can instantiate instances of. Each [`TestEnvironment`] instance has a substate store,
//! track, and kernel. On top of that, [`TestEnvironment`] implements the [`SystemApi`] trait. The
//! [`TestEnvironment`] can be looked at as self-contained Radix Engine that’s exposed through the
//! [`SystemApi`] and that has some other high-level helper methods as it contains all of the layers
//! of the engine. This means that:
//!
//! * A [`TestEnvironment`] instance is a self-contained instance of the Radix Engine and Kernel
//!   which are exposed through the [`SystemApi`].
//! * Since [`TestEnvironment`] implements the [`SystemApi`] it can be used as a substitute to
//!   ScryptoEnv from Scrypto and the SystemService from native. This means that the simple
//!   interface seen in the radix_native_sdk crate can be used within tests.
//! * If each test has it’s own [`TestEnvironment`] instance (they instantiate that themselves if
//!   they need it), then tests have no shared dependencies and are isolated.
//! * The biggest struggle with the transaction-based model was around dealing with transient nodes.
//!   More specifically, if we wanted to make sure that bucket X returned from some invocation
//!   contained Y resources, how would we do that? Unfortunately, there was no easy way to do it. In
//!   this model, if we make an invocation and get a Bucket back, there is no worktop for the bucket
//!   to go into, we have a proper Bucket object that we can call amount() on and assert against.
//!   Thus, this approach makes it easier to have assertions around transient nodes.
//!
//! Once the [`TestEnvironment`] has been instantiated we would get a Kernel with two Call Frames:
//! 1. **The Root Call Frame:** We have a root Call Frame to be consistent with how other parts of
//!    the stack use the kernel where there is always a root Call Frame. After the instantiation is
//!    complete, the root callframe is pushed onto the stack of previous Call Frames the kernel has.
//! 2. **The Test Call Frame:** This is the Call Frame that is used for all of the invocation that
//!    will be made throughout the test. This Call Frame functions exactly like any other Call
//!    Frame, it can own nodes, get messages from other Call Frames, and so on. As an example, say
//!    we invoke a method on some node that returns a Bucket, this Bucket is now owned and visible
//!    to this Call Frame. We are now able to call methods such as resource_address() and amount()
//!    on this Bucket since it’s a node we own and the [`TestEnvironment`] has a Heap.
//!
//! [`SystemApi`]: crate::prelude::SystemApi
//! [`TestEnvironment`]: crate::prelude::TestEnvironment

pub mod environment;
pub mod ledger_simulator;
pub mod prelude;
pub mod sdk;
pub mod utils;

#[macro_export]
macro_rules! this_package {
    () => {
        env!("CARGO_MANIFEST_DIR")
    };
}

/// Includes the WASM file of a Scrypto package.
///
/// Notes:
/// * This macro will NOT compile the package;
/// * The binary name is normally the package name with `-` replaced with `_`.
///
/// # Example
/// ```ignore
/// # // Ignoring because of include_code!
/// use scrypto::prelude::*;
///
/// // This package
/// let wasm1 = include_code!("bin_name");
///
/// // Another package
/// let wasm2 = include_code!("/path/to/package", "bin_name");
/// ```
#[macro_export]
macro_rules! include_code {
    ($bin_name: expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".wasm"
        ))
    };
    ($package_dir: expr, $bin_name: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".wasm"
        ))
    };
}

/// Includes the schema file of a Scrypto package.
///
/// Notes:
/// * This macro will NOT compile the package;
/// * The binary name is normally the package name with `-` replaced with `_`.
///
/// # Example
/// ```ignore
/// # // Including because of include_schema!(..)
/// use scrypto::prelude::*;
///
/// // This package
/// let schema1 = include_schema!("bin_name");
///
/// // Another package
/// let schema2 = include_schema!("/path/to/package", "bin_name");
/// ```
#[macro_export]
macro_rules! include_schema {
    ($bin_name: expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".rpd"
        ))
    };
    ($package_dir: expr, $bin_name: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".rpd"
        ))
    };
}
