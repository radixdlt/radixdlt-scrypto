# Hello, NFT!

From Wikipedia,

> A non-fungible token (NFT) is a unique and non-interchangeable unit of data stored on a digital ledger (blockchain). NFTs can be associated with easily-reproducible items such as photos, videos, audio, and other types of digital files as unique items

In this example, we will show you how to build a ticket vending machine in Scrypto.

## How to Play?

1. Create a new account, and save the account address
```
resim new-account
```
2. Publish the package, and save the package address
```
resim publish .
```
3. Call the `new` function to instantiate a component, and save the component address
```
resim call-function <PACKAGE_ADDRESS> HelloNft new 5
```
4. Call the `get_available_ticket_ids`
```
resim call-method <COMPONENT_ADDRESS> get_available_ticket_ids
```
4. Call the `buy_ticket` method
```
resim call-method <COMPONENT_ADDRESS> buy_ticket <TICKET_ID> "100,030000000000000000000000000000000000000000000000000004"
```
5. Check out our balance
```
resim show <ACCOUNT_ADDRESS>
```