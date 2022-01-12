# Day 3 - PresentDistributor
Let's build a present distributor component. It allows users to add good or naughty kids to the list and then distribute either presents or coal to them.

## How to test
1. Reset your environment: `resim reset`
1. Create 3 accounts: Call `resim new-account` three times, saving the account's address somewhere everytime
1. Build and deploy the blueprint on the ledger: `resim publish .`
1. Instantiate a component from the blueprint: `resim call-function [package_address] PresentDistributor new`. Save the component's address somewhere.
1. Add a good kid to the list: `resim call-method [component_address] add_kid [account_1_address] false`
1. Add a naughty kid to the list: `resim call-method [component_address] add_kid [account_2_address] true`
1. Add another naughty kid to the list: `resim call-method [component_address] add_kid [account_3_address] true`
1. Call the `distribute_gifts` methods on the component: `resim call-method [component_address] distribute_gifts`
1. Check the three account balances: `resim show [address]`.