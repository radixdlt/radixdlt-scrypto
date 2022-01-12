# Day 11 - GiftExchange with PriceOracle
Today we are building a more complex component. You will learn how to make a GiftExchange component connected to a PriceOracle to make sure no one send too expensive gifts.

## How to test

1. Reset your environment: `resim reset`
1. Create two accounts. Call `resim new-account` twice. Store the public keys and addresses somewhere.

### Configure the PriceOracle
1. `cd PriceOracle`
1. Build and deploy the PriceOracle blueprint: `resim publish .`. Remember the returned package address.
1. Create a PriceOracle component: `resim call-function [package_address] PriceOracle new`. This will return the USD resource definition and the component's address. Remember them.
1. Create different gifts and add their price on the oracle:
    - `resim new-token-fixed --name Flower 1`. And `resim call-method [oracle_component] update_price [flower_address] [usd_address] 5`
    - `resim new-token-fixed --name TeaPot 1`. And `resim call-method [oracle_component] update_price [teapot_address] [usd_address] 10`
    - `resim new-token-fixed --name iPod 1`. And `resim call-method [oracle_component] update_price [ipod_address] [usd_address] 400`
1. Send the tea pot and ipod to the second account: `resim transfer 1,[teapot_address] [account2_address]` and `resim transfer 1,[ipod_address] [account2_address]`

### Configure the GiftExchange
1. `cd ../GiftExchange`
1. Update line 7 of `GiftExchange/src/lib.rs`, add the price oracle package address
1. Build and publish the package on the ledger: `resim publish .`
1. Generate a GiftExchange component: `resim call-function [package_address] GiftExchange new [oracle_component_address]`. Save the returned component address and organizer badge.
1. Add the two participants. Call `resim call-method [component_address] add_participant [account_address] 1,[organizer_badge]` twice with the two generated account addresses. This will give you the participant badges.
1. Let the component decide who should give to who: `resim call-method [component_address] prepare_exchange 1,[organizer_badge]`.
1. Send a gift as account1: `resim call-method [component_address] send_gift 1,[flower_address] 1,[participant1_badge]`.
1. You should see the gift in account2: `resim show [account2_address]`
1. Set account2 as default account: `resim set-default-account $acc2 $pub2`
1. Try to send an iPod: `resim call-method [component_address] send_gift 1,[ipod_address] 1,[participant2_badge]`. You should get an error that the price of the gift is too much !
1. Send the tea pot: `resim call-method [component_address] send_gift 1,[teapot_address] 1,[participant2_badge]`