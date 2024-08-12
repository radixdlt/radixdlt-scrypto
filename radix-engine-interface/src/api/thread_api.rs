use crate::types::IndexedScryptoValue;
use radix_common::crypto::Hash;

pub trait SystemThreadApi<E> {
    fn switch_stack(
        &mut self,
        to_stack_id: Hash,
    ) -> Result<(), E>;

    fn send_to_stack(
        &mut self,
        to_stack_id: Hash,
        value: IndexedScryptoValue,
    ) -> Result<(), E>;

    fn free_and_switch_stack(&mut self, to_stack_id: Hash) -> Result<(), E>;
}
