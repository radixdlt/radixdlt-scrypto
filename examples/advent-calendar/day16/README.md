# Day 16 - Degenerate Elf NFT
Today, you will learn how to create your own degenerate NFT project !

## How to test
1. Reset your environment: `resim reset`
1. Create the default account: `resim new-account`
1. Build and publish the blueprint on the ledger: `resim publish .`
1. Instantiate a new DegenerateElves component with minting cost of 20 XRD and a max supply of 10000: `resim call-function [package_address] DegenerateElves new 20 10000`. Store the component's address somewhere.
1. Mint some nfts: `resim call-method [component_address] mint 20,030000000000000000000000000000000000000000000000000004`
1. Look at the resources on your account: `resim show [account_address]`. Find the NFT's resource definition address.
1. Display the properties of your nfts: `resim call-method [component_address] display_info 1,[nft_resource_definition]`.
1. Share the attributes you got in the [Scrypto Discord chanel](https://discord.gg/radixdlt) !