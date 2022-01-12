# Day 20 - DAO Membership System
From today to the 25th, we will be building a DAO block by block. Today, we are building a Membership system where users can become members, contribute to the DAO to get points and ban members.

## How to test
1. Reset your environment: `resim reset`
1. Create two accounts: call `resim new-account` twice. Note the addresses and public keys.
1. Build and deploy the blueprint on the ledger: `resim publish .`
1. Instantiate a component from the blueprint: `resim call-function [package_address] MembershipSystem new`. Take note of the second ResourceDef (member NFT) and the component address
1. Become a member: `resim call-method [component_address] become_member [your_name]`
1. You should see the member NFT in your account: `resim show [account1_address]`
1. Set the second account as default: `resim set-default-account [account2_address] [account2_pubkey]`
1. Become a member as account 2: `resim call-method [component_address] become_member [your_name]`
1. Try to ban account1 (nft id 1): `resim call-method [component_address] ban_member 1 1,[nft_address]`. You should get an error since you do not have enough points.
1. Contribute XRD to the DAO to receive points: `resim call-method [component_address] contribute 100000,030000000000000000000000000000000000000000000000000004 1,[nft_address]`
1. Now try to ban account1 again: `resim call-method [component_address] ban_member 1 1,[nft_address]`
1. Switch to the other account: `resim set-default-account [account1_address] [account1_pubkey]`
1. Try to contribute: `resim call-method [component_address] contribute 100000,030000000000000000000000000000000000000000000000000004 1,[nft_address]`. You should get an error stating that you are banned.