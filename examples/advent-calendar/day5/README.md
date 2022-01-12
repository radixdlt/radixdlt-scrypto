# Day 5 - PresentDeliverySchedule
Today, we will learn how to get the current epoch from the components and how to change the epoch with resim by building a delivery schedule component !

## How to test
1. Reset your environment: `resim reset`
1. Create a default account: `resim new-account`
1. Build and deploy the blueprint on the ledger: `resim publish .`. Remember the generated package address for the next step.
1. Instantiate a component: `resim call-function [package_address] PresentDeliverySchedule new`. Save the component's address somewhere.
1. To display the current epoch and the places left to visit type: `resim call-method [component_address] display_schedule`
1. Let's change the current epoch so that Santa is late for Africa: `resim set-current-epoch 3`
1. Call the `display_schedule` method again. You should see that the current epoch increased and that Santa is not on schedule anymore.
1. Let's fix that by adding Africa as visited: `resim call-method [component_address] add_done Africa`
1. Call the `display_schedule` method again. Now Santa should be on schedule and Africa should not be displayed in the list of places left to visit.
