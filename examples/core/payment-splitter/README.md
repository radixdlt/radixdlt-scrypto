# Payment Splitter

Just like in the physical world, when multiple people come together to build an NFT or a DeFI project they typically want to split the profits from such project in a clear and trustworthy way. The approach to have one entity to trust with the funds to later send to other shareholders simply bears too much risk of one party going rogue and refusing to send the shareholders their stake. This calls for a better way for funds from a project to be managed and split across the different shareholders in the project. The `PaymentSplitter` is a Scrypto blueprint which uses NFTs to authenticate shareholders and allow shareholders to have a clear and trustworthy way of withdrawing their profits from a project when they wish to do so.

## Motivations

The motivation behind this blueprint is to build a payment splitter similar to Ethereum's OpenZeppelin [implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/finance/PaymentSplitter.sol) but to build it on Radix using all of the new and exciting concepts and ideas that Radix provides.

## Getting Started

If you wand to try out this blueprint there are three main ways to do that, the first is by using a new feature with Scrypto v0.3.0 which is the new transaction model and the transaction manifests, the second is by using an `example.py` file which runs all of the needed CLI commands for the example, and the third and final method is by typing in the commands manually. 

### Method 1: Using transaction manifest files

Transaction manifests is an extremely cool new feature introduced with v0.3.0 of scrypto which allows for transaction instructions to be written in an external `.rtm` file and run by resim. This feature allows for extremely powerful transactions to be created where multiple different actions take place in a single transaction. 

Lets begin by resetting the radix engine simulator by running the following command:

```shell
$ resim reset
```

We will need a number of accounts for the examples that we will be running. So, let's create four different accounts using the following command:

```shell
$ export op1=$(resim new-account)
$ export pub_key1=$(echo $op1 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address1=$(echo $op1 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
$ export op2=$(resim new-account)
$ export pub_key2=$(echo $op2 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address2=$(echo $op2 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
$ export op3=$(resim new-account)
$ export pub_key3=$(echo $op3 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address3=$(echo $op3 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
$ export op4=$(resim new-account)
$ export pub_key4=$(echo $op4 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address4=$(echo $op4 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
```

Now that we have created four different accounts, let's set the first account to be the default account by running the following command:

```shell
resim set-default-account $address1 $pub_key1
```

Let's now publish the package to our local radix engine simulator by running the following:

```shell
resim publish .
```

Now begins the exciting stuff with the new transaction model! The very first radix transaction manifest file that we will using is a file that creates a PaymentSplitter component on the local simulator. To run this file, run the following command:

```
$ resim run transactions/component_creation.rtm
```

What we would like to do now is to add shareholders to our PaymentSplitter component and to send the shareholders the NFTs that prove their identity. If we decide not to use the new transaction manifest feature we would have twice as many transaction as shareholders we wish to add (i.e. 10 shareholders = 20 transactions) as each shareholder would require a transaction for the minting of the NFT and another transaction for the sending of the NFT. However, with the new transaction model and the introduction of the transaction manifest, we can perform all of that in a single transaction! We have created a radix transaction manifest (rtm) file for you that performs the minting and sending of the tokens all in a single transaction! You may run it through the following command:

```shell
$ resim run ./transactions/adding_shareholders.rtm
```

After the above command runs you can check the balances of the four accounts that we have and you will see that each one of those accounts now have 1 shareholder NFT.

We would now like to test the PaymentSplitter to make sure that it does indeed do what we expect it to do. For that purpose we will deposit some funds into the payment splitter from account 1 and then we will withdraw them the share of the second account of the funds.

Let's run the transaction manifest file containing the instruction of the depositing of tokens through account 1.
```shell
$ resim run ./transactions/funding_the_splitter.rtm 
```

We may now switch to account 2 and try to withdraw the funds.

```shell
$ resim set-default-account $address2 $pub_key2
$ resim run ./transactions/withdrawing_owed_amount.rtm 
```

