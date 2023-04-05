//! This module implements a number of useful utility functions used to prepare instructions for
//! calling methods and functions when a known schema is provided. This module implements all of the
//! parsing logic as well as the logic needed ot add these instructions to the original manifest
//! builder that is being used.

use radix_engine::types::*;
use radix_engine_interface::math::ParseDecimalError;

#[derive(Debug)]
pub enum ParseResourceSpecifierError {
    InvalidAmount(ParseDecimalError),
    InvalidResourceAddress(String),
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
    MoreThanOneAmountSpecified,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceSpecifier {
    Amount(Decimal, ResourceAddress),
    Ids(BTreeSet<NonFungibleLocalId>, ResourceAddress),
}

impl FromStr for ResourceSpecifier {
    type Err = ParseResourceSpecifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_resource_specifier(s, &Bech32Decoder::for_simulator())
    }
}

/// Attempts to parse a string as a [`ResourceSpecifier`] object.
///
/// Given a string, this function attempts to parse that string as a fungible or non-fungible
/// [`ResourceSpecifier`]. When a resource address is encountered in the string, the passed bech32m
/// decoder is used to attempt to decode the address.
///
/// The format expected for the string representation of fungible and non-fungible resource
/// specifiers differs. The following elaborates on the formats and the parsing modes.
///
/// ## Fungible Resource Specifiers
///
/// The string representation of fungible resource addresses is that it is a [`Decimal`] amount
/// followed by a resource address for that given resource. Separating the amount and the resource
/// address is a comma. The following is what that looks like as well as an example of that
///
/// ```txt
/// <resource_address>:<amount>
/// ```
///
/// As an example, say that `resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0` is a
/// fungible resource which we wish to create a resource specifier of `12.91` of, then the string
/// format to use for the fungible specifier would be:
///
/// ```txt
/// resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0:12.91
/// ```
///
/// ## Non-Fungible Resource Specifiers
///
/// The string representation of non-fungible resource specifiers follows the same format which will
/// be used for the wallet, explorer, and other parts of our system. A string non-fungible resource
/// specifier beings with a Bech32m encoded resource address, then a colon, and then a list of comma
/// separated non-fungible ids that we wish to specify.
///
/// The type of these non-fungible ids does not need to be provided in the non-fungible resource
/// specifier string representation; this is because the type is automatically looked up for the
/// given resource address and then used as context for the parsing of the given non-fungible id
/// string.
///
/// The format of the string representation of non-fungible resource specifiers is:
///
/// ```txt
/// <resource_address>:<non_fungible_local_id_1>,<non_fungible_local_id_2>,...,<non_fungible_local_id_n>
/// ```
///
/// As an example, say that `resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0` is a
/// non-fungible resource which has a non-fungible id type of [`NonFungibleIdType::Integer`], say that
/// we wish to specify non-fungible tokens of this resource with the ids: 12, 900, 181, the string
/// representation of the non-fungible resource specifier would be:
///
/// ```txt
/// resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0:#12#,#900#,#181#
/// ```
///
/// As you can see from the example above, there was no need to specify the non-fungible id type in
/// the resource specifier string, as mentioned above, this is because this information can be
/// looked up from the simulator's substate store.
pub fn parse_resource_specifier(
    input: &str,
    bech32_decoder: &Bech32Decoder,
) -> Result<ResourceSpecifier, ParseResourceSpecifierError> {
    // Splitting up the input into two parts: the resource address and the non-fungible ids
    let tokens = input
        .trim()
        .split(':')
        .map(|s| s.trim())
        .collect::<Vec<_>>();

    // There MUST only be two tokens in the tokens vector, one for the resource address, and
    // another for the non-fungible ids. If there is more or less, then this function returns
    // an error.
    if tokens.len() != 2 {
        return Err(ParseResourceSpecifierError::MoreThanOneAmountSpecified);
    }

    // If the second part starts with a non-fungible local id prefix, we consider it as non-fungible resource specifier.
    let is_non_fungible = tokens[1].starts_with("<")
        || tokens[1].starts_with("#")
        || tokens[1].starts_with("[")
        || tokens[1].starts_with("{");

    if !is_non_fungible {
        let resource_address_string = tokens[0];
        let amount_string = tokens[1];

        let amount = amount_string
            .parse()
            .map_err(ParseResourceSpecifierError::InvalidAmount)?;
        let resource_address =
            ResourceAddress::try_from_bech32(&bech32_decoder, resource_address_string).ok_or(
                ParseResourceSpecifierError::InvalidResourceAddress(
                    resource_address_string.to_string(),
                ),
            )?;

        Ok(ResourceSpecifier::Amount(amount, resource_address))
    } else {
        // Paring the resource address fully first to use it for the non-fungible id type ledger
        // lookup
        let resource_address_string = tokens[0];
        let resource_address =
            ResourceAddress::try_from_bech32(&bech32_decoder, resource_address_string).ok_or(
                ParseResourceSpecifierError::InvalidResourceAddress(
                    resource_address_string.to_string(),
                ),
            )?;

        // Parsing the non-fungible ids with the available id type
        let non_fungible_local_ids = tokens[1]
            .split(',')
            .map(|s| NonFungibleLocalId::from_str(s.trim()))
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(ParseResourceSpecifierError::InvalidNonFungibleLocalId)?;

        Ok(ResourceSpecifier::Ids(
            non_fungible_local_ids,
            resource_address,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn parsing_of_fungible_resource_specifier_succeeds() {
        // Arrange
        let resource_specifier_string =
            "resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje:900";
        let bech32_decoder = Bech32Decoder::for_simulator();

        // Act
        let resource_specifier =
            parse_resource_specifier(resource_specifier_string, &bech32_decoder)
                .expect("Failed to parse resource specifier");

        // Assert
        assert_eq!(
            resource_specifier,
            ResourceSpecifier::Amount(
                900.into(),
                ResourceAddress::try_from_bech32(
                    &bech32_decoder,
                    "resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje"
                )
                .unwrap()
            )
        )
    }

    #[test]
    pub fn parsing_of_single_non_fungible_resource_specifier_succeeds() {
        // Arrange
        let resource_specifier_string =
            "resource_sim1qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq056vhf:[1f5db2614c1c4c626e9c279349b240af7cb939ead29058fdff2c]";
        let bech32_decoder = Bech32Decoder::for_simulator();

        // Act
        let resource_specifier =
            parse_resource_specifier(resource_specifier_string, &bech32_decoder)
                .expect("Failed to parse resource specifier");

        // Assert
        assert_eq!(
            resource_specifier,
            ResourceSpecifier::Ids(
                BTreeSet::from([NonFungibleLocalId::bytes(vec![
                    31, 93, 178, 97, 76, 28, 76, 98, 110, 156, 39, 147, 73, 178, 64, 175, 124, 185,
                    57, 234, 210, 144, 88, 253, 255, 44
                ])
                .unwrap()]),
                ECDSA_SECP256K1_TOKEN
            )
        )
    }

    #[test]

    pub fn parsing_of_multiple_non_fungible_resource_specifier_succeeds() {
        // Arrange
        let resource_specifier_string =
            "resource_sim1qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq056vhf:[1f5db2614c1c4c626e9c279349b240af7cb939ead29058fdff2c],[d85dc446d8e5eff48db25b56f6b5001d14627b5a199598485a8d],[005d1ae87b0e7c5401d38e58d43291ffbd9ba6e1da54f87504a7]";
        let bech32_decoder = Bech32Decoder::for_simulator();

        // Act
        let resource_specifier =
            parse_resource_specifier(resource_specifier_string, &bech32_decoder)
                .expect("Failed to parse resource specifier");

        // Assert
        assert_eq!(
            resource_specifier,
            ResourceSpecifier::Ids(
                BTreeSet::from([
                    NonFungibleLocalId::bytes(vec![
                        31, 93, 178, 97, 76, 28, 76, 98, 110, 156, 39, 147, 73, 178, 64, 175, 124,
                        185, 57, 234, 210, 144, 88, 253, 255, 44
                    ])
                    .unwrap(),
                    NonFungibleLocalId::bytes(vec![
                        216, 93, 196, 70, 216, 229, 239, 244, 141, 178, 91, 86, 246, 181, 0, 29,
                        20, 98, 123, 90, 25, 149, 152, 72, 90, 141
                    ])
                    .unwrap(),
                    NonFungibleLocalId::bytes(vec![
                        0, 93, 26, 232, 123, 14, 124, 84, 1, 211, 142, 88, 212, 50, 145, 255, 189,
                        155, 166, 225, 218, 84, 248, 117, 4, 167
                    ])
                    .unwrap()
                ]),
                ECDSA_SECP256K1_TOKEN
            )
        )
    }
}
