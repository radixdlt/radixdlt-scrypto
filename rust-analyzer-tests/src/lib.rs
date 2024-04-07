//! This module contains code that can be imported into integration tests or benchmarks to aid in
//! benchmarking. This code is especially helpful for autocompletion benchmarks but can be expanded
//! to support benchmarking other features of Rust Analyzer as well.
#![allow(clippy::test_attr_in_doctest)]

use std::env::var;
use std::fs::{read_to_string, write};
use std::io::Error as IOError;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Once, OnceLock};
use std::time::{Duration, Instant};

use crossbeam_channel::SendError;
use paths::AbsPathBuf;
use radix_clis_common::package::{new_package, PackageError};
use regex::Regex;
use tempfile::tempdir;

/// A function used to time the amount that it takes to get autocompletion suggestion in some code
/// using Rust analyzer.
///
/// This function allows for arbitrary source code to be specified that contains _exactly one_
/// autocompletion expectation pattern where this function should attempt to perform autocomplete
/// and report how much time was taken. An example source code with the autocomplete expectation
/// pattern is as follows:
///
/// ```rust,norun
/// use std::collection::{{%EXPECT_ANY_OF:HashMap,BTreeMap%}};
/// ```
///
/// Given the above source code, this function will find the autocomplete expectation pattern, infer
/// from the pattern that if autocomplete is done at that location the results should include either
/// `HashMap` or `BTreeMap`, if not this function will return an error. The amount of time that it
/// took for rust-analyzer to report these will be returned by this function.
///
/// This autocomplete expectation pattern is found by this function using the following regex
/// pattern: `\{\{%EXPECT_ANY_OF:([0-9a-zA-z _,]*)%\}\}`, as you can see, `EXPECT_ANY_OF` is the
/// only supported expectation pattern, but other patterns can be added too.
///
/// The autocomplete expectation pattern could be put anywhere in the code, it doesn't have to be
/// in any specific line or location. As an example, the following is a valid autocomplete
/// expectation pattern:
///
/// ```rust,norun
/// use scrypto::prelude::*;
///
/// #[blueprint]
/// mod blueprint {
///     pub struct Hello;
///
///     impl Hello {
///         pub fn new() -> Global<Hello> {
///             ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
///         }
///     }
/// }
/// ```
///
/// The autocomplete expectation pattern has two uses:
///
/// 1. It gives this function information on where in the code the autocomplete should be performed.
/// this function then translates the location of the pattern to a [`lsp_types::Position`] which is
/// understood by the Rust Analyzer language server.
/// 2. It gives the function context on what correct and incorrect autocomplete is at that location
/// in code allowing it to either succeed or fail according to the expectation of the test writer.
///
/// The passed `source_code` argument must contain _exactly one_ autocomplete expectation pattern,
/// if not then this function returns a [`TimingError::AutocompletePatternsAreNotOne`] error to the
/// caller.
///
/// Since this function is primarily meant to test the integration between Scrypto and Rust Analyzer
/// each time this function is called, the specified source code is put in the `lib.rs` file of a
/// newly created Scrypto package that has all of the Scrypto imports in the manifest files and that
/// follows the Scrypto package template in general. After this function finishes execution this
/// package is removed since its created in a temporary directory.
///
/// The caller may optionally specify callback functions used to modify the test code and the cargo
/// manifest files. These functions could be used for simple things such as overriding the contents
/// of these files or could be used for programmatic access like find and replace of the contents of
/// these files (e.g., removing just one dependency from the manifest file without making any other
/// changes.)
///
/// The following example is of this function being used to benchmark the autocomplete performance
/// of without the `scrypto-test` dependency and without any tests:
///
/// ```rust,norun
/// #[test]
/// fn benchmark_autocomplete() -> Result<(), TimingError> {
///     let duration = time_autocompletion(
///         r#"
///         use scrypto::prelude::*;
///         pub fn some_function() {
///             ResourceBuilder::{{%EXPECT_ANY_OF:new_fungible,new_non_fungible%}}
///         }
///         "#,
///         Some(|_: &str| "".to_owned()),
///         Some(|manifest_file_contents: &str| {
///             manifest_file_contents
///                 .split('\n')
///                 .filter(|line| !line.contains("scrypto-test"))
///                 .collect::<Vec<_>>()
///                 .join("\n")
///         }),
///         LoggingHandling::LogToStdOut,
///     )?;
///     println!("Autocomplete took: {}ms", duration.as_millis());
///     Ok(())
/// }
/// ```
///
/// An important note on this is that the duration reported by this function is not the total amount
/// of time this function took to execute but just the amount that it took for autocomplete to be
/// done. There is a difference between these two since part of the execution of this function is
/// the compilation of the regex pattern for the autocomplete pattern, creating a new package,
/// informing rust-analyzer of said new package to analyze it, awaiting RPC responses and some IPC
/// overhead, and eventually getting to the autocompletion.
///
/// Rust Analyzer has very extensive tracing that might be useful in debugging the execution time of
/// some of its methods, this function allows the caller to pass in a simple set of arguments for
/// configuring the tracing and logging and they're primarily around where to log the data to and
/// nothing beyond that. This function attempt to use the most relaxed logging filters so that
/// everything is logged and can be processed later on if need be. Otherwise, the `RA_LOG`,
/// `CHALK_DEBUG`, and `RA_PROFILE` can be used to set the logging level. They carry the same
/// meaning as they do in the rust-analyzer codebase.
pub fn time_autocompletion<T, M>(
    source_code: &str,
    test_code_modifier: Option<T>,
    cargo_manifest_modifier: Option<M>,
    logging_handling: LoggingHandling,
) -> Result<Duration, TimingError>
where
    T: FnOnce(&str) -> String,
    M: FnOnce(&str) -> String,
{
    // First thing we do is setup Rust Analyzer's tracing, this is important for benchmarks as it
    // provides information such as a breakdown of the time it took to perform the computation. This
    // must be executed only once per thread and therefore a Once is used here. If an error occurs
    // in this part of the code then it is a panic and and not a result.
    {
        static TRACING_INIT: Once = Once::new();
        if !TRACING_INIT.is_completed() {
            TRACING_INIT.call_once(|| match logging_handling {
                // No logging, do nothing.
                LoggingHandling::NoLogging => {}
                LoggingHandling::LogToStdOut => rust_analyzer::tracing::Config {
                    writer: tracing_subscriber::fmt::TestWriter::new(),
                    filter: var("RA_LOG").ok().unwrap_or_else(|| "error".to_owned()),
                    chalk_filter: var("CHALK_DEBUG").ok(),
                    profile_filter: var("RA_PROFILE").ok(),
                }
                .init()
                .expect("Tracing initialization failed."),
            });
        }
    }

    // Getting the autocomplete expectation patterns found in the source code. Return an error if
    // more than one autocomplete pattern is found.
    let (source_code, autocomplete_patterns) = AutocompleteExpectation::new_from_code(source_code)?;
    let [autocomplete_pattern] = autocomplete_patterns.as_slice() else {
        return Err(TimingError::AutocompletePatternsAreNotOne);
    };

    // Creating a temporary directory and a Scrypto package in that temporary directory. The Scrypto
    // package is created at a relative path to the temporary directory of ./${timestamp}/. The
    // current timestamp is used there to avoid any possibility of collisions.
    let temporary_directory = tempdir().map_err(TimingError::TemporaryDirectoryError)?;
    let crate_directory = temporary_directory
        .path()
        .join(Instant::now().elapsed().as_secs().to_string());
    new_package("test-package", Some(crate_directory.clone()), true)
        .map_err(TimingError::PackageCreationError)?;

    let source_code_path = crate_directory.join("src").join("lib.rs");
    let test_code_path = crate_directory.join("tests").join("lib.rs");
    let manifest_path = crate_directory.join("Cargo.toml");

    // Writing the source code file to the package and performing the modification on the test and
    // manifest file and writing them.
    write(&source_code_path, &source_code).map_err(TimingError::FileWriteError)?;
    if let Some(callback) = test_code_modifier {
        read_to_string(&test_code_path)
            .map_err(TimingError::FileReadError)
            .map(|content| callback(&content))
            .and_then(|new_content| {
                write(test_code_path, new_content).map_err(TimingError::FileWriteError)
            })?;
    }
    if let Some(callback) = cargo_manifest_modifier {
        read_to_string(&manifest_path)
            .map_err(TimingError::FileReadError)
            .map(|content| callback(&content))
            .and_then(|new_content| {
                write(manifest_path, new_content).map_err(TimingError::FileWriteError)
            })?;
    }

    // All of the required setup is now completed. Rust Analyzer can now be started and completion
    // requests can be sent to it.

    // Creating the channels for the rust-analyzer communication and starting rust-analyzer in a new
    // thread.
    let client_connection = {
        let absolute_crate_directory = AbsPathBuf::try_from(crate_directory)
            .map_err(TimingError::AbsolutePathBufferConversionFailed)?;

        let config = {
            let mut config = rust_analyzer::config::Config::new(
                absolute_crate_directory.clone(),
                client_capabilities(),
                vec![absolute_crate_directory],
                None,
            );
            config
                .update(serde_json::json!({
                    "cargo": {
                        "sysroot": "discover",
                        "buildScripts": {
                            "useRustcWrapper": false,
                            "enable": true,
                        },
                    },
                    // Rust analyzer should expand proc-macros.
                    "procMacro": {
                        "enable": true,
                    }
                }))
                .map_err(TimingError::ConfigurationUpdateError)?;
            config.rediscover_workspaces();
            config
        };

        let (client_connection, server_connection) = lsp_server::Connection::memory();
        std::thread::spawn(|| rust_analyzer::main_loop(config, server_connection));
        client_connection
    };

    // Send a notification to the server informing them that we've opened the lib.rs file. Without
    // doing that the file would not be stored the language server's memory and we would not be able
    // to do any auto-completion on it.
    send_notification::<lsp_types::notification::DidOpenTextDocument>(
        &client_connection,
        &lsp_types::DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: url::Url::from_file_path(&source_code_path).expect("Can't fail!"),
                language_id: "rust".to_owned(),
                version: 1,
                text: source_code.clone(),
            },
        },
    )?;

    // Await a notification from the rust-analyzer language server informing us that it is done with
    // its startup routine where it runs cargo check on the package and analyzes it.
    loop {
        let Ok(lsp_server::Message::Notification(lsp_server::Notification { method, params })) =
            client_connection.receiver.recv()
        else {
            continue;
        };

        if method == "experimental/serverStatus"
            && matches!(
                params.get("quiescent"),
                Some(&serde_json::Value::Bool(true))
            )
        {
            break;
        }
    }

    // Send the server the autocomplete request
    send_request::<lsp_types::request::Completion>(
        &client_connection,
        // Large id to avoid possibility of collision.
        100_000_000.into(),
        &lsp_types::CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier {
                    uri: url::Url::from_file_path(&source_code_path).expect("Can't fail!"),
                },
                position: autocomplete_pattern.position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: Default::default(),
        },
    )?;

    // Await a response with the same request id.
    let (duration, completion_result) = time_execution(|| -> Result<_, TimingError> {
        loop {
            let Ok(lsp_server::Message::Response(lsp_server::Response { id, result, error })) =
                client_connection.receiver.recv()
            else {
                continue;
            };
            if id != 100_000_000.into() {
                continue;
            }

            let result = match (result, error) {
                (Some(result), None) => Ok(result),
                (None, Some(error)) => Err(error),
                (Some(_), Some(_)) | (None, None) => unreachable!(),
            }
            .map_err(TimingError::AutocompleteError)?;

            let completion_result = serde_json::from_value::<
                <lsp_types::request::Completion as lsp_types::request::Request>::Result,
            >(result)
            .expect("Can't happen");

            break Ok(completion_result);
        }
    });

    // Ensure the completion result satisfies the completion pattern.
    match (completion_result?, &autocomplete_pattern.pattern) {
        (None, AutocompleteExpectationPattern::AnyOf(any_of)) if any_of.is_empty() => {}
        (None, AutocompleteExpectationPattern::AnyOf(_)) => {
            return Err(TimingError::AutocompleteExpectationNotMet);
        }
        (
            Some(
                lsp_types::CompletionResponse::Array(items)
                | lsp_types::CompletionResponse::List(lsp_types::CompletionList { items, .. }),
            ),
            AutocompleteExpectationPattern::AnyOf(any_of),
        ) => {
            if !items.iter().any(|item| any_of.contains(&item.label)) {
                return Err(TimingError::AutocompleteExpectationNotMet);
            }
        }
    }

    Ok(duration)
}

