use crate::api::types::*;
use crate::data::scrypto_decode;
use crate::model::*;
use crate::*;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum PackageIdentifier {
    Scrypto(PackageAddress),
    Native(NativePackage),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum NativePackage {
    Auth,
    Component,
    Package,
    Metadata,
    EpochManager,
    Resource,
    Clock,
    Logger,
    TransactionRuntime,
    TransactionProcessor,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum FnIdentifier {
    Scrypto(ScryptoFnIdentifier),
    Native(NativeFn),
}

impl From<NativeFn> for FnIdentifier {
    fn from(native_fn: NativeFn) -> Self {
        FnIdentifier::Native(native_fn)
    }
}

impl FnIdentifier {
    pub fn is_scrypto_or_transaction(&self) -> bool {
        matches!(
            self,
            FnIdentifier::Scrypto(..)
                | FnIdentifier::Native(NativeFn::TransactionProcessor(TransactionProcessorFn::Run))
        )
    }

    pub fn package_identifier(&self) -> PackageIdentifier {
        match self {
            FnIdentifier::Scrypto(identifier) => {
                PackageIdentifier::Scrypto(identifier.package_address)
            }
            FnIdentifier::Native(identifier) => PackageIdentifier::Native(identifier.package()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ScryptoFnIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub ident: String,
}

impl ScryptoFnIdentifier {
    pub fn new(package_address: PackageAddress, blueprint_name: String, ident: String) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident,
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        &self.package_address
    }

    pub fn blueprint_name(&self) -> &String {
        &self.blueprint_name
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
pub enum NativeFn {
    AccessRulesChain(AccessRulesChainFn),
    Component(ComponentFn), // TODO: investigate whether to make royalty universal and take any "receiver".
    Package(PackageFn),
    Metadata(MetadataFn),
    EpochManager(EpochManagerFn),
    Validator(ValidatorFn),
    AuthZoneStack(AuthZoneStackFn),
    ResourceManager(ResourceManagerFn),
    Bucket(BucketFn),
    Vault(VaultFn),
    Proof(ProofFn),
    Worktop(WorktopFn),
    Clock(ClockFn),
    Logger(LoggerFn),
    TransactionRuntime(TransactionRuntimeFn),
    TransactionProcessor(TransactionProcessorFn),
}

impl NativeFn {
    pub fn package(&self) -> NativePackage {
        match self {
            NativeFn::AccessRulesChain(..) | NativeFn::AuthZoneStack(..) => NativePackage::Auth,
            NativeFn::Component(..) => NativePackage::Component,
            NativeFn::Package(..) => NativePackage::Package,
            NativeFn::Metadata(..) => NativePackage::Metadata,
            NativeFn::EpochManager(..) | NativeFn::Validator(..) => NativePackage::EpochManager,
            NativeFn::ResourceManager(..)
            | NativeFn::Bucket(..)
            | NativeFn::Vault(..)
            | NativeFn::Proof(..)
            | NativeFn::Worktop(..) => NativePackage::Resource,
            NativeFn::Clock(..) => NativePackage::Clock,
            NativeFn::Logger(..) => NativePackage::Logger,
            NativeFn::TransactionRuntime(..) => NativePackage::TransactionRuntime,
            NativeFn::TransactionProcessor(..) => NativePackage::TransactionProcessor,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum AccessRulesChainFn {
    AddAccessCheck,
    SetMethodAccessRule,
    SetGroupAccessRule,
    SetMethodMutability,
    SetGroupMutability,
    GetLength,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum MetadataFn {
    Set,
    Get,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentFn {
    SetRoyaltyConfig,
    ClaimRoyalty,
    Globalize,
    GlobalizeWithOwner,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum PackageFn {
    Publish,
    SetRoyaltyConfig,
    ClaimRoyalty,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum EpochManagerFn {
    Create,
    GetCurrentEpoch,
    NextRound,
    SetEpoch,
    CreateValidator,
    UpdateValidator,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum ValidatorFn {
    Register,
    Unregister,
    Stake,
    Unstake,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ResolveError {
    DecodeError(DecodeError),
    NotAMethod,
}

pub struct EpochManagerPackage;

impl EpochManagerPackage {
    pub fn resolve_method_invocation(
        receiver: ComponentAddress,
        method_name: &str,
        args: &[u8],
    ) -> Result<NativeInvocation, ResolveError> {
        let invocation = match receiver {
            ComponentAddress::EpochManager(..) => {
                let epoch_manager_fn =
                    EpochManagerFn::from_str(method_name).map_err(|_| ResolveError::NotAMethod)?;

                match epoch_manager_fn {
                    EpochManagerFn::Create => {
                        return Err(ResolveError::NotAMethod);
                    }
                    EpochManagerFn::GetCurrentEpoch => {
                        let _args: EpochManagerGetCurrentEpochMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::EpochManager(EpochManagerInvocation::GetCurrentEpoch(
                            EpochManagerGetCurrentEpochInvocation { receiver },
                        ))
                    }
                    EpochManagerFn::NextRound => {
                        let args: EpochManagerNextRoundMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::EpochManager(EpochManagerInvocation::NextRound(
                            EpochManagerNextRoundInvocation {
                                receiver,
                                round: args.round,
                            },
                        ))
                    }
                    EpochManagerFn::SetEpoch => {
                        let args: EpochManagerSetEpochMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::EpochManager(EpochManagerInvocation::SetEpoch(
                            EpochManagerSetEpochInvocation {
                                receiver,
                                epoch: args.epoch,
                            },
                        ))
                    }
                    EpochManagerFn::CreateValidator => {
                        let args: EpochManagerCreateValidatorMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::EpochManager(EpochManagerInvocation::CreateValidator(
                            EpochManagerCreateValidatorInvocation {
                                receiver,
                                key: args.validator,
                            },
                        ))
                    }
                    EpochManagerFn::UpdateValidator => {
                        let args: EpochManagerUpdateValidatorMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::EpochManager(EpochManagerInvocation::UpdateValidator(
                            EpochManagerUpdateValidatorInvocation {
                                receiver,
                                validator_address: args.validator_address,
                                update: args.update,
                            },
                        ))
                    }
                }
            }
            ComponentAddress::Validator(..) => {
                let validator_fn =
                    ValidatorFn::from_str(method_name).map_err(|_| ResolveError::NotAMethod)?;

                match validator_fn {
                    ValidatorFn::Register => {
                        let _args: ValidatorRegisterMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::Validator(ValidatorInvocation::Register(
                            ValidatorRegisterInvocation { receiver },
                        ))
                    }
                    ValidatorFn::Unregister => {
                        let _args: ValidatorUnregisterValidatorMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::Validator(ValidatorInvocation::Unregister(
                            ValidatorUnregisterInvocation { receiver },
                        ))
                    }

                    ValidatorFn::Stake => {
                        let args: ValidatorStakeMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::Validator(ValidatorInvocation::Stake(
                            ValidatorStakeInvocation { receiver, stake: args.stake },
                        ))
                    }

                    ValidatorFn::Unstake => {
                        let args: ValidatorUnstakeMethodArgs =
                            scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                        NativeInvocation::Validator(ValidatorInvocation::Unstake(
                            ValidatorUnstakeInvocation { receiver, amount: args.amount },
                        ))
                    }
                }
            }
            _ => return Err(ResolveError::NotAMethod),
        };

        Ok(invocation)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum AuthZoneStackFn {
    Pop,
    Push,
    CreateProof,
    CreateProofByAmount,
    CreateProofByIds,
    Clear,
    Drain,
    AssertAccessRule,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum ResourceManagerFn {
    CreateNonFungible,
    CreateFungible,
    CreateNonFungibleWithInitialSupply,
    CreateUuidNonFungibleWithInitialSupply,
    CreateFungibleWithInitialSupply,
    MintNonFungible,
    MintUuidNonFungible,
    MintFungible,
    Burn,
    UpdateVaultAuth,
    LockAuth,
    UpdateNonFungibleData,
    GetNonFungible,
    GetResourceType,
    GetTotalSupply,
    NonFungibleExists,
    CreateBucket,
    CreateVault,
    BurnBucket,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum BucketFn {
    Take,
    TakeNonFungibles,
    Put,
    GetNonFungibleIds,
    GetAmount,
    GetResourceAddress,
    CreateProof,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum VaultFn {
    Take,
    LockFee,
    Put,
    TakeNonFungibles,
    GetAmount,
    GetResourceAddress,
    GetNonFungibleIds,
    CreateProof,
    CreateProofByAmount,
    CreateProofByIds,
    Recall,
    RecallNonFungibles,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum ProofFn {
    Clone,
    GetAmount,
    GetNonFungibleIds,
    GetResourceAddress,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum WorktopFn {
    TakeAll,
    TakeAmount,
    TakeNonFungibles,
    Put,
    AssertContains,
    AssertContainsAmount,
    AssertContainsNonFungibles,
    Drain,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum ClockFn {
    Create,
    SetCurrentTime,
    GetCurrentTime,
    CompareCurrentTime,
}

pub struct ClockPackage;

impl ClockPackage {
    pub fn resolve_method_invocation(
        receiver: ComponentAddress,
        method_name: &str,
        args: &[u8],
    ) -> Result<ClockInvocation, ResolveError> {
        let clock_fn = ClockFn::from_str(method_name).map_err(|_| ResolveError::NotAMethod)?;
        let invocation = match clock_fn {
            ClockFn::Create => {
                return Err(ResolveError::NotAMethod);
            }
            ClockFn::CompareCurrentTime => {
                let args: ClockCompareCurrentTimeMethodArgs =
                    scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                ClockInvocation::CompareCurrentTime(ClockCompareCurrentTimeInvocation {
                    receiver,
                    instant: args.instant,
                    precision: args.precision,
                    operator: args.operator,
                })
            }
            ClockFn::GetCurrentTime => {
                let args: ClockGetCurrentTimeMethodArgs =
                    scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                ClockInvocation::GetCurrentTime(ClockGetCurrentTimeInvocation {
                    receiver,
                    precision: args.precision,
                })
            }
            ClockFn::SetCurrentTime => {
                let args: ClockSetCurrentTimeMethodArgs =
                    scrypto_decode(args).map_err(ResolveError::DecodeError)?;
                ClockInvocation::SetCurrentTime(ClockSetCurrentTimeInvocation {
                    receiver,
                    current_time_ms: args.current_time_ms,
                })
            }
        };

        Ok(invocation)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum LoggerFn {
    Log,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionRuntimeFn {
    Get,
    GenerateUuid,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionProcessorFn {
    Run,
}