And that's it! Now if you check the balance of account 2 you would see that account 2 now has more XRD as a result of the splitting of profits made.

### Method 2: Automatic method using `example.py`.

The `example.py` file included with this blueprint is a quick python script written during the implementation of the blueprint to allow for a quick way for the package to be redeployed and for accounts to be created quickly and with ease.

When you run the `example.py` file the following will take place:

* Your resim will be reset so that any accounts previously created or packages deployed are deleted.
* Four new accounts will be created. The script will keep track of their addresses and public keys.
* The package will be published and the script will store the package address.
* The script will call the `new` function on the blueprint to create the PaymentSplitter component. When creating the component, a number of resource definitions will be created for the badges that blueprint creates.
* The script will then add all four of the addresses that we created as shareholders by calling the `add_shareholder` method on the PaymentSplitter component.
* Some XRD will be deposited by account 1 (by calling the `deposit_xrd` method) and then later account 2 will attempt to withdraw their share of it (by calling the `withdraw_xrd` method).

You do not need to install any additional packages to run this `example.py` file. This file used the standard python 3 library and packages. So, to run this example just type the following into your command line:

```shell
python3 example.py
```

### Method 3: Manual method using Resim and CLI.

This method is the typical method used to run Scrypto programs which is through the command line interface and the `resim` tool. In this section of the document we will go through a similar example to the one above using the resim CLI.

> Note that the addresses that I'm getting here will be different from yours.

First of all, let's begin by resetting our simulator just in case there are any packages already deployed.

```shell
$ resim reset
Data directory cleared.
```

Now that we've reset our simulator, let's create four different accounts to use for this example. In the codeblock below, we're creating the accounts and then saving them as environment variables as soon as creation is done
```shell
$ export op1=$(resim new-account)
$ export pub_key1=$(echo $op1 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address1=$(echo $op1 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
$ export op2=$(resim new-account)
$ export pub_key2=$(echo $op2 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address2=$(echo $op2 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
$ export op3=$(resim new-account)
$ export pub_key3=$(echo $op3 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address3=$(echo $op3 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
$ export op4=$(resim new-account)
$ export pub_key4=$(echo $op4 | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
$ export address4=$(echo $op4 | sed -nr "s/Account address: ([[:alnum:]_]+)/\1/p")
```

Let's check to make sure that the account creation has happened with no problems. To do that, let's try to echo out the account addresses

```shell
$ echo $address1
02c1897261516ff0597fded2b19bf2472ff97b2d791ea50bd02ab2
$ echo $address2
02d8541671ab09116ae450d468f91e5488a9b22c705d70dcfe9e09
$ echo $address3
02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b
$ echo $address4
02b9f7c0c44a6e2162403cea3fa44500dff50eb18fd4ff5a9dd079
```

With the four accounts created, let's now change our default account so that it's the first account that we created.

```shell
$ resim set-default-account $address1 $pub_key1
Default account set!
```

Alright, now we're finally ready to publish our package to the simulator.

```shell
$ resim publish .
Finished release [optimized] target(s) in 0.08s
Transaction Status: SUCCESS
Execution Time: 400 ms
Instructions:
├─ CallFunction { package_address: 010000000000000000000000000000000000000000000000000001, blueprint_name: "System", function: "publish_package", args: [LargeValue(len: 1792331)] }
└─ End { signers: [04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9] }
Results:
├─ Ok(Some(01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c))
└─ Ok(None)
Logs: 0
New Entities: 1
└─ Package: 01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c
$ export package=01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c
```

We may now use the package address to instantiate a new PaymentSplitter component.

