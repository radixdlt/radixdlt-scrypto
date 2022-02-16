# Magic Card

In this example, we're going to show you how to use NFT in Scrypto under a game context.

## How to Create NFT?

NFT is just another type of resource in Scrypto, and the way to define NFTs is through `ResourceBuilder`.

To create a fixed supply of NFTs, we will need to define the NFT data structure first, like
```rust
#[derive(NonFungibleData)]
pub struct MagicCard {
    color: Color,
    rarity: Rarity,
    #[scrypto(mutable)]
    level: u8
}
```

and pass an array of instances to resource builder:

```rust
let special_cards_bucket = ResourceBuilder::new_non_fungible()
    .metadata("name", "Russ' Magic Card Collection")
    .initial_supply_non_fungible([
        (
            NftKey::from(1u128), // The ID of the first NFT, you can also use `Uuid::generate()` to create a random ID
            MagicCard {
                color: Color::Black,
                rarity: Rarity::MythicRare,
                level: 2,
            }
        ),
        (
            NftKey::from(2u128), // The ID of the second NFT
            MagicCard {
                color: Color::Green,
                rarity: Rarity::Rare,
                level: 3,
            }
        )
    ]);
```

To create NFTs with mutable supply, 

```rust
let random_card_mint_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
    .metadata("name", "Random Cards Mint Badge")
    .initial_supply_fungible(1);
let random_card_resource_def = ResourceBuilder::new_non_fungible()
    .metadata("name", "Random Cards")
    .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
    .badge(
        random_card_mint_badge.resource_def(),
        MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA,
    )
    .no_initial_supply();
```

Once the resource is created, we can mint NFTs with the `mint_non_fungible` method:
```rust
let nft = self.random_card_mint_badge.authorize(|auth| {
    self.random_card_resource_def.mint_non_fungible(
        // The NFT id
        self.random_card_id_counter,
        // The NFT data
        MagicCard {
            color: Self::random_color(random_seed),
            rarity: Self::random_rarity(random_seed),
            level: 5,
        },
        // authorization to mint
        auth
    )
});
```

## Transfer to Another Account/Component

Since NFT is just another type of resource, it must be stored in either a bucket and vault. To transfer one NFT to another account, we will need to withdraw it from the sender's account and deposit into the recipient's account.

To pick a specific NFT when calling a function or method, we can use the following syntax:

```
#nft_id_1,#nft_id2,resource_address
```

## Update an Existing NFT


To update, we need to call the `update_non_fungible_data` method on resource definition.

```rust
let nft = self.random_card_mint_badge.authorize(|auth| {
    self.random_card_resource_def.update_non_fungible_data(
        // The NFT id
        self.random_card_id_counter,
        // The new NFT data
        MagicCardMut {
            color: Color::Green,
            rarity: Rarity::Common,
            level: 100,
        },
        // authorization to update
        auth
    )
});
```

## How to Play?

1. Create a new account, and save the account address
```
resim new-account
```
2. Publish the package, and save the package address
```
resim publish .
```
3. Call the `instantiate_component` function to instantiate a component, and save the component address
```
resim call-function <PACKAGE_ADDRESS> HelloNft instantiate_component
```
4. Call the `buy_random_card` method of the component we just instantiated
```
resim call-method <COMPONENT_ADDRESS> buy_random_card "1000,030000000000000000000000000000000000000000000000000004"
```
4. Call the `buy_random_card` method again
```
resim call-method <COMPONENT_ADDRESS> buy_random_card "1000,030000000000000000000000000000000000000000000000000004"
```
5. Check out our balance
```
resim show <ACCOUNT_ADDRESS>
```
6. Fuse our random cards
```
resim call-method <COMPONENT_ADDRESS> fuse_my_cards "#0,#1,03d8541671ab09116ae450d468f91e5488a9b22c705d70dcfe9e09"
```
7. Check out our balance again and we should see a upgraded card
```
resim show <ACCOUNT_ADDRESS>
```