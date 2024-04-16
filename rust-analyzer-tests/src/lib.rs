//! This module contains code that can be imported into integration tests or benchmarks to aid in
//! benchmarking. This code is especially helpful for autocompletion benchmarks but can be expanded
//! to support benchmarking other features of Rust Analyzer as well.
#![allow(clippy::test_attr_in_doctest)]

use std::collections::HashSet;
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
/// use std::collection::{{%EXPECT_ANY_OF:HashSet,BTreeMap%}};
/// use std::collection::{{%EXPECT_ANY_OF:HashSet,BTreeMap%}};
/// ```
///
/// Given the above source code, this function will find the autocomplete expectation pattern, infer
/// from the pattern that if autocomplete is done at that location the results should include either
/// `HashSet` or `BTreeMap`, if not this function will return an error. The amount of time that it
/// `HashSet` or `BTreeMap`, if not this function will return an error. The amount of time that it
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
    template_source_code: &str,
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
    let autocomplete_document = AutocompleteDocument::new_from_code(template_source_code)?;

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

    let source_code_uri = url::Url::from_file_path(&source_code_path).expect("Can't fail!");

    // Writing the source code file to the package and performing the modification on the test and
    // manifest file and writing them.
    write(&source_code_path, &autocomplete_document.starting_code)
        .map_err(TimingError::FileWriteError)?;
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
                .update(
                    serde_json::from_str::<serde_json::Value>(include_str!(
                        "../assets/config.json"
                    ))
                    .unwrap(),
                )
                .map_err(TimingError::ConfigurationUpdateError)?;
            config.rediscover_workspaces();
            config
        };

        let (client_connection, server_connection) = lsp_server::Connection::memory();
        std::thread::spawn(|| rust_analyzer::main_loop(config, server_connection));
        client_connection
    };

    // At this point before we open the file Rust Analyzer will do a number of checks including on
    // the fly code checks, crate indexing, and other checks. The checks that it does are known to
    // us. Therefore, we will wait until Rust Analyzer finishes all of those checks and then we will
    // continue forward.
    {
        let mut remaining_checks = [
            "rustAnalyzer/Indexing",
            "rustAnalyzer/Building CrateGraph",
            "rustAnalyzer/Fetching",
            "rustAnalyzer/Roots Scanned",
            "rust-analyzer/flycheck/0",
            "rustAnalyzer/Building build-artifacts",
            "rustAnalyzer/Loading proc-macros",
        ]
        .into_iter()
        .map(ToOwned::to_owned)
        .map(lsp_types::NumberOrString::String)
        .collect::<HashSet<_>>();

        loop {
            let Ok(message) = client_connection.receiver.recv() else {
                continue;
            };

            match message {
                lsp_server::Message::Request(request) => {
                    if <lsp_types::request::WorkDoneProgressCreate as lsp_types::request::Request>::METHOD
                        == request.method.as_str()
                    {
                        send_response::<lsp_types::request::WorkDoneProgressCreate>(
                            &client_connection,
                            request.id,
                            Ok(&()),
                        )?;
                    }
                }
                // Await a progress notification informing us that it's finished
                lsp_server::Message::Notification(notification) => {
                    if <lsp_types::notification::Progress as lsp_types::notification::Notification>::METHOD
                        == notification.method.as_str()
                    {
                        // Decode
                        let params = serde_json::from_value::<lsp_types::ProgressParams>(notification.params)
                            .expect("Can't happen, server error");

                        // We only care about `End` tokens
                        let lsp_types::ProgressParams {
                            token,
                            value:
                                lsp_types::ProgressParamsValue::WorkDone(lsp_types::WorkDoneProgress::End(..)),
                        } = params
                        else {
                            continue;
                        };

                        // Remove it from the set of items we're waiting on.
                        remaining_checks.remove(&token);

                        // If there are no remaining checks then we break!
                        if remaining_checks.is_empty() {
                            break;
                        }
                    }
                }
                // Nothing to do about responses.
                lsp_server::Message::Response(_) => {}
            }
        }
    }

    // Send a notification to the server informing them that we've opened the lib.rs file. Without
    // doing that the file would not be stored the language server's memory and we would not be able
    // to do any auto-completion on it.
    send_notification::<lsp_types::notification::DidOpenTextDocument>(
        &client_connection,
        &lsp_types::DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: source_code_uri.clone(),
                language_id: "rust".to_owned(),
                version: 1,
                text: autocomplete_document.starting_code.clone(),
            },
        },
    )?;

    // The allocator used for allocating new request ids for the various requests we will be making.
    let mut id_allocator = IdAllocator::new();

    // Right after opening the file run a full sematic analysis over the file.
    {
        let request_id = id_allocator.next();
        send_request::<lsp_types::request::SemanticTokensFullRequest>(
            &client_connection,
            request_id.clone(),
            &lsp_types::SemanticTokensParams {
                text_document: lsp_types::TextDocumentIdentifier {
                    uri: source_code_uri.clone(),
                },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            },
        )?;
        loop {
            let Ok(lsp_server::Message::Response(response)) = client_connection.receiver.recv()
            else {
                continue;
            };
            if response.id == request_id {
                break;
            }
        }
    };

    // Make changes to the `lib.rs` file and notify the server of them. We will now write the
    // item that we want to get autocomplete for.
    write(source_code_path, &autocomplete_document.final_code)
        .map_err(TimingError::FileWriteError)?;
    send_notification::<lsp_types::notification::DidChangeTextDocument>(
        &client_connection,
        &lsp_types::DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier {
                uri: source_code_uri.clone(),
                version: 1,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                text: autocomplete_document.final_code,
                range_length: None,
            }],
        },
    )?;

    // Sending the autocomplete request
    let (duration, completion_result) = time_execution(|| -> Result<_, TimingError> {
        let request_id = id_allocator.next();
        send_request::<lsp_types::request::Completion>(
            &client_connection,
            request_id.clone(),
            &lsp_types::CompletionParams {
                text_document_position: dbg!(lsp_types::TextDocumentPositionParams {
                    text_document: lsp_types::TextDocumentIdentifier {
                        uri: source_code_uri,
                    },
                    position: autocomplete_document.autocomplete_position,
                }),
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
                context: Default::default(),
            },
        )?;
        loop {
            let Ok(lsp_server::Message::Response(lsp_server::Response { id, result, error })) =
                client_connection.receiver.recv()
            else {
                continue;
            };
            if id != request_id {
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
    match (
        dbg!(completion_result?),
        &autocomplete_document.autocomplete_expectations,
    ) {
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
            if !items.iter().any(|item| any_of.contains(dbg!(&item.label))) {
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
            work_done_progress: Some(true),
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    Unexpected,
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

fn send_response<R: lsp_types::request::Request>(
    connection: &lsp_server::Connection,
    request_id: lsp_server::RequestId,
    response: Result<&R::Result, lsp_server::ResponseError>,
) -> Result<(), TimingError> {
    // Construct a message from the parameters.
    let message = match response {
        Ok(response) => lsp_server::Message::Response(lsp_server::Response {
            id: request_id,
            result: Some(serde_json::to_value(response).map_err(TimingError::SerializationError)?),
            error: None,
        }),
        Err(error) => lsp_server::Message::Response(lsp_server::Response {
            id: request_id,
            result: None,
            error: Some(error),
        }),
    };

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

pub struct IdAllocator(i32);

impl IdAllocator {
    pub fn new() -> Self {
        Self(0)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> lsp_server::RequestId {
        let id = lsp_server::RequestId::from(self.0);
        self.0 += 1;
        id
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AutocompleteDocument {
    /// The original code template as it was passed in by the user with the item identifier and the
    /// autocomplete pattern.
    ///
    /// ## Example
    ///
    /// ```txt
    /// fn some_function() {
    ///     {{%
    ///         ResourceBuilder::new_fungible(OwnerRole::None)
    ///             .{{%EXPECT_ANY_OF:divisibility%}}divisibility(18);
    ///     %}}}
    /// }
    /// ```
    code_template: String,
    /// The original code without the item identified by the item identifier. Typically this is what
    /// is written to the `lib.rs` file when the package is first created since the item might not
    /// be syntactically valid (e.g., it might end with a dot)
    ///
    /// /// ## Example
    ///
    /// ```txt
    /// fn some_function() {
    /// }
    /// ```
    starting_code: String,
    /// The original code but without the item identifiers and with the contents of it. Typically
    /// this is the code that is written to the file after the first pass of analysis to trigger
    /// autocompletion on.
    ///
    /// ## Example
    ///
    /// ```txt
    /// fn some_function() {
    ///     ResourceBuilder::new_fungible(OwnerRole::None)
    ///         .divisibility(18);
    /// }
    /// ```
    final_code: String,
    /// The position to perform the autocomplete at.
    autocomplete_position: lsp_types::Position,
    /// The autocomplete expectations.
    autocomplete_expectations: AutocompleteExpectationPattern,
    /// The starting position of the entire item identifier pattern.
    position_of_entire_item_identifier_pattern: lsp_types::Position,
    /// The changes between the starting and the final version.
    changes: String,
    /// The range where the changes were made.
    changes_range: lsp_types::Range,
}

impl AutocompleteDocument {
    pub fn new_from_code(code: &str) -> Result<Self, TimingError> {
        // Capture the regex pattern in the document.
        let pattern = Self::regex_pattern();
        let mut captures = pattern.captures_iter(code);

        // Ensure that there is only a single capture, as in, there is only a single item identifier
        // that is needed.
        let (Some(capture), None) = (captures.next(), captures.next()) else {
            return Err(TimingError::RegexMatchError);
        };

        // Extract useful information from the capture.

        // Eg: {{%ResourceBuilder::new_fungible(OwnerRole::None).{{%EXPECT_ANY_OF:divisibility%}}%}}
        let entire_item_identifier_pattern = capture.get(0).ok_or(TimingError::RegexMatchError)?;
        // Eg: ResourceBuilder::new_fungible
        let item_pre_autocomplete_pattern = capture.get(1).ok_or(TimingError::RegexMatchError)?;
        // Eg: {{%EXPECT_ANY_OF:divisibility%}}
        let entire_autocomplete_pattern = capture.get(2).ok_or(TimingError::RegexMatchError)?;
        // Eg: EXPECT_ANY_OF
        let _ = capture.get(3).ok_or(TimingError::RegexMatchError)?;
        // Eg: divisibility
        let autocomplete_qualifier_args = capture.get(4).ok_or(TimingError::RegexMatchError)?;
        // Eg: .
        let item_post_autocomplete_pattern = capture.get(5).ok_or(TimingError::RegexMatchError)?;

        let code_template = code.to_owned();
        let starting_code = pattern.replace(code, "").into_owned();
        let final_code = pattern.replace(code, "$1$5").into_owned();

        let autocomplete_position = pattern
            .replace(code, "$1$2$5")
            .split('\n')
            .enumerate()
            .find_map(|(line_number, line)| {
                line.find(entire_autocomplete_pattern.as_str())
                    .map(|column| lsp_types::Position::new(line_number as u32, column as u32))
            })
            .expect("Pattern was captured but not found in file?");
        let autocomplete_expectations = AutocompleteExpectationPattern::AnyOf(
            autocomplete_qualifier_args
                .as_str()
                .trim()
                .trim_matches(',')
                .split(',')
                .map(|item| item.trim().to_owned())
                .collect::<Vec<_>>(),
        );

        let position_of_entire_item_identifier_pattern = (0..entire_item_identifier_pattern
            .start())
            .zip(code_template.chars())
            .fold(lsp_types::Position::new(0, 0), |mut acc, (_, char)| {
                // If char is '\n' then reset the character to zero and increment the line.
                match char {
                    '\n' => {
                        acc.line += 1;
                        acc.character = 0;
                        acc
                    }
                    _ => {
                        acc.character += 1;
                        acc
                    }
                }
            });

        let changes = format!(
            "{}{}",
            item_pre_autocomplete_pattern.as_str(),
            item_post_autocomplete_pattern.as_str()
        );
        let changes_range = lsp_types::Range::new(
            position_of_entire_item_identifier_pattern,
            lsp_types::Position::new(
                position_of_entire_item_identifier_pattern.line,
                position_of_entire_item_identifier_pattern.character + changes.len() as u32,
            ),
        );

        Ok(Self {
            code_template,
            starting_code,
            final_code,
            autocomplete_position,
            autocomplete_expectations,
            position_of_entire_item_identifier_pattern,
            changes,
            changes_range,
        })
    }

    fn regex_pattern() -> &'static Regex {
        static REGEX_PATTERN: OnceLock<Regex> = OnceLock::new();
        REGEX_PATTERN.get_or_init(|| {
            Regex::new(r"(?s)\{\{%(.*)(\{\{%(EXPECT_ANY_OF):([0-9a-zA-z _,]*)%\}\})(.*)%}}")
                .expect("Must be valid!")
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_autocomplete_document() {
        // Arrange
        const DOCUMENT: &str = indoc!(
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
        );

        // Act
        let document = AutocompleteDocument::new_from_code(DOCUMENT).unwrap();

        // Assert
        assert_eq!(document.code_template, DOCUMENT);
        assert_eq!(
            document.starting_code,
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
            )
        );
        assert_eq!(
            document.final_code,
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
    
                            ResourceBuilder::new_fungible(OwnerRole::None).
    
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
            )
        );
        assert_eq!(
            document.autocomplete_position,
            lsp_types::Position::new(20, 59)
        );
        assert_eq!(
            document.autocomplete_expectations,
            AutocompleteExpectationPattern::AnyOf(vec!["divisibility".to_owned()])
        );
        assert_eq!(
            document.position_of_entire_item_identifier_pattern,
            lsp_types::Position::new(20, 12)
        );
        assert_eq!(
            document.changes,
            "ResourceBuilder::new_fungible(OwnerRole::None)."
        );
        assert_eq!(
            document.changes_range,
            lsp_types::Range::new(
                lsp_types::Position::new(20, 12),
                lsp_types::Position::new(20, 59)
            )
        );
    }
}
