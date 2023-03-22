use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::clock::{
    ClockSetCurrentTimeInput, CLOCK_SET_CURRENT_TIME_IDENT,
};
use radix_engine_interface::time::UtcDateTime;
use transaction::model::Instruction;

use crate::resim::*;

/// Set the current time
#[derive(Parser, Debug)]
pub struct SetCurrentTime {
    /// UTC date time in ISO-8601 format, up to second precision, such as '2011-12-03T10:15:30Z'.
    pub date_time: UtcDateTime,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

impl SetCurrentTime {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let instructions = vec![Instruction::CallMethod {
            component_address: CLOCK,
            method_name: CLOCK_SET_CURRENT_TIME_IDENT.to_string(),
            args: to_manifest_value(&ClockSetCurrentTimeInput {
                current_time_ms: self.date_time.to_instant().seconds_since_unix_epoch * 1000,
            }),
        }];

        let blobs = vec![];
        let initial_proofs = btreeset![
            AuthAddresses::system_role(),
            AuthAddresses::validator_role(),
        ];
        handle_system_transaction(instructions, blobs, initial_proofs, self.trace, true, out)
            .map(|_| ())
    }
}
