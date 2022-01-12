# Day 2 - SantaCookieEater Component
Today, we will build a SantaCookieEater component. This component consists of a single `give_food` method allowing the callers to send "Cookie" tokens to this component.

## How to test
1. Reset your environment: `resim reset`
1. Create a default account: `resim new-account`
2. Build and publish the blueprint to the component: `resim publish .`. Save the address of the package somewhere.
3. Instantiate a component: `resim call-function [package_address] SantaCookieEater new`

The last command returns two addresses. The address of the created "Cookie" tokens and the address of the component. Remember those, we will need them in the next steps

4. Try to send 500 XRD to the component: `resim call-method [component_address] give_food 500,030000000000000000000000000000000000000000000000000004`. You should get an error message stating that this component only wants "Cookie" tokens
5. Try to send "Cookie" tokens: `resim call-method [component_address] give_food 500,[cookie_address]`
6. You should get a "Thank you !" message