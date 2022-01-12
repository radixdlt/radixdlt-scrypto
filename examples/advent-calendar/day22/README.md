# Day 22 - Proposal Voting System for a DAO
Today, we are going to build a Proposal System for the DAO we are building. This component will allow members to create new proposals and vote on them.

## How to test
1. Reset your environment: `resim reset`
1. Create two accounts: call `resim new-account` twice and note the returned addresses and public keys.
1. Build and deploy the blueprints on the ledger: `resim publish .`
1. Instantiate a new ProposalVoting component: `resim call-function [package_address] ProposalVoting new`. Take note of the two component addresses. The first one is the MembershipSystem and the last one is the ProposalVoting.
1. Request to become a member: `resim call-method [membership_system_address] become_member Alice`
1. Find the membership NFT resource definition by looking at your resources: `resim show [account1_address]`
1. Create a new proposal: `resim call-method [proposal_voting_address] create_proposal "Increase Salary" "I think we should increase the salary, do you agree ?" 1,[member_nft_address]`
1. Switch to the second user: `resim set-default-account [account2_address] [account2_pubkey]`
1. Register as a member: `resim call-method [membership_system_address] become_member Bob`
1. Vote for the proposal: `resim call-method [proposal_voting_address] vote_on_proposal 0 1,[member_nft_address]`
1. Display the list of proposals: `resim call-method [proposl_voting_address] list_proposals`