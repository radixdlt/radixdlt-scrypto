pub trait ClientMeteringApi<W, E> {
    fn consume_cost_units(&mut self, units: u32) -> Result<(), E>;
}
