pub trait SystemThreadApi<E> {
    fn switch_stack(&mut self, thread: usize) -> Result<(), E>;
}
