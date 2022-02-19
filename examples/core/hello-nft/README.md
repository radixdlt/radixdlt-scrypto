# Hello, NFT!

From Wikipedia,

> A non-fungible token (NFT) is a unique and non-interchangeable unit of data stored on a digital ledger (blockchain). NFTs can be associated with easily-reproducible items such as photos, videos, audio, and other types of digital files as unique items

In this example, we will show you how to build a ticket vending machine in Scrypto.

## Blueprint and Component

The blueprint we're building is called `HelloNft`. Each `HelloNft` manages the following resources and data.

```rust
struct HelloNft {
    /// A vault that holds all available tickets.
    available_tickets: Vault,
    /// The price for each ticket.
    ticket_price: Decimal,
    /// A vault for collecting payments.
    collected_xrd: Vault,
}
```

The `available_tickets` contains non-fungible `Ticket` resource. Both fungible and non-fungible resources are stored in a `Vault`.

## Creating NFT Units

In our example, the supply of NFT units are fixed, and we allocate the resource upfront.

First, we prepare the data for each NFT unit (every ticket is associated with a specific row and column number).

```rust
let mut tickets = Vec::new();
for row in 1..5 {
    for column in 1..5 {
        tickets.push((NftKey::from(Uuid::generate()), Ticket { row, column }));
    }
}
```

Then, the whole vector of NFT data is passed to `ResourceBuilder` as the initial supply.

```rust
let ticket_bucket: Bucket = ResourceBuilder::new_non_fungible()
    .metadata("name", "Ticket")
    .initial_supply_non_fungible(tickets);
```

After that, we get a bucket of NFT units stored in `ticket_bucket`.

## Allowing Callers to Buy Tickets

A `HelloNft` component exposes three public methods:

* `buy_ticket`: allowing caller to buy one ticket;
* `buy_ticket_by_id`: allowing caller to buy one specific ticket;
* `available_ticket_ids`: returns the IDs of all available tickets.

The workflow of `buy_ticket` and `buy_ticket_by_id` is very similar.

```rust
self.collected_xrd.put(payment.take(self.ticket_price));
let ticket = self.available_tickets.take(1);
// OR let ticket = self.available_tickets.take_non_fungible(id);
(ticket, payment)
```

Both involves
1. Taking a payment according to pre-defined price and putting it into the `collected_xrd` vault;
1. Taking a ticket from the `available_tickets` vault:
   * `take(1)` returns one NFT unit;
   * `take_non_fungible(id)` returns the specified NFT unit.
1. Returning the ticket and payment change.

## How to Play?

1. Create a new account, and save the account address
```
resim new-account
```
2. Publish the package, and save the package address
```
resim publish .
```
3. Call the `instantiate_hello` function to instantiate a component, and save the component address
```
resim call-function <PACKAGE_ADDRESS> HelloNft instantiate_hello 5
```
4. Call the `available_ticket_ids`
```
resim call-method <COMPONENT_ADDRESS> get_available_ticket_ids
```
5. Call the `buy_ticket_by_id` method
```
resim call-method <COMPONENT_ADDRESS> buy_ticket <TICKET_ID> "100,030000000000000000000000000000000000000000000000000004"
```
6. Check out our balance
```
resim show <ACCOUNT_ADDRESS>
```