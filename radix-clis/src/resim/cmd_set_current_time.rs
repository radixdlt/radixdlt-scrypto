use clap::Parser;
use radix_common::prelude::*;
use radix_common::time::UtcDateTime;
use radix_engine::blueprints::consensus_manager::{
    ProposerMilliTimestampSubstate, ProposerMinuteTimestampSubstate,
};

use crate::resim::*;

/// Set the current time
#[derive(Parser, Debug)]
pub struct SetCurrentTime {
    /// UTC date time in ISO-8601 format, up to second precision, such as '2011-12-03T10:15:30Z'.
    pub date_time: UtcDateTime,
}

impl SetCurrentTime {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        let instant = self.date_time.to_instant();
        db_upsert_timestamps(
            ProposerMilliTimestampSubstate {
                epoch_milli: instant.seconds_since_unix_epoch * 1000,
            },
            ProposerMinuteTimestampSubstate {
                epoch_minute: i32::try_from(instant.seconds_since_unix_epoch / 60).unwrap(),
            },
        )?;
        writeln!(out, "Time set successfully").map_err(Error::IOError)?;
        Ok(())
    }
}
