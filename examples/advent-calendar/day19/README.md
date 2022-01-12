# Day 19 - Subscription Payment System
Today will be a little bit different. 
The Alexandria update introduced many flags that we can attach to tokens. One of them is `RECALLABLE`, allowing someone presenting a badge with the `MAY_RECALL` permission to recall tokens from any vaults. I wanted to explore an interesting use case with this feature. Unfortunatly, the functionality to recall tokens have not been implemented yet. Thus, this example is just a proof of concept and won't build.

## How it would work
1. A user would swap their XRD for Payment tokens by calling `buy_payment_tokens(payment: Bucket)`
1. The user would request a new subscription to an external service.
1. That service would call the `setup_subscription` of the `RecurrentPayment` component and send the returned NFT to the user.
1. A script would call the `take_payments` methods every epochs.
1. If a payment is due, the component will take X Payment tokens from the user's vault and send the same amount of XRD to the service.