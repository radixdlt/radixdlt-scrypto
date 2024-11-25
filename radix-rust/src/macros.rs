/// Attempts to be a replacement for `assert!(matches!(...))` but with better error messages,
/// and allowing further code on success.
///
/// Matches the [`assert_eq!`] syntax for error messages.
///
/// ```rust
/// # use radix_rust::assert_matches;
/// let x = Some(42);
/// assert_matches!(x, Some(_));
/// ```
///
/// ```rust
/// # use radix_rust::assert_matches;
/// # let x = Some(42);
/// assert_matches!(x, Some(x) => { assert_eq!(x, 42); });
/// ```
///
/// ```rust,should_panic
/// # use radix_rust::assert_matches;
/// # let x = Some(42);
/// assert_matches!(x, None, "Expected None, got {:?}", x);
/// ```
///
/// ```rust,should_panic
/// # use radix_rust::assert_matches;
/// # let x = Some(42);
/// assert_matches!(x, Some(x) => { assert_eq!(x, 41); }, "Expected Some(41), got {:?}", x);
/// ```
///
/// ## Alternatives
/// We can't use the std [assert_matches!](https://github.com/rust-lang/rust/issues/82775) macro,
/// as it looks like it is doomed to be stuck unstabilized for the foreseeable future.
///
/// This takes some inspiration regarding the `=> {}` syntax from the
/// [assert_matches](https://docs.rs/assert_matches/1.5.0/assert_matches/macro.assert_matches.html)
/// crate but the code had bugs with trailing commas and its error messages weren't ideal.
#[macro_export]
macro_rules! assert_matches {
    ($expression:expr, $pattern:pat $(if $condition:expr)? $(,)?) => {
        match $expression {
            $pattern $(if $condition)? => (),
            ref expression => panic!(
                "assertion `left matches right` failed\n  left: {:?}\n right: {}",
                expression,
                stringify!($pattern $(if $condition)?),
            )
        }
    };
    ($expression:expr, $pattern:pat $(if $condition:expr)? => $code:expr $(,)?) => {
        match $expression {
            $pattern $(if $condition)? => $code,
            ref expression => panic!(
                "assertion `left matches right` failed\n  left: {:?}\n right: {}",
                expression,
                stringify!($pattern $(if $condition)?),
            )
        }
    };
    ($expression:expr, $pattern:pat $(if $condition:expr)?, $($arg:tt)+) => {
        match $expression {
            $pattern $(if $condition)? => (),
            ref expression => panic!(
                "assertion `left matches right` failed: {}\n  left: {:?}\n right: {}",
                format_args!($($arg)+),
                expression,
                stringify!($pattern $(if $condition)?),
            )
        }
    };
    ($expression:expr, $pattern:pat $(if $condition:expr)? => $code:expr, $($arg:tt)+) => {
        match $expression {
            $pattern $(if $condition)? => $code,
            ref expression => panic!(
                "assertion `left matches right` failed: {}\n  left: {:?}\n right: {}",
                format_args!($($arg)+),
                expression,
                stringify!($pattern $(if $condition)?),
            )
        }
    };
}
