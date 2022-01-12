# Day 24: Election System for DAO
Today we are building an election system where members can vote on who should be the next leader of the DAO.

## How to test
1. Reset your environment: `resim reset`
1. Create three accounts. Call `resim new-account` three times.
1. Build and deploy the blueprint on the ledger: `resim publish .`
1. Instantiate a new ElectionSystem component with an election duration of 100 epochs: `resim call-function [blueprint_address] ElectionSystem new 100`. Take note of the two last components. The first one is the membership_system. The last one is the election_system.
1. Become a member as account1: `resim call-method [membership_system_address] become_member Alice`
1. Look at the resources of the account1 to note the membership NFT resource def: `resim show [account1_address]`
1. Start a new election: `resim call-method [election_address] start_election 1 1,[member_nft_address]`
1. Vote for yourself (member NFT id #1): `resim call-method [election_address] vote 1,[member_nft_address]`
1. Set the default account to account2: `resim set-default-account [account2_address] [account2_pubkey]`
1. Become a member: `resim call-method [membership_system_address] become_member Bob`
1. Vote for account1 again: `resim call-method [election_address] vote 1 1,[member_nft_address]`
1. Set the default account to account3: `resim set-default-account [account3_address] [account3_pubkey]`
1. Become a member: `resim call-method [membership_system_address] become_member Carl`
1. Vote for account2 now: `resim call-method [election_address] vote 2 1,[member_nft_address]`
1. Set epoch to 100: `resim set-current-epoch 100`
1. Close the election: `resim call-method [election_address] close_election 1,[member_nft_address]`
1. Display the current leader: `resim call-method [election_address] get_current_leader`