// TODO: Move this logic into procedural macro.
#[macro_export]
macro_rules! access_and_or {
    (|| $tt:tt) => {{
        let next = $crate::composite_requirement!($tt);
        move |e: $crate::blueprints::resource::CompositeRequirement| e.or(next)
    }};
    (|| $right1:ident $right2:tt) => {{
        let next = $crate::composite_requirement!($right1 $right2);
        move |e: $crate::blueprints::resource::CompositeRequirement| e.or(next)
    }};
    (|| $right:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::composite_requirement!($right);
        move |e: $crate::blueprints::resource::CompositeRequirement| e.or(f(next))
    }};
    (|| $right:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::composite_requirement!($right);
        move |e: $crate::blueprints::resource::CompositeRequirement| f(e.or(next))
    }};
    (|| $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::composite_requirement!($right1 $right2);
        move |e: $crate::blueprints::resource::CompositeRequirement| e.or(f(next))
    }};
    (|| $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::composite_requirement!($right1 $right2);
        move |e: $crate::blueprints::resource::CompositeRequirement| f(e.or(next))
    }};

    (&& $tt:tt) => {{
        let next = $crate::composite_requirement!($tt);
        move |e: $crate::blueprints::resource::CompositeRequirement| e.and(next)
    }};
    (&& $right1:ident $right2:tt) => {{
        let next = $crate::composite_requirement!($right1 $right2);
        move |e: $crate::blueprints::resource::CompositeRequirement| e.and(next)
    }};
    (&& $right:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::composite_requirement!($right);
        move |e: $crate::blueprints::resource::CompositeRequirement| f(e.and(next))
    }};
    (&& $right:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::composite_requirement!($right);
        move |e: $crate::blueprints::resource::CompositeRequirement| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::composite_requirement!($right1 $right2);
        move |e: $crate::blueprints::resource::CompositeRequirement| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::composite_requirement!($right1 $right2);
        move |e: $crate::blueprints::resource::CompositeRequirement| f(e.and(next))
    }};
}

#[macro_export]
macro_rules! composite_requirement {
    // Handle leaves
    ($rule:ident $args:tt) => {{
        $rule $args
    }};

    // Handle group
    (($($tt:tt)+)) => {{ $crate::composite_requirement!($($tt)+) }};

    // Handle and/or logic
    ($left1:ident $left2:tt $($right:tt)+) => {{
        let f = $crate::access_and_or!($($right)+);
        f($crate::composite_requirement!($left1 $left2))
    }};
    ($left:tt $($right:tt)+) => {{
        let f = $crate::access_and_or!($($right)+);
        f($crate::composite_requirement!($left))
    }};
}

#[macro_export]
macro_rules! rule {
    (allow_all) => {{
        $crate::blueprints::resource::AccessRule::AllowAll
    }};
    (deny_all) => {{
        $crate::blueprints::resource::AccessRule::DenyAll
    }};
    ($($tt:tt)+) => {{
        $crate::blueprints::resource::AccessRule::Protected($crate::composite_requirement!($($tt)+))
    }};
}

#[macro_export]
macro_rules! role_entry {
    ($roles: expr, $role: expr, $rule:expr) => {{
        $roles.define_role($role, $rule);
    }};
}

#[macro_export]
macro_rules! roles2 {
    ( ) => ({
        $crate::blueprints::resource::RoleAssignmentInit::new()
    });
    ( $($role:expr => $rule:expr $(, $updatable:ident)?;)* ) => ({
        let mut roles = $crate::blueprints::resource::RoleAssignmentInit::new();
        $(
            role_entry!(roles, $role, $rule);
        )*
        roles
    })
}

extern crate radix_common_derive;
pub use radix_common_derive::dec;
pub use radix_common_derive::pdec;
