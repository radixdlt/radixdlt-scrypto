# Day 23 - Rating System
Today, we are building a rating system, where users can rate the services of the members of the DAO.

## How to test
1. Reset your environment: `resim reset`
1. Create an account: `resim new-account`
1. Build and publish the blueprint on the ledger: `resim publish .`
1. Instantiate a RatingSystem component: `resim call-function [package_address] RatingSystem new`. Take note of the two returned component addresses. The first one is the membership_system and the second is the rating_system.
1. Register as a member: `resim call-method [membership_system_address] become_member Alice`
1. Look at the resources on account 1 to know the resource def of the member nft: `resim show [account1_address]`
1. Create a new service: `resim call-method [rating_system_address] create_service smart_contract_development 1,[member_nft_address]`
1. Make a review on that service: `resim call-method [rating_system_address] review_service 1 smart_contract_development 0 0 "They wrote the contract in Solidity !"`
1. Make another review: `resim call-method [rating_system_address] review_service 1 smart_contract_development 0 5 "Bug-free and made with Scrypto !"`
1. List the reviews: `resim call-method [rating_system] display_ratings 1`