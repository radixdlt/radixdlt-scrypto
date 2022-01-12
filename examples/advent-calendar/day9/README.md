# Day 9 - Santa and PresentList components
Today, we are learning how to import a component from another package. We are going to make a Santa component interacting with a PresentList component.

## How to test
1. Reset your environment: `resim reset`
2. Create the default account: `resim new-account`

### Configure PresentList
3. `cd present_list`
4. Build and publish the blueprint: `resim publish .`. Save the package address somewhere. We will need it later.
5. Instantiate a PresentList component: `resim call-function [package_address] PresentList new`. Store component address somewhere.
6. Start a new list: `resim call-method [component_method] start_new_list`. Save the returned badge address.
7. Add multiple presents to the list: `resim call-method [component_method] add [name] 1,[list_badge_address]`.
8. You can preview your list with: `resim call-method [component_method] display_list 1,[list_badge_address]`.

### Configure Santa
8. cd `../santa`
9. Update line 12 of `santa/lib.rs` with the package address of the PresentList blueprint.
10. Build and publish de Santa blueprint: `resim publish .`
11. Instantiate a new Santa component: `resim call-function [santa_package_address] Santa new [present_list_component_address]`.
12. Prepare the gifts: `resim call-method [santa_component_address] prepare_gifts`
13. Withdraw the gifts: `resim call-method [santa_component_address] withdraw_gifts 1,[list_badge_address]`
14. Look at the resources in your account: `resim show [account_address]`. You should see the gifts you requested !