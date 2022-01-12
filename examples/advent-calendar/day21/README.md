# Day21 - Funds Splitter
Today, we continue building a DAO block by block. We are adding a funds splitter component on top of the membership system created yesterday.

## How to test
1. Reset your environment: `resim reset`
1. Create two accounts: call `resim new-account` twice. Take note of the two addresses and public keys.
1. Build and deploy the blueprint on the ledger: `resim publish .`
1. Instantiate a FundsSplitter component: `resim call-function [package_address] FundsSplitter new`. Take note of the two component addresses. The first one is the MembershipSystem and the second is the FundsSplitter.
1. Register as a new member named Musk: `resim call-method [membership_system_address] become_member Musk`
1. Send funds to the splitter: `resim call-method [splitter_address] add_funds 100000,030000000000000000000000000000000000000000000000000004`
1. Look at the resources on account1 to find the Member NFT resource definition: `resim show [account1_address]`
1. Withdraw your share of funds (100%): `resim call-method [splitter_address] withdraw 1,[member_nft_address]`.
1. Switch to account2: `resim set-default-account [account2_address] [account2_pubkey]`
1. Register as a new member named Warren: `resim call-method [membership_system_address] become_member Warren`
1. Switch back to account1: `resim set-default-account [account1_address] [account1_pubkey]`
1. Add funds to the splitter: `resim call-method [splitter_address] add_funds 100000,030000000000000000000000000000000000000000000000000004`
1. Withdraw your share of funds (50%, now that there are two members): `resim call-method [splitter_address] withdraw 1,[member_nft_address]`
1. Look at the amount of XRD on account1: `resim show [account1_address]`. It should contain 950 000 XRD.