use bech32::{self, ToBase32, Variant, WriteBase32};
use sbor::rust::borrow::Cow;
use sbor::rust::fmt;
use sbor::rust::string::String;
use utils::combine;

use super::entity::EntityType;
use super::errors::AddressError;
use super::hrpset::HrpSet;
use crate::model::*;
use crate::node::NetworkDefinition;

/// Represents an encoder which understands how to encode Scrypto addresses in Bech32.
#[derive(Debug)]
pub struct Bech32Encoder {
    pub hrp_set: HrpSet,
}

impl Bech32Encoder {
    pub fn for_simulator() -> Self {
        Self::new(&NetworkDefinition::simulator())
    }

    /// Instantiates a new Bech32Encoder with the HRP corresponding to the passed network.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            hrp_set: network.into(),
        }
    }

    /// Encodes a package address in Bech32 and returns a String or panics on failure.
    pub fn encode_package_address_to_string(&self, package_address: &PackageAddress) -> String {
        let mut buf = String::new();
        self.encode_package_address_to_fmt(&mut buf, package_address)
            .expect("Failed to encode package address as Bech32");
        buf
    }

    /// Encodes a package address in Bech32 to the given fmt, or returns an `AddressError` on failure.
    pub fn encode_package_address_to_fmt<F: fmt::Write>(
        &self,
        fmt: &mut F,
        package_address: &PackageAddress,
    ) -> Result<(), AddressError> {
        match package_address {
            PackageAddress::Normal(data) => {
                self.encode_to_fmt(fmt, EntityType::package(package_address), data)
            }
        }
    }

    /// Encodes a system address in Bech32 and returns a String or panics on failure.
    pub fn encode_system_address_to_string(&self, system_address: &SystemAddress) -> String {
        let mut buf = String::new();
        self.encode_system_address_to_fmt(&mut buf, system_address)
            .expect("Failed to encode system address as Bech32");
        buf
    }

    /// Encodes a system address in Bech32 to the given fmt, or returns an `AddressError` on failure.
    pub fn encode_system_address_to_fmt<F: fmt::Write>(
        &self,
        fmt: &mut F,
        system_address: &SystemAddress,
    ) -> Result<(), AddressError> {
        let data = match system_address {
            SystemAddress::EpochManager(data) => data,
            SystemAddress::Clock(data) => data,
        };

        self.encode_to_fmt(fmt, EntityType::system(system_address), data)
    }

    /// Encodes a component address in Bech32 and returns a String or panics on failure.
    pub fn encode_component_address_to_string(
        &self,
        component_address: &ComponentAddress,
    ) -> String {
        let mut buf = String::new();
        self.encode_component_address_to_fmt(&mut buf, component_address)
            .expect("Failed to encode component address as Bech32");
        buf
    }

    /// Encodes a component address in Bech32 to the given fmt, or returns an `AddressError` on failure.
    pub fn encode_component_address_to_fmt<F: fmt::Write>(
        &self,
        fmt: &mut F,
        component_address: &ComponentAddress,
    ) -> Result<(), AddressError> {
        match component_address {
            ComponentAddress::Normal(data)
            | ComponentAddress::Account(data)
            | ComponentAddress::EcdsaSecp256k1VirtualAccount(data)
            | ComponentAddress::EddsaEd25519VirtualAccount(data) => {
                self.encode_to_fmt(fmt, EntityType::component(component_address), data)
            }
        }
    }

    /// Encodes a resource address in Bech32 and returns a String or panics on failure
    pub fn encode_resource_address_to_string(&self, resource_address: &ResourceAddress) -> String {
        let mut buf = String::new();
        self.encode_resource_address_to_fmt(&mut buf, resource_address)
            .expect("Failed to encode resource address as Bech32");
        buf
    }

    /// Encodes a resource address in Bech32 to the given fmt, or returns an `AddressError` on failure.
    pub fn encode_resource_address_to_fmt<F: fmt::Write>(
        &self,
        fmt: &mut F,
        resource_address: &ResourceAddress,
    ) -> Result<(), AddressError> {
        match resource_address {
            ResourceAddress::Normal(data) => {
                self.encode_to_fmt(fmt, EntityType::resource(resource_address), data)
            }
        }
    }

    /// Low level method which performs the Bech32 encoding of the data.
    fn encode_to_fmt<F: fmt::Write>(
        &self,
        fmt: &mut F,
        entity_type: EntityType,
        other_data: &[u8],
    ) -> Result<(), AddressError> {
        // Obtain the HRP corresponding to this entity type
        let hrp = self.hrp_set.get_entity_hrp(&entity_type);

        let full_data = combine(entity_type.id(), other_data);

        match bech32_encode_to_fmt(fmt, hrp, full_data.to_base32(), Variant::Bech32m) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(format_error)) => Err(AddressError::FormatError(format_error)),
            Err(encoding_error) => Err(AddressError::Bech32mEncodingError(encoding_error)),
        }
    }
}

