#[macro_export]
macro_rules! sys_env_native_fn {
    ($vis:vis $fn:ident $fn_name:ident ($($args:tt)+) -> $rtn:ty { $invocation:ident { $($invocation_args:tt)* } }) => {
        $vis $fn $fn_name<Y, E>($($args)*, env: &mut Y) -> Result<$rtn, E>
        where
            Y: radix_engine_interface::api::Invokable<$invocation, E>,
            E: sbor::rust::fmt::Debug + Categorize<radix_engine_interface::data::ScryptoCustomValueKind> + radix_engine_interface::data::ScryptoDecode,
        {
            radix_engine_interface::api::Invokable::invoke(env, $invocation { $($invocation_args)* })
        }
    };

    ($vis:vis $fn:ident $fn_name:ident () -> $rtn:ty { $invocation:ident { $($invocation_args:tt)* } }) => {
        $vis $fn $fn_name<Y, E>(env: &mut Y) -> Result<$rtn, E>
        where
            Y: radix_engine_interface::api::Invokable<$invocation, E>,
            E: sbor::rust::fmt::Debug + Categorize<radix_engine_interface::data::ScryptoCustomValueKind> + radix_engine_interface::data::ScryptoDecode,
        {
            radix_engine_interface::api::Invokable::invoke(env, $invocation { $($invocation_args)* })
        }
    };
}
