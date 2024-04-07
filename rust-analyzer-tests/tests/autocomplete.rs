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
        Some(|_: &str| "".to_owned()),
        Some(|manifest_file_contents: &str| {
            manifest_file_contents
                .split('\n')
                .filter(|line| !line.contains("scrypto-test"))
                .collect::<Vec<_>>()
                .join("\n")
        }),
        LoggingHandling::LogToStdOut,
    )?;
    println!("Autocomplete took: {}ms", duration.as_millis());
    Ok(())
}
