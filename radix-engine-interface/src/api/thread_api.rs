use crate::types::IndexedScryptoValue;

pub trait SystemThreadApi<E> {

    fn free_stack(&mut self, stack_id: usize) -> Result<(), E>;

    fn move_to_stack(&mut self, stack_id: usize, value: IndexedScryptoValue) -> Result<(), E>;

    fn switch_stack(&mut self, stack_id: usize) -> Result<(), E>;
}
