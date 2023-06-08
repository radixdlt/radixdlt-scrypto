pub trait ClientExecutionTraceApi<E> {
    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), E>;
}
