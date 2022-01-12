# Day 12: Yankee Swap component
Today, we are going to build a component allowing a group a people to play [Yankee Swap](https://www.secretsanta.com/yankee-swap-rules/) !

## How to test
1. Reset your environment: `resim reset`
1. Create three accounts. Call `resim new-account` three times. Remember the returned addresses and public keys
1. Build and deploy the blueprint on the ledger: `resim publish .`
1. Instantiate the component: `resim call-function [package_address] YankeeSwap new`. The returned ResourceDef is the admin's badge. Note it somewhere.

### Create the gifts:
1. `resim new-token-fixed --name book 1`
1. `resim new-token-fixed --name mug 1`
1. Send the mug to account2: `resim transfer 1,[mug_address] [account2_address]`
1. `resim new-token-fixed --name giftcard 1`
1. Send the giftcard to account3: `resim transfer 1,[giftcard_address] [account3_address]`

1. Enter the game as account1: `resim call-method [component_address] enter_swap 1,[book_address]`. Note the returned BucketRef somewhere. This is the participant's badge
1. Set account2 as default: `resim set-default-account [account2_address] [account2_pub_key]`
1. Enter the game as account2: `resim call-method [component_address] enter_swap 1,[mug_address]`. Note the returned BucketRef somewhere. This is the participant's badge
1. Set account3 as default: `resim set-default-account [account3_address] [account3_pub_key]`
1. Enter the game as account3: `resim call-method [component_address] enter_swap 1,[giftcard_address]`. Note the returned BucketRef somewhere. This is the participant's badge
1. Set account1 (admin) as default: `resim set-default-account [account1_address] [account1_pub_key]`


### Playing the game
1. Start the game: `resim call-method [component_address] start 1,[admin_badge]`
1. Display the status: `resim call-method [component_address] current_gift`. This will tell you who the current player is and which gift they picked. You will also see the list of gift that this player can swap with.
1. Set the default account to the one that should decide: `resim set-default-account [account_address] [pub_address]`
1. Swap with the gift at index 0: `resim call-method [component_address] swap 0 1,[participant_address]`
1. Now, if you call `resim call-method [component_address] current_gift` again, the information should be updated.
1. Set the default account to the one that should decide: `resim set-default-account [account_address] [pub_address]`
1. Decide to keep the gift: `resim call-method [component_address] keep 1,[participant_address]`
1. You should now see that the game is ended.

### Withdrawing the gifts
1. Set account3 as default: `resim set-default-account [account3_address] [account3_pub_key]`
1. Withdraw the gift of account 3: `resim call-method [component_address] withdraw 1,[participant3_address]`
1. Set account2 as default: `resim set-default-account [account2_address] [account2_pub_key]`
1. Withdraw the gift of account 2: `resim call-method [component_address] withdraw 1,[participant2_address]`
1. Set account1 as default: `resim set-default-account [account1_address] [account1_pub_key]`
1. Withdraw the gift of account 1: `resim call-method [component_address] withdraw 1,[participant1_address]`
1. You should now see the three gifts in the resources of the accounts: `resim show [account_address]`
