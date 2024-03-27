use crate::internal_prelude::*;
use core::future::*;
use core::ops::DerefMut;
use core::pin::*;
use core::task::*;

pub struct NativeVmInstance {
}

impl NativeVmInstance {
    pub fn invoke(
        &mut self,
        input: u32,
        api: &mut DummyApi,
    ) -> Result<u32, RuntimeError>
    {
        // We basically implement a custom executor
        // https://rust-lang.github.io/async-book/02_execution/04_executor.html

        // It may make even more sense for the executor to live in the kernel call frame

        // Note - In this scenario, we store the Future on the stack
        // But if it gets too big we may wish to store it with Box::pin
        // See https://fasterthanli.me/articles/pin-and-suffering
        let runtime = NativeRuntime::new();
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut pollable_invocation = pin!(Self::start_invocation(input, &runtime));
        loop {
            // TODO: Wrap this in a panic catcher
            let continuation_outcome = pollable_invocation.as_mut().poll(&mut cx);
            // Then the following sys calls are performed, which can throw
            match continuation_outcome {
                Poll::Pending => {
                    runtime.perform_pending_action(api)?;
                },
                Poll::Ready(result) => return Ok(result),
            }
        }
    }

    // Could be in a NativeVmRuntime
    async fn start_invocation(
        input: u32,
        runtime: &NativeRuntime,
    ) -> u32
    {
        DummyPackage::start(input, runtime).await
    }
}

// Copied from Waker::noop(), except that is currently unstable
pub fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Cloning just returns a new no-op raw waker
        |_| RAW,
        // `wake` does nothing
        |_| {},
        // `wake_by_ref` does nothing
        |_| {},
        // Dropping does nothing as we don't allocate anything
        |_| {},
    );
    const RAW: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);

    unsafe { Waker::from_raw(RAW) }
}

pub struct DummyApi;

impl DummyApi {
    fn add_one(&mut self, x: u32) -> Result<u32, RuntimeError> {
        Ok(x + 1)
    }
}

trait DummyApiRequest: 'static {
    type Response: 'static;

    fn perform(self, api: &mut DummyApi) -> Result<Self::Response, RuntimeError>;
}

struct DummyApiReq1<F: Fn(&mut DummyApi, Input) -> Result<Output, RuntimeError>, Input: 'static, Output: 'static>(F, Input);

impl <F: Fn(&mut DummyApi, Input) -> Result<Output, RuntimeError> + 'static, Input: 'static, Output: 'static> DummyApiReq1<F, Input, Output> {
    pub fn new(func: F, input: Input) -> Self {
        Self(func, input)
    }
}

impl <F: Fn(&mut DummyApi, Input) -> Result<Output, RuntimeError> + 'static, Input: 'static, Output: 'static> DummyApiRequest for DummyApiReq1<F, Input, Output> {
    type Response = Output;

    fn perform(self, api: &mut DummyApi) -> Result<Self::Response, RuntimeError> {
        self.0(api, self.1)
    }
}

struct NativeRuntime {
    pending_action: RefCell<Option<PendingAction>>,
}

enum PendingAction {
    DummyApiRequest(Rc<RefCell<dyn DummyApiRequestPerformer>>),
}

impl NativeRuntime {
    fn new() -> Self {
        Self {
            pending_action: RefCell::new(None),
        }
    }

    fn register_pending_action(&self, pending_action: PendingAction) {
        // This is performed by the native blueprint, therefore allowed to panic
        match self.pending_action.borrow_mut().replace(pending_action) {
            None => {},
            _ => panic!("New pending action was set whilst existing pending action is outstanding"),
        }
    }

    fn perform_pending_action(&self, api: &mut DummyApi) -> Result<(), RuntimeError> {
        let action = self.pending_action.borrow_mut().take()
            // TODO - replace with some kind of RuntimeError, as this is an error of the native component, and needs to be handled without panicking
            .expect("A future returned control, but there is no registered pending action. Likely this was caused by use of a non-Radix future.");
        match action {
            PendingAction::DummyApiRequest(request) => request.borrow_mut().perform_action(api),
        }
    }

    pub async fn dummy_api_request<
        F: Fn(&mut DummyApi, Input) -> Result<Output, RuntimeError> + 'static,
        Input: 'static,
        Output: 'static,
    >(
        &self,
        func: F,
        input: Input,
    ) -> Output {
        DummyApiRequestFuture::new(self, DummyApiReq1::new(func, input)).await
    }
}

enum RequestState<R: DummyApiRequest> {
    Requested { request: R, already_polled: bool },
    Executing,
    Response(R::Response),
    Completed,
}

/// Purposefully suitable for dyn boxing
trait DummyApiRequestPerformer {
    fn perform_action(&mut self, api: &mut DummyApi) -> Result<(), RuntimeError>;
}

struct DummyApiRequestFuture<'r, R: DummyApiRequest> {
    runtime: &'r NativeRuntime,
    request_state: Rc<RefCell<RequestState::<R>>>,
}

impl<'r, R: DummyApiRequest> DummyApiRequestFuture<'r, R> {
    fn new(runtime: &'r NativeRuntime, request: R) -> Self {
        Self {
            runtime,
            request_state: Rc::new(RefCell::new(RequestState::Requested { request, already_polled: false })),
        }
    }
}

impl<R: DummyApiRequest> DummyApiRequestPerformer for RequestState<R> {
    fn perform_action(&mut self, api: &mut DummyApi) -> Result<(), RuntimeError> {
        let state = mem::replace(self, RequestState::Executing);
        let request = match state {
            RequestState::Requested { request, already_polled } if already_polled => request,
            _ => panic!(), // TODO - replace with a RuntimeError
        };
        let response = request.perform(api)?;
        *self = RequestState::Response(response);
        Ok(())
    }
}

impl<'r, R: DummyApiRequest + 'static> Future for DummyApiRequestFuture<'r, R> {
    type Output = R::Response;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Note - panics here are allowed and indicate a misbehaving native component
        match self.request_state.borrow_mut().deref_mut() {
            RequestState::Requested { already_polled, .. } => {
                if *already_polled {
                    panic!();
                }
                *already_polled = true;
                self.runtime.register_pending_action(PendingAction::DummyApiRequest(self.request_state.clone()));
                Poll::Pending
            },
            RequestState::Executing => panic!(),
            state @ RequestState::Response(_) => {
                let RequestState::Response(response) = mem::replace(state, RequestState::Completed) else {
                    unreachable!()
                };
                Poll::Ready(response)
            },
            RequestState::Completed => panic!(),
        }
    }
}

struct DummyPackage {

}

impl DummyPackage {
    pub async fn start(
        input: u32,
        runtime: &NativeRuntime,
    ) -> u32
    {
        runtime.dummy_api_request(DummyApi::add_one, input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attempt_one() {
        let mut instance = NativeVmInstance {};
        let mut api = DummyApi;
        let out = instance.invoke(31, &mut api);
        assert_eq!(out, Ok(32))
    }
}