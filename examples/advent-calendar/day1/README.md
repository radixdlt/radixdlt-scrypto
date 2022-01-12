# Day 1 - PresentFactory
Today, we will be building a present factory. 
It allows people to push new presents to an HashMap stored on the component's state.

## How to run

1. Reset the environment: `resim reset`
1. Create an account: `resim new-account`
1. Build and publish the blueprint on the ledger: `resim publish .`. Save resulted package address somewhere.
1. Call the `new` method on the blueprint to generate a component: `resim call-function [package_address] PresentFactory new`. Save the resulted component's address somewhere.
1. Add a new present by calling the `create_present` method on the component: `resim call-method [component_address] create_present [name] [quantity]`
1. After adding a bunch of presents, call the `list_presents` method on the component: `resim call-method [component_address] list_presents`
