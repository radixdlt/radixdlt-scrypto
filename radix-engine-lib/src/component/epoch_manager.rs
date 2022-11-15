use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::misc::{copy_u8_array, ContextualDisplay};

use crate::abi::*;
use crate::address::*;
use crate::engine::{api::*, scrypto_env::*};

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerCreateInvocation {}

impl SysInvocation for EpochManagerCreateInvocation {
    type Output = SystemAddress;
}

impl ScryptoNativeInvocation for EpochManagerCreateInvocation {}

impl Into<NativeFnInvocation> for EpochManagerCreateInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::EpochManager(
            EpochManagerFunctionInvocation::Create(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerGetCurrentEpochInvocation {
    pub receiver: SystemAddress,
}

impl SysInvocation for EpochManagerGetCurrentEpochInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for EpochManagerGetCurrentEpochInvocation {}

impl Into<NativeFnInvocation> for EpochManagerGetCurrentEpochInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::GetCurrentEpoch(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerSetEpochInvocation {
    pub receiver: SystemAddress,
    pub epoch: u64,
}

impl SysInvocation for EpochManagerSetEpochInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for EpochManagerSetEpochInvocation {}

impl Into<NativeFnInvocation> for EpochManagerSetEpochInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::SetEpoch(self),
        ))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SystemAddress {
    EpochManager([u8; 26]),
}

//========
// binary
//========

impl TryFrom<&[u8]> for SystemAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::EpochManager => Ok(Self::EpochManager(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl SystemAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::system(self).id());
        match self {
            Self::EpochManager(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

scrypto_type!(
    SystemAddress,
    ScryptoCustomTypeId::SystemAddress,
    Type::SystemAddress,
    27
);

//======
// text
//======

impl fmt::Debug for SystemAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for SystemAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_system_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            SystemAddress::EpochManager(_) => {
                write!(f, "EpochManagerSystem[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
