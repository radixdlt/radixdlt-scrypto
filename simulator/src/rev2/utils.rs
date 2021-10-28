use clap::ArgMatches;
use scrypto::types::*;
use std::path::PathBuf;

use crate::rev2::*;

/// Match an address argument.
pub fn match_address(matches: &ArgMatches, name: &str) -> Result<Address, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)
}

/// Match an amount argument.
pub fn match_amount(matches: &ArgMatches, name: &str) -> Result<Amount, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_owned()))?
        .parse()
        .map_err(Error::InvalidAmount)
}

/// Match a u64 argument.
pub fn match_u64(matches: &ArgMatches, name: &str) -> Result<u64, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_owned()))?
        .parse()
        .map_err(Error::InvalidNumber)
}

/// Match a string argument.
pub fn match_string(matches: &ArgMatches, name: &str) -> Result<String, Error> {
    matches
        .value_of(name)
        .ok_or_else(|| Error::MissingArgument(name.to_owned()))
        .map(String::from)
}

/// Match a file path argument.
pub fn match_path(matches: &ArgMatches, name: &str) -> Result<PathBuf, Error> {
    Ok(PathBuf::from(
        matches
            .value_of(name)
            .ok_or_else(|| Error::MissingArgument(name.to_owned()))?,
    ))
}

/// Match variable length args argument.
pub fn match_args(matches: &ArgMatches, name: &str) -> Result<Vec<String>, Error> {
    let mut v = Vec::new();
    if let Some(x) = matches.values_of(name) {
        x.for_each(|a| v.push(a.to_owned()));
    }
    Ok(v)
}

/// Match signers
pub fn match_signers(matches: &ArgMatches, name: &str) -> Result<Vec<Address>, Error> {
    let mut v = Vec::<Address>::new();
    if let Some(x) = matches.value_of(name) {
        for a in x.split(',') {
            let address: Address = a.trim().parse().map_err(Error::InvalidAddress)?;
            if address.is_public_key() {
                v.push(address);
            } else {
                return Err(Error::InvalidSignerPublicKey);
            }
        }
    }
    Ok(v)
}
