use crate::types::IndexedScryptoValue;

pub trait SystemThreadApi<E> {
    fn send(&mut self, thread: usize, value: IndexedScryptoValue) -> Result<(), E>;

    fn switch_context(&mut self, thread: usize) -> Result<(), E>;

    fn join(&mut self, thread: usize) -> Result<(), E>;
}
