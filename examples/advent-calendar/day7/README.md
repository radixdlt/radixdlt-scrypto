# Day 7 - Elf Workshop
Today, we are learning how to make an ElfWorkshop blueprint. This blueprint will keep track of the different elf employees and their respective list of created toys !

## How to test
1. Reset your environment: `resim reset`
1. Create two accounts: call `resim new-account` two times. Take note of the addresses and public keys.
1. Build and deploy the blueprint on the ledger: `resim publish .`. Remember the package address for the next step.
1. Instantiate a new component from the blueprint: `resim call-function [package_address] ElfWorkshop new`. Take note of the component address
1. Call the `become_elf` method: `resim call-method [component_address] become_elf`
1. You can see your badge by calling: `resim show [account_1_address]`
1. Create a toy: `resim call-method [component_address] create_toy RubikCube 1,[elf_badge_address]`
1. Call the last method multiples times. You should see the counter increase !
1. Set the second account as the default one: `resim set-default-account [account_2_address] [account_2_pubkey]`
1. Try to create a toy by providing XRD as badge: `resim call-method [component_address] create_toy LegoBlock 1,030000000000000000000000000000000000000000000000000004`. You should get an error !