use crate::kernel::*;
use crate::types::*;

pub trait BucketRef {
    fn amount(&self) -> U256;

    fn resource(&self) -> Address;

    fn destroy(self);
}

impl BucketRef for RID {
    fn amount(&self) -> U256 {
        let input = GetAmountRefInput { reference: *self };
        let output: GetAmountRefOutput = call_kernel(GET_AMOUNT_REF, input);

        output.amount
    }

    fn resource(&self) -> Address {
        let input = GetResourceRefInput { reference: *self };
        let output: GetResourceRefOutput = call_kernel(GET_RESOURCE_REF, input);

        output.resource
    }

    fn destroy(self) {
        let input = DropReferenceInput { reference: self };
        let _: DropReferenceOutput = call_kernel(DROP_REFERENCE, input);
    }
}
