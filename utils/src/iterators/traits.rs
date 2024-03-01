use core::iter::Peekable;

/// An internal [`Peekable`] extension trait; only for easier syntax.
pub trait PeekableKeyExt<'a, K> {
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
