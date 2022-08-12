use super::*;
use core::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use crate::scrypto::{forward_ref_binop, forward_ref_op_assign};
use paste::paste;
macro_rules! types {

    ($($t:ident),*) => {
        $(
            {
                type: $t:ident,
                impl Ord for $t {
                    fn cmp(&self, other: &Self) -> Ordering {
                        let mut a: Vec<u8> = self.to_le_bytes().into();
                        let mut b: Vec<u8> = other.to_le_bytes().into();
                        a.reverse();
                        b.reverse();
                        if Self::MIN != Zero::zero() {
                            a[0] ^= 0x80;
                            b[0] ^= 0x80;
                        }
                        a.cmp(&b)
                    }
                }

                impl PartialOrd for $t {
                    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                        Some(self.cmp(other))
                    }
                }

                impl PartialEq for $t {
                    fn eq(&self, other: &Self) -> bool {
                        self.0 == other.0
                    }
                }
            }
        )*
    };
}
