use super::super::*;
use super::*;
use super::common::*;
use linreg::linear_regression_of;
use radix_engine_store_interface::{
    db_key_mapper::*,
    interface::{
        CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, PartitionUpdates,
        SubstateDatabase,
    },
};
use rand::Rng;
use std::{io::Write, path::PathBuf};

#[test]
fn test_delete_per_size() {
}

#[test]
fn test_delete_per_partition() {
}

