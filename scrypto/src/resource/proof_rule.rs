#[macro_export]
macro_rules! resource_list {
  ($($resource: expr),*) => ({
      let mut list: Vec<::scrypto::resource::SoftResourceOrNonFungible> = Vec::new();
      $(
        list.push($resource.into());
      )*
      ::scrypto::resource::SoftResourceOrNonFungibleList::Static(list)
  });
}

// TODO: Move this logic into preprocessor. It probably needs to be implemented as a procedural macro.
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
    ($rule:ident $args:tt) => {{ radix_engine_interface::model::AccessRuleNode::ProofRule($rule $args) }};

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
        radix_engine_interface::model::AccessRule::AllowAll
    }};
    (deny_all) => {{
        radix_engine_interface::model::AccessRule::DenyAll
    }};
    ($($tt:tt)+) => {{
        radix_engine_interface::model::AccessRule::Protected($crate::access_rule_node!($($tt)+))
    }};
}
