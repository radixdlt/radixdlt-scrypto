use crate::engine::*;

/// A utility for UUID generation.
#[derive(Debug)]
pub struct Uuid {}

impl Uuid {
    /// Generates an UUID.
    pub fn generate() -> u128 {
        let input = GenerateUuidInput {};
        let output: GenerateUuidOutput = call_engine(GENERATE_UUID, input);

        output.uuid
    }
}
