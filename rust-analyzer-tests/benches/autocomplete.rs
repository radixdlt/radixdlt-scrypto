use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_analyzer_tests::*;

fn bench_collection_autocomplete_no_scrypto_dependency(c: &mut Criterion) {
    c.bench_function("collection-autocomplete-no-scrypto-dependency", |b| {
        b.iter_custom(|iters| {
            (0..iters)
                .map(|_| {
                    black_box(
                        time_autocompletion(
                            r#"
                                use std::collections::{{%EXPECT_ANY_OF:HashMap%}}BTreeSet;
                                "#,
                            Some(|_: &str| "".to_owned()),
                            Some(|_: &str| {
                                r#"
                                [package]
                                name = "test-package"
                                version = "0.0.1"
                                edition = "2021"
                                "#
                                .to_owned()
                            }),
                            LoggingHandling::NoLogging,
                        )
                        .unwrap(),
                    )
                })
                .sum()
        });
    });
}

fn bench_resource_builder_method_outside_blueprint(c: &mut Criterion) {
    c.bench_function("resource-builder-method-outside-blueprint", |b| {
        b.iter_custom(|iters| {
            (0..iters)
                .map(|_| {
                    black_box(
                        time_autocompletion(
                            r#"
                                use scrypto::prelude::*;
                                pub fn some_function() {
                                    ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
                                }
                                "#,
                            Some(|_: &str| "".to_owned()),
                            Some(|manifest_file_contents: &str| {
                                manifest_file_contents
                                    .split('\n')
                                    .filter(|line| !line.contains("scrypto-test"))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }),
                            LoggingHandling::NoLogging,
                        )
                        .unwrap(),
                    )
                })
                .sum()
        });
    });
}

fn bench_resource_builder_method_inside_blueprint_no_scrypto_test_dependency_and_no_tests(
    c: &mut Criterion,
) {
    c.bench_function("resource-builder-method-inside-blueprint-no-scrypto-test-dependency-and-no-tests", |b| {
        b.iter_custom(|iters| {
            (0..iters)
                .map(|_| {
                    black_box(
                        time_autocompletion(
                            r#"
                                use scrypto::prelude::*;

                                #[blueprint]
                                mod blueprint {
                                    pub struct Hello;

                                    impl Hello {
                                        pub fn new() -> Global<Hello> {
                                            ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
                                        }
                                    }
                                }
                                "#,
                            Some(|_: &str| "".to_owned()),
                            Some(|manifest_file_contents: &str| {
                                manifest_file_contents
                                    .split('\n')
                                    .filter(|line| !line.contains("scrypto-test"))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }),
                            LoggingHandling::NoLogging,
                        )
                        .unwrap(),
                    )
                })
                .sum()
        });
    });
}

fn bench_resource_builder_method_inside_blueprint_with_scrypto_test_dependency_and_no_tests(
    c: &mut Criterion,
) {
    c.bench_function("resource-builder-method-inside-blueprint-with-scrypto-test-dependency-and-no-tests", |b| {
        b.iter_custom(|iters| {
            (0..iters)
                .map(|_| {
                    black_box(
                        time_autocompletion(
                            r#"
                                use scrypto::prelude::*;

                                #[blueprint]
                                mod blueprint {
                                    pub struct Hello;

                                    impl Hello {
                                        pub fn new() -> Global<Hello> {
                                            ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
                                        }
                                    }
                                }
                                "#,
                            Some(|_: &str| "".to_owned()),
                            Some(|manifest_file_contents: &str| {
                                manifest_file_contents.to_owned()
                            }),
                            LoggingHandling::NoLogging,
                        )
                        .unwrap(),
                    )
                })
                .sum()
        });
    });
}

fn bench_resource_builder_method_inside_blueprint_with_scrypto_test_dependency_and_tests(
    c: &mut Criterion,
) {
    c.bench_function("resource-builder-method-inside-blueprint-with-scrypto-test-dependency-and-tests", |b| {
        b.iter_custom(|iters| {
            (0..iters)
                .map(|_| {
                    black_box(
                        time_autocompletion(
                            r#"
                                use scrypto::prelude::*;

                                #[blueprint]
                                mod blueprint {
                                    pub struct Hello;

                                    impl Hello {
                                        pub fn new() -> Global<Hello> {
                                            ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
                                        }
                                    }
                                }
                                "#,
                            Some(|tests: &str| tests.to_owned()),
                            Some(|manifest_file_contents: &str| {
                                manifest_file_contents.to_owned()
                            }),
                            LoggingHandling::NoLogging,
                        )
                        .unwrap(),
                    )
                })
                .sum()
        });
    });
}

