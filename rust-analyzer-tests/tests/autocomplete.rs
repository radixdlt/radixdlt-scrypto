//! The tests in this module benchmark how long it takes for Rust Analyzer to perform autocomplete
//! on a number of common scenarios that developers might find themselves in when using Scrypto.
//!
//! While it would be more accurate for these benchmarks to use criterion and be turned into proper
//! benchmarks with multiple samples, each run of those tests does a lot of things that are not
//! directly related to autocomplete such as running `cargo check`, build scripts, expanding proc
//! macros, etc. This means that each run of this test takes too long and having multiple samples
//! would mean that they would take even longer. For the time being, we're satisfied with the
//! accuracy of a single sample.
//!
//! Tests in this module can be run on their own, but they are typically run from the
//! `benchmark_autocomplete.py` script at the root of the repo which runs each of the tests
//! sequentially and generates a report from them.

use cargo_toml::Manifest;
use indoc::indoc;
use rust_analyzer_tests::*;

/*
#[test]
fn benchmark_baseline() -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use std::collections::{{%EXPECT_ANY_OF:HashMap,BTreeMap,HashSet,BTreeSet%}}HashMap;
        "#,
        Some(remove_all_modifier),
        Some(no_changes_modifier),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

#[test]
fn benchmark_resource_builder_method_outside_blueprint_with_dev_dependencies(
) -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use scrypto::prelude::*;
        pub fn some_function() {
            ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
        }
        "#,
        Some(remove_all_modifier),
        Some(no_changes_modifier),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

#[test]
fn benchmark_resource_builder_method_outside_blueprint_without_dev_dependencies(
) -> Result<(), TimingError> {
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

#[test]
fn benchmark_resource_builder_method_inside_blueprint_with_dev_dependencies(
) -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use scrypto::prelude::*;

        #[blueprint]
        mod blueprint {
            pub struct Hello;

            impl Hello {
                pub fn new() {
                    ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}new_fungible(OwnerRole::None);
                }
            }
        }
        "#,
        Some(remove_all_modifier),
        Some(no_changes_modifier),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

#[test]
fn benchmark_resource_builder_method_inside_blueprint_without_dev_dependencies(
) -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use scrypto::prelude::*;

        #[blueprint]
        mod blueprint {
            pub struct Hello;

            impl Hello {
                pub fn new() {
                    ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}new_fungible(OwnerRole::None);
                }
            }
        }
        "#,
        Some(remove_all_modifier),
        Some(remove_dev_dependencies_modifier),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

#[test]
fn benchmark_component_instantiation_method_inside_blueprint_with_dev_dependencies(
) -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use scrypto::prelude::*;

        #[blueprint]
        mod blueprint {
            pub struct Hello;

            impl Hello {
                pub fn new() -> Global<Hello> {
                    Self
                        .instantiate()
                        .prepare_to_globalize(OwnerRole::None)
                        .{{%EXPECT_ANY_OF:roles%}}globalize()
                }
            }
        }
        "#,
        Some(remove_all_modifier),
        Some(no_changes_modifier),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

#[test]
fn benchmark_component_instantiation_method_inside_blueprint_without_dev_dependencies(
) -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use scrypto::prelude::*;

        #[blueprint]
        mod blueprint {
            pub struct Hello;

            impl Hello {
                pub fn new() -> Global<Hello> {
                    Self
                        .instantiate()
                        .prepare_to_globalize(OwnerRole::None)
                        .{{%EXPECT_ANY_OF:roles%}}globalize()
                }
            }
        }
        "#,
        Some(remove_all_modifier),
        Some(remove_dev_dependencies_modifier),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}
*/

