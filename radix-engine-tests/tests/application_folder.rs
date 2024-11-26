// Integration tests reads all files in /tests/*.rs directory as separate
// crates, but it doesn't read `/tests/x/mod.rs` like we use elsewhere in this
// codebase. But we still want to have separate compilation units for better
// parallelism.
//
// Our workaround is to:
// * Have this X_folder.rs file which gets loaded by the test loader
// * Use a mod definition to point to `X/mod.rs` where tests are defined
mod application;

pub mod prelude {
    pub use radix_engine_tests::prelude::*;
}
