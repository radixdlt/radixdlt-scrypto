use lazy_static::lazy_static;
use sbor::rust::collections::HashMap;

macro_rules! known_enum {
    ($map: expr, enum $name:ident { $($variant:ident = $id:expr;)* }) => {
        $(
            $map.insert(concat!(stringify!($name), "::", stringify!($variant)), $id);
        )*
    };
}

// TODO: we need a final sanity check before mainnet launch!

lazy_static! {
    pub static ref KNOWN_ENUM_DISCRIMINATORS: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();

        // Protocol Buffer syntax

        known_enum!(
            m,
            enum Option {
                None = 0;
                Some = 1;
            }
        );

        known_enum!(
            m,
            enum Result {
                Ok = 0;
                Err = 1;
            }
        );

        known_enum!(
            m,
            enum AccessRuleEntry {
                AccessRule = 0;
                Group = 1;
            }
        );

        known_enum!(
            m,
            enum AccessRule {
                AllowAll = 0;
                DenyAll = 1;
                Protected = 2;
            }
        );

        known_enum!(
            m,
            enum AccessRuleNode {
                ProofRule = 0;
                AnyOf = 1;
                AllOf = 2;
            }
        );

        known_enum!(
            m,
            enum ProofRule {
                Require = 0;
                AmountOf = 1;
                CountOf = 2;
                AllOf = 3;
                AnyOf = 4;
            }
        );

        known_enum!(
            m,
            enum SoftResourceOrNonFungible {
                StaticNonFungible = 0;
                StaticResource = 1;
                Dynamic = 2;
            }
        );

        known_enum!(
            m,
            enum AccessRuleKey {
                Scrypto = 0;
                Native = 1;
            }
        );

        known_enum!(
            m,
            enum NativeFn {
                AccessRulesChain = 0;
                Component = 1;
                Package = 2;
                Metadata = 3;
                EpochManager = 4;
                AuthZoneStack = 5;
                ResourceManager = 6;
                Bucket = 7;
                Vault = 8;
                Proof = 9;
                Worktop = 10;
                Clock = 11;
                Logger = 12;
                TransactionRuntime = 13;
                TransactionProcessor = 14;
            }
        );

        known_enum!(
            m,
            enum PackageFn {
                Publish = 0;
                SetRoyaltyConfig = 1;
                ClaimRoyalty = 2;
            }
        );

        known_enum!(
            m,
            enum MetadataFn {
                Set = 0;
                Get = 1;
            }
        );

        known_enum!(
            m,
            enum ResourceMethodAuthKey {
                Mint = 0;
                Burn = 1;
                UpdateNonFungibleData = 2;
                UpdateMetadata = 3;
                Withdraw = 4;
                Deposit = 5;
                Recall = 6;
            }
        );

        known_enum!(
            m,
            enum NonFungibleIdType {
                String = 0;
                Integer = 1;
                Bytes = 2;
                UUID = 3;
            }
        );

        m
    };
}
