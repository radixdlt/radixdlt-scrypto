use crate::types::IndexedScryptoValue;

pub trait SystemThreadApi<E> {
    fn send(&mut self, thread: usize, value: IndexedScryptoValue) -> Result<(), E>;

    fn context_switch(&mut self, thread: usize) -> Result<(), E>;
}
