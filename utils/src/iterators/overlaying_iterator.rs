use crate::prelude::*;
use core::cmp::*;
use core::iter::*;

/// An iterator overlaying a "change on a value" (coming from the [`overlaying`] iterator) over a
/// "base value" (coming from the [`underlying`] iterator).
/// The one is matched to another by a `K` part (of the iterated tuple `(K, V)`), which both
/// iterators are assumed to be ordered by.
pub struct OverlayingIterator<U, O>
where
    U: Iterator,
    O: Iterator,
{
    underlying: Peekable<U>,
    overlaying: Peekable<O>,
}

impl<K, V, U, O> OverlayingIterator<U, O>
where
    K: Ord,
    U: Iterator<Item = (K, V)>,
    O: Iterator<Item = (K, Option<V>)>,
{
    /// Creates an overlaying iterator.
    /// The [`underlying`] iterator provides the "base values".
    /// The [`overlaying`] one provides the "changes" to those values, represented as `Option<V>`:
    /// - A [`Some`] is an upsert, i.e. it may override an existing base value, or "insert" a
    ///   completely new one to the iterated results.
    /// - A [`None`] is a delete, which causes the base value to be omitted in the iterated results.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(underlying: U, overlaying: O) -> impl Iterator<Item = (K, V)> {
        Self {
            underlying: underlying.peekable(),
            overlaying: overlaying.peekable(),
        }
    }
}

impl<K, V, U, O> Iterator for OverlayingIterator<U, O>
where
    K: Ord,
    U: Iterator<Item = (K, V)>,
    O: Iterator<Item = (K, Option<V>)>,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(overlaying_key) = self.overlaying.peek_key() {
                if let Some(underlying_key) = self.underlying.peek_key() {
                    match underlying_key.cmp(overlaying_key) {
                        Ordering::Less => {
                            return self.underlying.next(); // return and move it forward
                        }
                        Ordering::Equal => {
                            self.underlying.next(); // only move it forward
                        }
                        Ordering::Greater => {
                            // leave it as-is
                        }
                    };
                }
                let (overlaying_key, overlaying_change) = self.overlaying.next().unwrap();
                match overlaying_change {
                    Some(value) => return Some((overlaying_key, value)),
                    None => continue, // we may need to skip over an unbounded number of deletes
                }
            } else {
                return self.underlying.next();
            }
        }
    }
}
