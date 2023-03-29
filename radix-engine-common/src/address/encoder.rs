use super::hrpset::HrpSet;
use crate::address::errors::EncodeBech32AddressError;
use crate::network::NetworkDefinition;
use crate::types::EntityType;
use bech32::{self, ToBase32, Variant, WriteBase32};
use sbor::rust::prelude::*;

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

    pub fn encode(&self, full_data: &[u8]) -> Result<String, EncodeBech32AddressError> {
        let mut buf = String::new();
        self.encode_to_fmt(&mut buf, full_data)?;
        Ok(buf)
    }

    /// Low level method which performs the Bech32 encoding of the data.
    pub fn encode_to_fmt<F: fmt::Write>(
        &self,
        fmt: &mut F,
        full_data: &[u8],
    ) -> Result<(), EncodeBech32AddressError> {
        // Decode the entity type
        let entity_type = EntityType::from_repr(
            *full_data
                .get(0)
                .ok_or(EncodeBech32AddressError::MissingEntityTypeByte)?,
        )
        .ok_or_else(|| EncodeBech32AddressError::InvalidEntityTypeId(full_data[0]))?;

        // Obtain the HRP corresponding to this entity type
        let hrp = self.hrp_set.get_entity_hrp(&entity_type);

        match bech32_encode_to_fmt(fmt, hrp, full_data.to_base32(), Variant::Bech32m) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(format_error)) => Err(EncodeBech32AddressError::FormatError(format_error)),
            Err(encoding_error) => Err(EncodeBech32AddressError::Bech32mEncodingError(
                encoding_error,
            )),
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