```shell
$ resim call-function $package PaymentSplitter new
Finished release [optimized] target(s) in 0.03s
Transaction Status: SUCCESS
Execution Time: 400 ms
Instructions:
├─ CallFunction { package_address: 010000000000000000000000000000000000000000000000000001, blueprint_name: "System", function: "publish_package", args: [LargeValue(len: 1792331)] }
└─ End { signers: [04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9] }
Results:
├─ Ok(Some(01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c))
└─ Ok(None)
Logs: 0
New Entities: 1
└─ Package: 01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c
omarabdulla@Omars-MacBook-Pro payment_splitter % export package=01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c
omarabdulla@Omars-MacBook-Pro payment_splitter % resim call-function $package PaymentSplitter new
Transaction Status: SUCCESS
Execution Time: 17 ms
Instructions:
├─ CallFunction { package_address: 01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c, blueprint_name: "PaymentSplitter", function: "new", args: [] }
├─ DropAllBucketRefs
├─ DepositAllBuckets { account: 02c1897261516ff0597fded2b19bf2472ff97b2d791ea50bd02ab2 }
└─ End { signers: [04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9] }
Results:
├─ Ok(Some((023af09cc79097add03aa9614eadb005e61874681545a1ac2b8caf, Bid(1))))
├─ Ok(None)
├─ Ok(Some(()))
└─ Ok(None)
Logs: 0
New Entities: 4
├─ ResourceDef: 03c29248a0d4c7d4da7b323adfeb4b4fbe811868eb637725ebb7c1
├─ ResourceDef: 03fbffedd2e0f3d0f3c5381b57b02c0f3b30bad1c57120f1c334bd
└─ Component: 023af09cc79097add03aa9614eadb005e61874681545a1ac2b8caf
$ export adm=03c29248a0d4c7d4da7b323adfeb4b4fbe811868eb637725ebb7c1
$ export shb=03fbffedd2e0f3d0f3c5381b57b02c0f3b30bad1c57120f1c334bd
$ export component=023af09cc79097add03aa9614eadb005e61874681545a1ac2b8caf
```

Our payment splitter has now been created and initialized. We may now begin adding shareholders to our component. The following codeblock will add all four addresses to the shareholders list with a random amount of shares.

```shell
$ resim call-method $component add_shareholder $address1 $RANDOM 1,$adm
$ resim call-method $component add_shareholder $address2 $RANDOM 1,$adm
$ resim transfer "#00000000000000000000000000000001,$shb" $address2
$ resim call-method $component add_shareholder $address3 $RANDOM 1,$adm
$ resim transfer "#00000000000000000000000000000002,$shb" $address3
$ resim call-method $component add_shareholder $address4 $RANDOM 1,$adm
$ resim transfer "#00000000000000000000000000000003,$shb" $address4
```

Now we can finally do the exciting stuff! Let's now deposit some XRD using account 1 and then try to withdraw our share as account 2. Note that when account 1 deposits the money, the NFT metadata will get updated to reflect on the amount of XRD that each shareholder is owed.

```shell
$ export RADIX_TOKEN=030000000000000000000000000000000000000000000000000004
$ resim call-method $component deposit_xrd 100000,$RADIX_TOKEN
```

Lets now switch to account 2 and try to withdraw our share of the tokens in the PaymentSplitter

```shell
$ resim set-default-account $address2 $pub_key2
Default account set!
$ resim call-method $component withdraw_xrd 1,$shb
```

And we're done! Account 2 has now withdrawn their share of the total XRD stored in the Payment splitter. If they now try to withdraw again they will get an empty bucket of XRD. Only when another payment is received to the payment splitter can account 2 withdraw again.

## Future Work and Improvements

This is certainly a simple example of what a payment splitter might look like if built on Radix using Scrypto where NFTs are used for authentication and tracking of data. Of course, there are a number of improvements that can be made to this blueprint to make it function even better:

* Allowing for a `HashMap` to be passed to the function `new` so that the shareholders are added at the creation of the component (requires Scrypto v3.0.0 for this).
* Allowing shareholders to only withdraw a portion of their XRD instead of withdrawing it all when calling the `withdraw_xrd` method.
