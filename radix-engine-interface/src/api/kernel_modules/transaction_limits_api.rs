pub trait ClientTransactionLimitsApi<E> {
    fn update_wasm_memory_usage(&mut self, size: usize) -> Result<(), E>;
}
