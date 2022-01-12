# Day 18 - Price Based token unlock
Today, we are building a price based unlock scheduler where a percentage of locked tokens gets unlocked as the price increases.

## Prices at which unlock occurs in the example

|price|percentage unlocked|
| ----| ----|
| 10    | 10%  |
| 20    | 30%  |
| 50    | 60%  |
| 60    | 100% |

## How to test

1. Reset your environment: `resim reset`
1. Create two accounts. Call `resim new-account` twice. Store the returned public keys and addresses somewhere.
1. Create the token to do the unlock with: `resim new-token-fixed --name Bubblegum 10000000`. Note the returned ResourceDef address.

### Setup PriceOracle
1. `cd price_oracle`
1. Build and deploy on the ledger: `resim publish .`
1. Instantiate a PriceOracle component: `resim call-function [package_address] PriceOracle new`. Note the returned ResourceDef and component address somewhere. The resource is the USD token.

## Setup Unlocker
1. `cd ../unlocker`
1. Build and deploy on the ledger: `resim publish .`
1. Instantiate an Unlocker component with the resource being XRD and the instantiated price oracle address: `resim call-function [unlocker_package_address] PriceBasedUnlockScheduler new [bubblegum_address] [price_oracle_component_address]`. Store the first and third returned ResourceDef somewhere. Those are the admin badge and recipient NFT definition.
1. Add the two accounts as recipients: `resim call-method [component_address] add_recipient [account1_address] 1000,[bubblegum_address] 1,[admin_badge_address]`

## Test the unlock
1. Increase the price of the BubbleGum to 10 USD: `resim call-method [oracle_address] update_price [bubblegum_address] [usd_address] 10`
1. As admin of the Unlocker component, trigger the unlock: `resim call-method [component_address] do_unlock 1,[admin_badge_address]`. You should see that the unlocked percentage is 10%.
1. As account1 (also the admin), withdraw the 10% unlocked tokens: `resim call-method [component_address] withdraw 1,[recipient_nft_address]`
1. You should see 100 more gumballs in your account's balance: `resim show [account1_address]`
1. Increase the price to 20 USD: `resim call-method [oracle_address] update_price [bubblegum_address] [usd_address] 20`
1. Trigger the unlock again: `resim call-method [component_address] do_unlock 1,[admin_badge_address]`. Now 30% should be unlocked.
1. As account1, withdraw the 20% unlocked tokens (30% - 10% already unlocked): `resim call-method [component_address] withdraw 1,[recipient_nft_address]`
1. You should see 200 more gumballs in your account's balance: `resim show [account1_address]`
1. Let's try with account 2 to see if they still get the full 30%: `resim set-default-account [account1_address] [account1_pubkey]`
1. Withdraw the tokens: `resim call-method [component_address] withdraw 1,[recipient_nft_address]`
1. You should see 300 total gumballs in account 2: `resim show [account2_address]`