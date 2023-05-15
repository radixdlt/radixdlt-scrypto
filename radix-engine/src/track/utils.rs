use crate::types::*;
use sbor::rust::cmp::*;
use sbor::rust::iter::*;

/// An iterator overlaying a "change on a value" (coming from the [`overlaid`] iterator) over a
/// "base value" (coming from the [`underlying`] iterator).
/// The one is matched to another by a `K` part (of the iterated tuple `(K, V)`), which both
/// iterators are assumed to be ordered by.
pub struct OverlayingIterator<U, O>
where
    U: Iterator,
    O: Iterator,
{
    underlying: Peekable<U>,
    overlaid: Peekable<O>,
}

impl<K, V, U, O> OverlayingIterator<U, O>
where
    K: Ord,
    U: Iterator<Item = (K, V)>,
    O: Iterator<Item = (K, Option<V>)>,
{
    /// Creates an overlaying iterator.
    /// The [`underlying`] iterator provides the "base values".
    /// The [`overlaid`] one provides the "changes" to those values, represented as `Option<V>`:
    /// - A [`Some`] is an upsert, i.e. it may override an existing base value, or "insert" a
    ///   completely new one to the iterated results.
    /// - A [`None`] is a delete, which causes the base value to be omitted in the iterated results.
    pub fn new(underlying: U, overlaid: O) -> impl Iterator<Item = (K, V)> {
        Self {
            underlying: underlying.peekable(),
            overlaid: overlaid.peekable(),
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
            if let Some(overlaid_key) = self.overlaid.peek_key() {
                if let Some(underlying_key) = self.underlying.peek_key() {
                    match underlying_key.cmp(overlaid_key) {
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
                let (overlaid_key, overlaid_change) = self.overlaid.next().unwrap();
                match overlaid_change {
                    Some(value) => return Some((overlaid_key, value)),
                    None => continue, // we may need to skip over an unbounded number of deletes
                }
            } else {
                return self.underlying.next();
            }
        }
    }
}

/// An internal [`Peekable`] extension trait; only for easier syntax.
trait PeekableKeyExt<'a, K> {
    /// Peeks at the next entry's key.
    fn peek_key(&'a mut self) -> Option<&'a K>;
}

impl<'a, K, V: 'a, I> PeekableKeyExt<'a, K> for Peekable<I>
where
    I: Iterator<Item = (K, V)>,
{
    fn peek_key(&'a mut self) -> Option<&'a K> {
        self.peek().map(|(key, _value)| key)
    }
}
