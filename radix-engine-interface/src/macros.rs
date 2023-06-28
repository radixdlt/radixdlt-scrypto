// TODO: Move this logic into procedural macro.
#[macro_export]
macro_rules! access_and_or {
    (|| $tt:tt) => {{
        let next = $crate::access_rule_node!($tt);
        move |e: AccessRuleNode| e.or(next)
    }};
    (|| $right1:ident $right2:tt) => {{
        let next = $crate::access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| e.or(next)
    }};
    (|| $right:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::access_rule_node!($right);
        move |e: AccessRuleNode| e.or(f(next))
    }};
    (|| $right:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::access_rule_node!($right);
        move |e: AccessRuleNode| f(e.or(next))
    }};
    (|| $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| e.or(f(next))
    }};
    (|| $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| f(e.or(next))
    }};

    (&& $tt:tt) => {{
        let next = $crate::access_rule_node!($tt);
        move |e: AccessRuleNode| e.and(next)
    }};
    (&& $right1:ident $right2:tt) => {{
        let next = $crate::access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| e.and(next)
    }};
    (&& $right:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::access_rule_node!($right);
        move |e: AccessRuleNode| f(e.and(next))
    }};
    (&& $right:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::access_rule_node!($right);
        move |e: AccessRuleNode| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = $crate::access_and_or!(&& $($rest)+);
        let next = $crate::access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = $crate::access_and_or!(|| $($rest)+);
        let next = $crate::access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| f(e.and(next))
    }};
}

#[macro_export]
macro_rules! access_rule_node {
    // Handle leaves
    ($rule:ident $args:tt) => {{
        $rule $args
    }};

    // Handle group
    (($($tt:tt)+)) => {{ $crate::access_rule_node!($($tt)+) }};

    // Handle and/or logic
    ($left1:ident $left2:tt $($right:tt)+) => {{
        let f = $crate::access_and_or!($($right)+);
        f($crate::access_rule_node!($left1 $left2))
    }};
    ($left:tt $($right:tt)+) => {{
        let f = $crate::access_and_or!($($right)+);
        f($crate::access_rule_node!($left))
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
        $crate::blueprints::resource::AccessRule::Protected($crate::access_rule_node!($($tt)+))
    }};
}

#[macro_export]
macro_rules! role_entry {
    ($roles: expr, $role: expr, $rule:expr) => {{
        $roles.define_immutable_role($role, $rule);
    }};
    ($roles: expr, $role: expr, $rule:expr, updatable) => {{
        $roles.define_mutable_role($role, $rule);
    }};
}

#[macro_export]
macro_rules! roles2 {
    ( ) => ({
        $crate::blueprints::resource::RolesInit::new()
    });
    ( $($role:expr => $rule:expr $(, $updatable:ident)?;)* ) => ({
        let mut roles = $crate::blueprints::resource::RolesInit::new();
        $(
            role_entry!(roles, $role, $rule $(, $updatable)? );
        )*
        roles
    })
}
