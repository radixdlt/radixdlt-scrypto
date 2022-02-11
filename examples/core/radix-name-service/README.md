# Radix Name Service (RNS)
This is a very basic implementation of a DNS on Radix. It is conceptually similar to and heavily inspired by the ENS
project: https://ens.domains

Something like the RNS would typically be integrated with wallets to allow users to send funds to a human-readable
addresses like e.g. `satoshi.xrd` instead of a cryptic and long ledger addresses like e.g.
`02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b`.

# How to use RNS
You can follow the steps below to instantiate a new RNS component and simulate some standard usage.  

Those steps can also be executed automatically by calling `revup -r rns.revup` in a terminal. Please note that in order
for this to work you must have the [revup](https://github.com/RadGuild/revup) utility by @RockHoward installed!

In a terminal:

1. Reset the simulator
```
resim reset
```
2. Create a new account to administer the RNS component. Save the account address to `$admin_account`
```
resim new-account
```
3. Publish the package. Save the package address to `$package`
```
resim publish .
```
4. Instantiate a new RNS component.  
The component is instantiated with the following parameters:
deposit_per_year=50, fee_address_update=10 and fee_renewal_per_year=25 (all values are in XRD).  
Save the address of the admin badge to `$admin_badge` (first new entity), the address of the DomainName resource
to `$name_resource` (third new entity) and the component address to `$component` (fourth new entity)
```
resim call-function $package RadixNameService new 50 10 25 
```
5. Simulate that a user comes along and uses the RNS component.  
Save the account address to `$user_account` and the public key to `$user_pubkey`
```
resim new-account
```
6. Set this account as the new default account
```
resim set-default-account $user_account $user_pubkey
```
7. Simulate that the user registers the name "satoshi.xrd" to point to his account address.  
The name is reserved for 10 years which requires a refundable deposit of $XRD 500
```
resim call-method $component register_name satoshi.xrd $user_account 10 "500,030000000000000000000000000000000000000000000000000004"
```
8. Display the user's account
```
resim show $user_account
```
Taking a look at the account, please note that the user is now the owner of a DomainName NFT that represents his
ownership of the "satoshi.xrd" name:
```
Resources:
├─ { amount: 999500, resource_def: 030000000000000000000000000000000000000000000000000004, name: "Radix", symbol: "XRD" }
└─ { amount: 1, resource_def: 03d8541671ab09116ae450d468f91e5488a9b22c705d70dcfe9e09, name: "DomainName" }
  └─ NFT { id: 339715316826500606461318410874891739268, immutable_data: Struct {  }, mutable_data: Struct { 02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b, 150000, 500 } }
```
The NFT has an ID of 339715316826500606461318410874891739268 because that is, what "satoshi.xrd" is hashed to.
Next, in the mutable_data part there are 3 values:
- the address that the name maps to (02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b)
- the last epoch in which the mapping is valid (150000)
- the amount of XRD that has been deposited when registering this name (500)

9. Call the lookup_address method for "satoshi.xrd" and observer that the name maps to
02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b, which is indeed the account address of the user.  
You will find this address in the "Results" section of the transaction receipt.
```
resim call-method $component lookup_address satoshi.xrd
```
10. Now, simulate that the user creates another account to which future payments should be directed to.  
Save the account address to `$new_user_account`
```
resim new-account
```
11. The name mapping can be changed by calling the update_address method on the RNS component.  
The parameters to this method are:  
1: A BucketRef with the DomainName NFT that demonstrates the user's ownership of the name and his right to change  
the mapped address (#339715316826500606461318410874891739268,$name_resource)  
2: The address of the newly created account ($new_user_account)  
3: A bucket that contains the fee for the name update (10,$tokenXRD)
```
resim call-method $component update_address "#339715316826500606461318410874891739268,$name_resource" $new_user_account "10,030000000000000000000000000000000000000000000000000004"
```
12. Call the lookup_address method one more time to see that the mapping has changed  
and that the name "satoshi.xrd" now points to the user's new account  
(02fbffedd2e0f3d0f3c5381b57b02c0f3b30bad1c57120f1c334bd).
```
resim call-method $component lookup_address satoshi.xrd
```
13. To simulate a renewal of the name mapping, call the renew_name method.  
The method must be called with the following parameters:  
1: A BucketRef with the DomainName NFT that demonstrates the user's ownership of the name and his right to change  
the mapped address (#339715316826500606461318410874891739268,$name_resource)  
2: The number of years for which the name should be renewed (10)  
3: A bucket that contains the fee for the name renewal (250,$tokenXRD)  
```
resim call-method $component renew_name "#339715316826500606461318410874891739268,$name_resource" 10 "250,030000000000000000000000000000000000000000000000000004"
```

14. Again, display the user's account and note that the name is now reserved until epoch 300000.  
Please also note that the DomainName NFT is still owned by the user's initial account even though the name
mapping points to the address of the user's new account. Ownership of a DomainName NFT is decoupled from
the actual address it maps to.
```
resim show $user_account
```

15. Finally, simulate that the user decides he now longer needs the domain name and wants to unregister it.  
This is done by calling the unregister_name method with a single argument.
This argument has to be a Bucket (not BucketRef) containing the DomainName NFT that should be unregistered
(#339715316826500606461318410874891739268,$name_resource).
In exchange for the DomainName NFT the user gets refunded his initial deposit of $XRD 500.
All other fees are kept by the RNS component.
```
resim call-method $component unregister_name "#339715316826500606461318410874891739268,$name_resource"
```

16. Display the user's account one last time.  
The NFT is gone and the users account holds exactly $XRD 999740. He initially deposited $XRD 500, which he got back,
but he payed another $XRD 10 to change the address and $XRD 250 to renew the name for 10 years.
```
resim show $user_account
```
