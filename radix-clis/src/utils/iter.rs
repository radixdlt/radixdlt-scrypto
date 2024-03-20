use std::iter;

/// A trait for identifying the last element.
pub trait IdentifyLast: Iterator + Sized {
    fn identify_last(self) -> Iter<Self>;
}

impl<I: Iterator> IdentifyLast for I {
    fn identify_last(self) -> Iter<Self> {
        Iter(self.peekable())
    }
}

/// An iterator that supports `IdentifyLast`.
pub struct Iter<I: Iterator>(iter::Peekable<I>);

impl<I: Iterator> Iterator for Iter<I> {
    type Item = (bool, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| (self.0.peek().is_none(), e))
    }
}
