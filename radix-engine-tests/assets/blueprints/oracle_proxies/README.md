# Introduction

This folder includes Proxy blueprint examples for `Oracle` component (available in `oracle_v1`, `oracle_v2`
and `oracle_v3` blueprints).
It is assumed that `Oracle` component includes:
- public methods, eg. `get_price`
- protected methods, eg. `set_price`
Below examples take above into account.

Proxy blueprint examples:
- `oracle_proxy_with_global`
    - It assumes that signatures of `Oracle` component proxied methods will remain unchanged
    (thus not compliant with `oracle_v3`)
    - This proxy works with `Oracle` instantiated as a global component
    - It can call only public methods from `Oracle` component

- `oracle_proxy_with_owned`
    - It assumes that signatures of `Oracle` component proxied methods will remain unchanged
    (thus not compliant with `oracle_v3`)
    - It works with `Oracle`  instantiated as an owned component (it must be instantiated by this proxy)
    - It can call any method from owned `Oracle` component

- `oracle_generic_proxy_with_global`
    - It is a generic proxy which can call any method with any arguments from configured component
    - It works with component instantiated as a global component
    - It can call only public methods from configured component

NOTE!
There is no `oracle_generic_proxy_with_owned` example because proxy generic `call_method()` shall be public.
And since the proxy owns the component it can call any method of the owned component and this is not acceptable,
since some methods eg. `set_price` shall be protected.

# Cost overhead

Below table shows the cost overhead of a singe `get_price` method call via proxy.

| Proxy | Cost overhead |
| :--- | :--- |
| `oracle_proxy_with_global` | < 0.19 XRD |
| `oracle_proxy_with_owned` | < 0.19 XRD |
| `oracle_generic_proxy_with_global` | < 0.19 XRD |

NOTE!
The cost overhead highly depepends on the complexity of the arguments and return values of proxied methods,
which are decoded/encoded from/into ScryptoValue.