#[test]
fn benchmark_slower() -> Result<(), TimingError> {
    let duration = time_autocompletion(
        r#"
        use scrypto::prelude::*;

        #[blueprint]
        mod hello {
            struct Hello {
                sample_vault: Vault,
            }

            impl Hello {

                pub fn instantiate_hello() -> Global<Hello> {
                    let my_bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                        .metadata(metadata! {
                            init {
                                "name" => "HelloToken".to_owned(), locked;
                                "symbol" => "HT".to_owned(), locked;
                            }
                        })
                        .mint_initial_supply(1000);

                    {{%ResourceBuilder::new_fungible(OwnerRole::None).{{%EXPECT_ANY_OF:divisibility%}}%}}

                    Self {
                        sample_vault: Vault::with_bucket(my_bucket),
                    }
                    .instantiate()
                    .prepare_to_globalize(OwnerRole::None)
                    .globalize()
                }

                pub fn free_token(&mut self) -> Bucket {
                    info!(
                        "My balance is: {} HelloToken. Now giving away a token!",
                        self.sample_vault.amount()
                    );
                    self.sample_vault.take(1)
                }
            }
        }
        "#,
        Some(remove_all_modifier),
        Some(|manifest: &str| {
            let package_name = Manifest::from_str(manifest).unwrap().package.unwrap().name;
            format!(
                r#"
                [package]
                name = "{package_name}"
                version = "1.0.0"
                edition = "2021"
                resolver = "2"

                [dependencies]
                sbor = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "6c635d3360b1b6a352ae83234fad278b524b040e" }}
                scrypto = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "6c635d3360b1b6a352ae83234fad278b524b040e" }}

                [dev-dependencies]
                transaction = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "6c635d3360b1b6a352ae83234fad278b524b040e" }}
                radix-engine = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "6c635d3360b1b6a352ae83234fad278b524b040e" }}
                scrypto-unit = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "6c635d3360b1b6a352ae83234fad278b524b040e" }}

                [features]
                default = []
                test = []

                [lib]
                crate-type = ["cdylib", "lib"]
                "#
            )
        }),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

#[test]
fn benchmark_faster() -> Result<(), TimingError> {
    let duration = time_autocompletion(
        indoc!(
            r#"
            use scrypto::prelude::*;

            #[blueprint]
            mod hello {
                struct Hello {
                    sample_vault: Vault,
                }

                impl Hello {

                    pub fn instantiate_hello() -> Global<Hello> {
                        let my_bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                            .metadata(metadata! {
                                init {
                                    "name" => "HelloToken".to_owned(), locked;
                                    "symbol" => "HT".to_owned(), locked;
                                }
                            })
                            .mint_initial_supply(1000);

                        {{%ResourceBuilder::new_fungible(OwnerRole::None).{{%EXPECT_ANY_OF:divisibility%}}%}}

                        Self {
                            sample_vault: Vault::with_bucket(my_bucket),
                        }
                        .instantiate()
                        .prepare_to_globalize(OwnerRole::None)
                        .globalize()
                    }

                    pub fn free_token(&mut self) -> Bucket {
                        info!(
                            "My balance is: {} HelloToken. Now giving away a token!",
                            self.sample_vault.amount()
                        );
                        self.sample_vault.take(1)
                    }
                }
            }
            "#
        ),
        Some(remove_all_modifier),
        Some(|manifest: &str| {
            let package_name = Manifest::from_str(manifest).unwrap().package.unwrap().name;
            format!(
                r#"
                [package]
                name = "{package_name}"
                version = "1.0.0"
                edition = "2021"
                resolver = "2"

                [dependencies]
                sbor = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "204f2c5a2571ec5e5dab7d51c9f12b68eb684ead" }}
                scrypto = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "204f2c5a2571ec5e5dab7d51c9f12b68eb684ead" }}

                [dev-dependencies]
                transaction = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "204f2c5a2571ec5e5dab7d51c9f12b68eb684ead" }}
                radix-engine = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "204f2c5a2571ec5e5dab7d51c9f12b68eb684ead" }}
                scrypto-unit = {{ git = "https://github.com/radixdlt/radixdlt-scrypto", rev = "204f2c5a2571ec5e5dab7d51c9f12b68eb684ead" }}

                [features]
                default = []
                test = []

                [lib]
                crate-type = ["cdylib", "lib"]
                "#
            )
        }),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}

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
