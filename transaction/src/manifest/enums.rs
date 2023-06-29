use lazy_static::lazy_static;
use sbor::rust::collections::NonIterMap;

macro_rules! known_enum {
    ($map: expr, enum $name:ident { $($variant:ident = $id:expr;)* }) => {
        $(
            $map.insert(concat!(stringify!($name), "::", stringify!($variant)), $id);
        )*
    };
}

// FIXME: we need a final sanity check before mainnet launch!
lazy_static! {
    pub static ref KNOWN_ENUM_DISCRIMINATORS: NonIterMap<&'static str, u8> = {
        let mut m = NonIterMap::new();

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
            enum Metadata {
                String = 0;
                Bool = 1;
                U8 = 2;
                U32 = 3;
                U64 = 4;
                I32 = 5;
                I64 = 6;
                Decimal = 7;
                Address = 8;
                PublicKey = 9;
                NonFungibleGlobalId = 10;
                NonFungibleLocalId = 11;
                Instant = 12;
                Url = 13;
                Origin = 14;
                PublicKeyHash = 15;

                StringArray = 128;
                BoolArray = 129;
                U8Array = 130;
                U32Array = 131;
                U64Array = 132;
                I32Array = 133;
                I64Array = 134;
                DecimalArray = 135;
                AddressArray = 136;
                PublicKeyArray = 137;
                NonFungibleGlobalIdArray = 138;
                NonFungibleLocalIdArray = 139;
                InstantArray = 140;
                UrlArray = 141;
                OriginArray = 142;
                PublicKeyHashArray = 143;
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
            enum ModuleId {
                Main = 0;
                Metadata = 1;
                Royalty = 2;
                AccessRules = 3;
            }
        );

        known_enum!(
            m,
            enum ResourceAction {
                Mint = 0;
                Burn = 1;
                UpdateNonFungibleData = 2;
                Withdraw = 3;
                Deposit = 4;
                Recall = 5;
            }
        );

        known_enum!(
            m,
            enum NonFungibleIdType {
                String = 0;
                Integer = 1;
                Bytes = 2;
                RUID = 3;
            }
        );

        known_enum!(
            m,
            enum AccountDefaultDepositRule {
                Accept = 0;
                Reject = 1;
                AllowExisting = 2;
            }
        );

        known_enum!(
            m,
            enum ResourceDepositRule {
                Neither = 0;
                Allowed = 1;
                Disallowed = 2;
            }
        );

        known_enum!(
            m,
            enum PublicKey {
                Secp256k1 = 0;
                Ed25519 = 1;
            }
        );

        known_enum!(
            m,
            enum PublicKeyHash {
                Secp256k1 = 0;
                Ed25519 = 1;
            }
        );

        known_enum!(
            m,
            enum RoyaltyAmount {
                Free = 0;
                Xrd = 1;
                Usd = 2;
            }
        );

        known_enum!(
            m,
            enum OwnerRole {
                None = 0;
                Fixed = 1;
                Updatable = 2;
            }
        );

        m
    };
}