fn bench_instantiation_method_inside_blueprint_no_scrypto_test_dependency_and_no_tests(
    c: &mut Criterion,
) {
    c.bench_function(
        "instantiation-method-inside-blueprint-no-scrypto-test-dependency-and-no-tests",
        |b| {
            b.iter_custom(|iters| {
                (0..iters)
                    .map(|_| {
                        black_box(
                            time_autocompletion(
                                r#"
                                use scrypto::prelude::*;

                                #[blueprint]
                                mod blueprint {
                                    pub struct Hello;

                                    impl Hello {
                                        pub fn new() -> Global<Hello> {
                                            Hello
                                                .instantiate()
                                                .prepare_to_globalize(OwnerRole::Fixed)
                                                .{{%EXPECT_ANY_OF:globalize,roles%}}
                                        }
                                    }
                                }
                                "#,
                                Some(|_: &str| "".to_owned()),
                                Some(|manifest_file_contents: &str| {
                                    manifest_file_contents
                                        .split('\n')
                                        .filter(|line| !line.contains("scrypto-test"))
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                }),
                                LoggingHandling::NoLogging,
                            )
                            .unwrap(),
                        )
                    })
                    .sum()
            });
        },
    );
}

fn bench_instantiation_method_inside_blueprint_with_scrypto_test_dependency_and_no_tests(
    c: &mut Criterion,
) {
    c.bench_function(
        "instantiation-method-inside-blueprint-with-scrypto-test-dependency-and-no-tests",
        |b| {
            b.iter_custom(|iters| {
                (0..iters)
                    .map(|_| {
                        black_box(
                            time_autocompletion(
                                r#"
                                use scrypto::prelude::*;

                                #[blueprint]
                                mod blueprint {
                                    pub struct Hello;

                                    impl Hello {
                                        pub fn new() -> Global<Hello> {
                                            Hello
                                                .instantiate()
                                                .prepare_to_globalize(OwnerRole::Fixed)
                                                .{{%EXPECT_ANY_OF:globalize,roles%}}
                                        }
                                    }
                                }
                                "#,
                                Some(|_: &str| "".to_owned()),
                                Some(|manifest_file_contents: &str| {
                                    manifest_file_contents.to_owned()
                                }),
                                LoggingHandling::NoLogging,
                            )
                            .unwrap(),
                        )
                    })
                    .sum()
            });
        },
    );
}

fn bench_instantiation_method_inside_blueprint_with_scrypto_test_dependency_and_tests(
    c: &mut Criterion,
) {
    c.bench_function(
        "instantiation-method-inside-blueprint-with-scrypto-test-dependency-and-tests",
        |b| {
            b.iter_custom(|iters| {
                (0..iters)
                    .map(|_| {
                        black_box(
                            time_autocompletion(
                                r#"
                                use scrypto::prelude::*;

                                #[blueprint]
                                mod blueprint {
                                    pub struct Hello;

                                    impl Hello {
                                        pub fn new() -> Global<Hello> {
                                            Hello
                                                .instantiate()
                                                .prepare_to_globalize(OwnerRole::Fixed)
                                                .{{%EXPECT_ANY_OF:globalize,roles%}}
                                        }
                                    }
                                }
                                "#,
                                Some(|tests: &str| tests.to_owned()),
                                Some(|manifest_file_contents: &str| {
                                    manifest_file_contents.to_owned()
                                }),
                                LoggingHandling::NoLogging,
                            )
                            .unwrap(),
                        )
                    })
                    .sum()
            });
        },
    );
}

criterion_group! {
  name = benches;
  config = Criterion::default().sample_size(10);
  targets = bench_collection_autocomplete_no_scrypto_dependency,
            bench_resource_builder_method_outside_blueprint,
            bench_resource_builder_method_inside_blueprint_no_scrypto_test_dependency_and_no_tests,
            bench_resource_builder_method_inside_blueprint_with_scrypto_test_dependency_and_no_tests,
            bench_resource_builder_method_inside_blueprint_with_scrypto_test_dependency_and_tests,
            bench_instantiation_method_inside_blueprint_no_scrypto_test_dependency_and_no_tests,
            bench_instantiation_method_inside_blueprint_with_scrypto_test_dependency_and_no_tests,
            bench_instantiation_method_inside_blueprint_with_scrypto_test_dependency_and_tests,
}
criterion_main!(benches);
