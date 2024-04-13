//! The tests in this module benchmark how long it takes for Rust Analyzer to perform autocomplete
//! on a number of common scenarios that developers might find themselves in when using Scrypto.
//!
//! While it would be more accurate for these benchmarks to use criterion and be turned into proper
//! benchmarks with multiple samples, each run of those tests does a lot of things that are not
//! directly related to autocomplete such as running `cargo check`, build scripts, expanding proc
//! macros, etc. This means that each run of this test takes too long and having multiple samples
//! would mean that they would take even longer. For the time being, we're satisfied with the
//! accuracy of a single sample.

use cargo_toml::Manifest;
use rust_analyzer_tests::*;

#[test]
fn end_to_end_test_autocomplete_benchmark() -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use scrypto::prelude::*;
        pub fn some_function() {
            ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
        }
        "#,
        Some(remove_all_modifier),
        Some(remove_dev_dependencies_modifier),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

// NOTE: Commented to test faster CI.
// #[test]
// fn benchmark_resource_builder_method_inside_blueprint_with_dev_dependencies(
// ) -> Result<(), TimingError> {
//     let duration = time_autocompletion(
//         r#"
//         use scrypto::prelude::*;

//         #[blueprint]
//         mod blueprint {
//             pub struct Hello;

//             impl Hello {
//                 pub fn new() {
//                     ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}new_fungible(OwnerRole::None);
//                 }
//             }
//         }
//         "#,
//         Some(remove_all_modifier),
//         Some(no_changes_modifier),
//         LoggingHandling::LogToStdOut,
//     )?;
//     println!("Autocomplete took: {}ms", duration.as_millis());
//     Ok(())
// }

// #[test]
// fn benchmark_resource_builder_method_inside_blueprint_without_dev_dependencies(
// ) -> Result<(), TimingError> {
//     let duration = time_autocompletion(
//         r#"
//         use scrypto::prelude::*;

//         #[blueprint]
//         mod blueprint {
//             pub struct Hello;

//             impl Hello {
//                 pub fn new() {
//                     ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}new_fungible(OwnerRole::None);
//                 }
//             }
//         }
//         "#,
//         Some(remove_all_modifier),
//         Some(remove_dev_dependencies_modifier),
//         LoggingHandling::LogToStdOut,
//     )?;
//     println!("Autocomplete took: {}ms", duration.as_millis());
//     Ok(())
// }

// #[test]
// fn benchmark_component_instantiation_method_inside_blueprint_with_dev_dependencies(
// ) -> Result<(), TimingError> {
//     let duration = time_autocompletion(
//         r#"
//         use scrypto::prelude::*;

//         #[blueprint]
//         mod blueprint {
//             pub struct Hello;

//             impl Hello {
//                 pub fn new() -> Global<Hello> {
//                     Self
//                         .instantiate()
//                         .prepare_to_globalize(OwnerRole::None)
//                         .{{%EXPECT_ANY_OF:roles%}}globalize()
//                 }
//             }
//         }
//         "#,
//         Some(remove_all_modifier),
//         Some(no_changes_modifier),
//         LoggingHandling::LogToStdOut,
//     )?;
//     println!("Autocomplete took: {}ms", duration.as_millis());
//     Ok(())
// }

// #[test]
// fn benchmark_component_instantiation_method_inside_blueprint_without_dev_dependencies(
// ) -> Result<(), TimingError> {
//     let duration = time_autocompletion(
//         r#"
//         use scrypto::prelude::*;

//         #[blueprint]
//         mod blueprint {
//             pub struct Hello;

//             impl Hello {
//                 pub fn new() -> Global<Hello> {
//                     Self
//                         .instantiate()
//                         .prepare_to_globalize(OwnerRole::None)
//                         .{{%EXPECT_ANY_OF:roles%}}globalize()
//                 }
//             }
//         }
//         "#,
//         Some(remove_all_modifier),
//         Some(remove_dev_dependencies_modifier),
//         LoggingHandling::LogToStdOut,
//     )?;
//     println!("Autocomplete took: {}ms", duration.as_millis());
//     Ok(())
// }

fn remove_all_modifier(_: &str) -> String {
    "".to_owned()
}

fn no_changes_modifier(s: &str) -> String {
    s.to_owned()
}

fn remove_dev_dependencies_modifier(s: &str) -> String {
    let mut manifest = Manifest::from_str(s).unwrap();
    manifest.dev_dependencies.clear();
    toml::to_string(&manifest).unwrap()
}
