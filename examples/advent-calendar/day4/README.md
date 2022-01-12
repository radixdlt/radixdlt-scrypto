# Day 4 - House component
Today, we are building a House component containing a single `enter` method. I am using badges to make sure only Santa and the house owner can call this method.

## How to test
1. Reset your environment: `resim reset`
2. Create three users: call `resim new-account` three times. Store somewhere the account's public keys and addresses. We will need them later
3. Build and deploy the blueprint to the ledger: `resim publish .`. Remember the returned package address
4. Instantiate a component: `resim call-function [package_address] House new`. Remember the returned component address

- Account 1 will act as Santa.
- Account 2 will act as the house owner.
- Account 3 will act as a thief trying to steal the presents inside the house.

5. Right now account 1 has both the Santa and Owner badges: `resim show [account_1_address]`. Let's send the owner badge to account 2: `resim transfer 1,[owner_badge_address] [account2_address]`
6. You can verify account 2 received it by typing: `resim show [account_2_address]`
7. Call the `enter` method as account 1: `resim call-method [component_address] enter 1,[santa_badge_address]`
8. Make account 2 the default user: `resim set-default-account [account2_address] [account_2_pubkey]`
9. Call the `enter` method as account 2: `resim call-method [component_address] enter 1,[owner_badge_address]`
10. Make account 3 the default user: `resim set-default-account [account3_address] [account_3_pubkey]` 
11. Try to call the `enter` by providing XRD as badge: `resim call-method [component_address] enter 1,030000000000000000000000000000000000000000000000000004`. You should get an error !
12. Try again but use santa's badge address. `resim call-method [component-address] enter 0,[santa_badge_address]`. As you will see, dressing up like santa by using the santa_badge_address is not good enough, You need the actual badge!