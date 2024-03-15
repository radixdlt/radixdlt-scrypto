use crate::prelude::*;
use core::cmp::*;
use core::iter::*;

/// An iterator overlaying a "change on a value" (coming from the [`overlaying`] iterator) over a
/// "base value" (coming from the [`underlying`] iterator) which may error.
/// The one is matched to another by a `K` part (of the iterated tuple `(K, V)`), which both
/// iterators are assumed to be ordered by.
pub struct OverlayingResultIterator<U, O>
where
    U: Iterator,
    O: Iterator,
{
    underlying: Peekable<U>,
    overlaying: Peekable<O>,
    errored_out: bool,
}

impl<K, V, U, O, E> OverlayingResultIterator<U, O>
where
    K: Ord,
    U: Iterator<Item = Result<(K, V), E>>,
    O: Iterator<Item = (K, Option<V>)>,
{
    /// Creates an overlaying iterator.
    /// The [`underlying`] iterator provides the "base values" from some I/O.
    /// The [`overlaying`] one provides the "changes" to those values, represented as `Option<V>`:
    /// - A [`Some`] is an upsert, i.e. it may override an existing base value, or "insert" a
    ///   completely new one to the iterated results.
    /// - A [`None`] is a delete, which causes the base value to be omitted in the iterated results.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(underlying: U, overlaying: O) -> impl Iterator<Item = Result<(K, V), E>> {
        Self {
            underlying: underlying.peekable(),
            overlaying: overlaying.peekable(),
            errored_out: false,
        }
    }
}

impl<K, V, U, O, E> Iterator for OverlayingResultIterator<U, O>
where
    K: Ord,
    U: Iterator<Item = Result<(K, V), E>>,
    O: Iterator<Item = (K, Option<V>)>,
{
    type Item = Result<(K, V), E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored_out {
            return None;
        }

        loop {
            if let Some(overlaying_key) = self.overlaying.peek_key() {
                if let Some(underlying) = self.underlying.peek() {
                    match underlying {
                        Ok((underlying_key, _)) => {
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
                        Err(..) => {
                            self.errored_out = true;
                            return self.underlying.next();
                        }
                    }
                }

                let (overlaying_key, overlaying_change) = self.overlaying.next().unwrap();
                match overlaying_change {
                    Some(value) => return Some(Ok((overlaying_key, value))),
                    None => continue, // we may need to skip over an unbounded number of deletes
                }
            } else {
                let rtn = self.underlying.next();
                if let Some(Err(..)) = rtn {
                    self.errored_out = true;
                }
                return rtn;
            }
        }
    }
}
