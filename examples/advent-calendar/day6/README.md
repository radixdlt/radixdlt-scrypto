# Day 6 - Present List with badges
Toady we are building a present list component allowing multiple people to manage their Christmas list. You will learn how you can use badges as IDs to fetch and update the user's list.

## How to test
1. Reset the environment: `resim reset`
2. Create an account: `resim new-account`. Save the accound address somewhere.
3. Build and publish the blueprint to the ledger: `resim publish .`. Remember the returned package address, you will need it in the next step.
4. Call the `new` function to generate the component: `resim call-function [package_address] PresentList new`. Save the component's address somewhere.
5. Start a new list: `resim call-method [component_address] start_new_list`
6. This will mint a new list badge and deposit it in your account. Take note of the badge address: `resim show [account_address]`
7. Add an item to your list: `resim call-method [component_address] add [present_name] 1,[list_badge_address]`
8. Display your list with: `resim call-method [component_address] display_list 1,[list_badge_address]`
9. You can remove an item with `resim call-method [component_address] remove [present_name] 1,[list_badge_address]`