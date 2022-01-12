# Day 17 - PresentList part 2: the return of the NFT
Today, we are revisiting the example from day 6. Now, instead of storing the lists in the state of the component, we are storing them in NFTs.

## How to test
1. Reset your environment: `resim reset`
1. Create the default account: `resim new-account`
1. Build and deploy the blueprint to the ledger: `resim publish .`
1. Instantiate a PresentList component: `resim call-function [package_address] PresentListWithNFT new`. The second returned ResourceDef is the NFT's definition address. Note it somewhere.
1. Start a new list: `resim call-method [component_address] start_new_list`
1. Add an item to your list: `resim call-method [component_address] add [present_name] 1,[nft_definition_address]`
1. Display the list: `resim call-method [component_address] display_list 1,[nft_definition_address]`
1. Remove an item from your list: `resim call-method [component_address] remove [present_name] 1,[nft_definition_address]`
1. Display the list: `resim call-method [component_address] display_list 1,[nft_definition_address]`. The item should have been removed