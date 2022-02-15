# Flat Admin
This example demonstrates a blueprint which can be used by other blueprints and components to manage a set of admin badges for a single level of administration.

Instantiating a `FlatAdmin` component will create a badge manager component, as well as the first admin badge.  You can then use this badge for authorization into privileged methods on another component.  By interacting with the badge manager component, anyone possessing an admin badge can create an additional badge, which can be distributed as desired.

## Resources and Data
```rust
struct FlatAdmin {
    admin_mint_badge: Vault,
    admin_badge: ResourceDef,
}
```

In order to be able to mint additional admin badges after our first, we'll need a vault to contain a badge which holds that minting permission.

For user convenience, we'll also maintain the `ResourceDef` of the external admin badge that we'll be handing out, so that they can interrogate an instantiated `FlatAdmin` component about which badge it manages.

## Getting Ready for Instantiation
Upon instantiation, we'll only ask the user to name the badge.  We'll return to the user the instantiated component, as well as the first admin badge managed by the component.

```rust
pub fn instantiate_flatadmin(badge_name: String) -> (Component, Bucket) {
```

We'll want our supply of admin badges to be mutable.  Mutable supply resources can only be minted and burned by an appropriate authority, so we'll first create a badge to serve as that authority, and then use that new badge to create our supply of admin badges.

```rust
let admin_mint_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
    .initial_supply_fungible(1);
let admin_badge_def = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
    .metadata("name", badge_name)
    .flags(MINTABLE | BURNABLE)
    .badge(admin_mint_badge.resource_def(), MAY_MINT | MAY_BURN)
    .no_initial_supply();
```

With that out of the way, we can mint our first admin badge and create our component.  We'll tuck our sole minting authority badge safely away within its vault.  Then we'll return the new component and the admin badge.

```rust
let first_admin_badge = admin_badge_def.mint(1, admin_mint_badge.present());
let component = Self {
    admin_mint_badge: Vault::with_bucket(admin_mint_badge),
    admin_badge: admin_badge_def
}
.instantiate();

(component, first_admin_badge)
```

## Allowing Users to Mint and Burn Admin Badges
In order for `FlatAdmin` to be more useful than just manually creating a single admin badge, it needs the capability to create and destroy admin badges.

Obviously we don't want just anyone to be able to create additional admin badges at will, so that privilege is protected by having to prove that you're already in possession of an admin badge.

```rust
#[auth(admin_badge)]
pub fn create_additional_admin(&self) -> Bucket {
  self.admin_mint_badge
    .authorize(|auth| self.admin_badge.mint(1, auth))
}
```

The `authorize` method is a convenience method which allows us to present the badge contained within our `admin_mint_badge` vault without having to fetch it, present it, and return it.  The closure syntax using `|` characters may be unfamiliar to you: think of `|auth|` as being equivalent to `(auth) ->` in Java or `(auth) =>` in C#.