/**
 * NOTE:
 * The below code is copied with minor alterations from the bech32 crate.
 * These alterations are to avoid using std for allocations, and fit with the sbor no-alloc options.
 *
 * The original source for the bech32 crate is under MIT license: https://crates.io/crates/bech32
 * This license permits modification without restriction, but requires the license copying below.
 *
 * Important additional note - the use of this modified code is also covered under the Radix license,
 * as per all code in this repository.
 *
 * -----------------
 *
 * MIT License
 *
 * Copyright (c) [year] [fullname]
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

/// Encode a bech32 payload to an [fmt::Write].
/// This method is intended for implementing traits from [std::fmt].
/// This method uses the std::fmt traits
///
/// # Errors
/// * If [check_hrp] returns an error for the given HRP.
/// # Deviations from standard
/// * No length limits are enforced for the data part
pub fn bech32_encode_to_fmt<F: fmt::Write, T: AsRef<[bech32::u5]>>(
    fmt: &mut F,
    hrp: &str,
    data: T,
    variant: Variant,
) -> Result<fmt::Result, bech32::Error> {
    let hrp_lower = match bech32_check_hrp(hrp)? {
        Bech32Case::Upper => Cow::Owned(hrp.to_lowercase()),
        Bech32Case::Lower | Bech32Case::None => Cow::Borrowed(hrp),
    };

    match bech32::Bech32Writer::new(&hrp_lower, variant, fmt) {
        Ok(mut writer) => {
            Ok(writer.write(data.as_ref()).and_then(|_| {
                // Finalize manually to avoid panic on drop if write fails
                writer.finalize()
            }))
        }
        Err(e) => Ok(Err(e)),
    }
}

/// Check if the HRP is valid. Returns the case of the HRP, if any.
///
/// # Errors
/// * **MixedCase**: If the HRP contains both uppercase and lowercase characters.
/// * **InvalidChar**: If the HRP contains any non-ASCII characters (outside 33..=126).
/// * **InvalidLength**: If the HRP is outside 1..83 characters long.
fn bech32_check_hrp(hrp: &str) -> Result<Bech32Case, bech32::Error> {
    if hrp.is_empty() || hrp.len() > 83 {
        return Err(bech32::Error::InvalidLength);
    }

    let mut has_lower: bool = false;
    let mut has_upper: bool = false;
    for b in hrp.bytes() {
        // Valid subset of ASCII
        if !(33..=126).contains(&b) {
            return Err(bech32::Error::InvalidChar(b as char));
        }

        if (b'a'..=b'z').contains(&b) {
            has_lower = true;
        } else if (b'A'..=b'Z').contains(&b) {
            has_upper = true;
        };

        if has_lower && has_upper {
            return Err(bech32::Error::MixedCase);
        }
    }

    Ok(match (has_upper, has_lower) {
        (true, false) => Bech32Case::Upper,
        (false, true) => Bech32Case::Lower,
        (false, false) => Bech32Case::None,
        (true, true) => unreachable!(),
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bech32Case {
    Upper,
    Lower,
    None,
}