// The capabilities of the LSP client (us) as seen in the rust-analyzer codebase:
// https://github.com/rust-lang/rust-analyzer/blob/d9c29afaee6cb26044b5a605e0073fcabb2e9722/crates/rust-analyzer/tests/slow-tests/support.rs#L127-L175
fn client_capabilities() -> lsp_types::ClientCapabilities {
    lsp_types::ClientCapabilities {
        workspace: Some(lsp_types::WorkspaceClientCapabilities {
            did_change_watched_files: Some(lsp_types::DidChangeWatchedFilesClientCapabilities {
                dynamic_registration: Some(true),
                relative_pattern_support: None,
            }),
            workspace_edit: Some(lsp_types::WorkspaceEditClientCapabilities {
                resource_operations: Some(vec![
                    lsp_types::ResourceOperationKind::Create,
                    lsp_types::ResourceOperationKind::Delete,
                    lsp_types::ResourceOperationKind::Rename,
                ]),
                ..Default::default()
            }),
            ..Default::default()
        }),
        text_document: Some(lsp_types::TextDocumentClientCapabilities {
            definition: Some(lsp_types::GotoCapability {
                link_support: Some(true),
                ..Default::default()
            }),
            code_action: Some(lsp_types::CodeActionClientCapabilities {
                code_action_literal_support: Some(lsp_types::CodeActionLiteralSupport::default()),
                ..Default::default()
            }),
            hover: Some(lsp_types::HoverClientCapabilities {
                content_format: Some(vec![lsp_types::MarkupKind::Markdown]),
                ..Default::default()
            }),
            ..Default::default()
        }),
        window: Some(lsp_types::WindowClientCapabilities {
            work_done_progress: Some(false),
            ..Default::default()
        }),
        experimental: Some(serde_json::json!({
            "serverStatusNotification": true,
        })),
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub enum LoggingHandling {
    NoLogging,
    LogToStdOut,
}

#[derive(Clone, Debug)]
pub struct AutocompleteExpectation {
    pub position: lsp_types::Position,
    pub pattern: AutocompleteExpectationPattern,
}

impl AutocompleteExpectation {
    pub fn new_from_code(code: &str) -> Result<(String, Vec<Self>), TimingError> {
        // Using a once lock here so that the pattern only needs to be compiled once. It's static so
        // the fact that it's scoped to this function is just syntactic sugar.
        let pattern = {
            static REGEX_PATTERN: OnceLock<Regex> = OnceLock::new();
            REGEX_PATTERN.get_or_init(|| {
                Regex::new(r"\{\{%EXPECT_ANY_OF:([0-9a-zA-z _,]*)%\}\}")
                    .expect("Regex pattern is not valid")
            })
        };

        // Capture all of the strings that match this pattern in the code.
        let mut patterns_found = Vec::new();
        let captures = pattern.captures_iter(code);
        for capture in captures {
            // The `complete_pattern_match` is used to extract the position of the pattern which is
            // translated into `lsp_types::Position` and passed to Rust Analyzer eventually. The
            // `types_group_match` is the types to expect.
            let (Some(complete_pattern_match), Some(types_group_match)) =
                (capture.get(0), capture.get(1))
            else {
                return Err(TimingError::RegexMatchError);
            };

            // Get position from `complete_pattern_match`. Note that in the LSP standard the line is
            // 0 indexed and the column is 1 indexed. When we find the column we must add 1.
            let position = code
                .split('\n')
                .enumerate()
                .find_map(|(line_number, line)| {
                    line.find(complete_pattern_match.as_str()).map(|column| {
                        lsp_types::Position::new(line_number as u32, (column + 1) as u32)
                    })
                })
                .expect("Pattern was captured but not found in file?");

            // Extract the type idents found in the pattern.
            let pattern = AutocompleteExpectationPattern::AnyOf(
                types_group_match
                    .as_str()
                    .trim()
                    .trim_matches(',')
                    .split(',')
                    .map(|item| item.trim().to_owned())
                    .collect::<Vec<_>>(),
            );
            patterns_found.push(Self { pattern, position });
        }

        // Remove all occurrences of the pattern in the code.
        let code_without_pattern = pattern.replace_all(code, "").deref().to_owned();

        Ok((code_without_pattern, patterns_found))
    }
}

#[derive(Clone, Debug)]
pub enum AutocompleteExpectationPattern {
    AnyOf(Vec<String>),
}

#[derive(Debug)]
pub enum TimingError {
    RegexMatchError,
    AutocompletePatternsAreNotOne,
    TemporaryDirectoryError(IOError),
    FileReadError(IOError),
    FileWriteError(IOError),
    PackageCreationError(PackageError),
    AbsolutePathBufferConversionFailed(PathBuf),
    ConfigurationUpdateError(rust_analyzer::config::ConfigError),
    SerializationError(serde_json::Error),
    DeserializationError(serde_json::Error),
    ChannelSendingError(SendError<lsp_server::Message>),
    AutocompleteError(lsp_server::ResponseError),
    AutocompleteExpectationNotMet,
}

fn send_notification<N: lsp_types::notification::Notification>(
    connection: &lsp_server::Connection,
    params: &N::Params,
) -> Result<(), TimingError> {
    // Serialize the input to a serde value.
    let serialized_params =
        serde_json::to_value(params).map_err(TimingError::SerializationError)?;

    // Construct a message from the parameters.
    let message = lsp_server::Message::Notification(lsp_server::Notification {
        method: N::METHOD.to_owned(),
        params: serialized_params,
    });

    // Write the message to the channel.
    connection
        .sender
        .send(message)
        .map_err(TimingError::ChannelSendingError)?;

    Ok(())
}

fn send_request<R: lsp_types::request::Request>(
    connection: &lsp_server::Connection,
    request_id: lsp_server::RequestId,
    params: &R::Params,
) -> Result<(), TimingError> {
    // Serialize the input to a serde value.
    let serialized_params =
        serde_json::to_value(params).map_err(TimingError::SerializationError)?;

    // Construct a message from the parameters.
    let message = lsp_server::Message::Request(lsp_server::Request {
        id: request_id,
        method: R::METHOD.to_owned(),
        params: serialized_params,
    });

    // Write the message to the channel.
    connection
        .sender
        .send(message)
        .map_err(TimingError::ChannelSendingError)?;

    Ok(())
}

fn time_execution<F, O>(func: F) -> (Duration, O)
where
    F: FnOnce() -> O,
{
    let before = Instant::now();
    let out = func();
    let after = Instant::now();
    let duration = after - before;
    (duration, out)
}
