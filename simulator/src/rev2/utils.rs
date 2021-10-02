use clap::ArgMatches;
use scrypto::types::*;
use std::path::PathBuf;

use crate::rev2::*;

pub fn match_address(matches: &ArgMatches, name: &str) -> Result<Address, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_string()))?
        .parse()
        .map_err(Error::InvalidAddress)
}

pub fn match_amount(matches: &ArgMatches, name: &str) -> Result<Amount, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_string()))?
        .parse()
        .map_err(Error::InvalidAmount)
}

pub fn match_u64(matches: &ArgMatches, name: &str) -> Result<u64, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_string()))?
        .parse()
        .map_err(|_| Error::InvalidNumber)
}

pub fn match_string(matches: &ArgMatches, name: &str) -> Result<String, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_string()))
        .map(ToString::to_string)
}

pub fn match_path(matches: &ArgMatches, name: &str) -> Result<PathBuf, Error> {
    Ok(PathBuf::from(
        matches
            .value_of(name)
            .ok_or_else(|| Error::MissingArgument(name.to_owned()))?,
    ))
}

pub fn match_args(matches: &ArgMatches, name: &str) -> Result<Vec<String>, Error> {
    let mut v = Vec::new();
    if let Some(x) = matches.values_of(name) {
        x.for_each(|a| v.push(a.to_owned()));
    }
    Ok(v)
}
