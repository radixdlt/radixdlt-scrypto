# Composing RadiSwap with GumballMachine
In this example, you will compose a simple transaction that is interacting with multiple components. We will first do a swap from TKN to XRD using the RadiSwap component then buy gumballs with the XRD we got.

## How to test
1. **Reset your environment**: `resim reset`
2. Instantiate an account: `resim new-account`
3. Create a TKN token definition: `resim new-token-fixed --symbol TKN 1000`
4. Go into the RadiSwap directory: `cd ../../../examples/defi/radiswap`
5. Publish and instantiate the RadiSwap component: `resim publish .` and `resim call-function 01d527faee6d0b91e7c1bab500c6a986e5777a25d704acc288d542 Radiswap new 1000,$xrd 10,$tkn 1000 "LP" "LP" "
   " 0`
6. Go into the GumballMachine blueprint directory: `cd ../../core/gumball-machine`
7. Publish and instantiate the component: `resim publish .` and `resim call-function 01afe53b9981dddb1b4ae44ff09deeae68f5bf42b360d88eaeb077 GumballMachine new 10`

Now that the two components are setup, you can call the transaction manifest file:

`cd ../../../transaction-manifest/examples/radiswap-gumball`

`resim run call.rtm`

You can look at the resources of your account: `resim show 0293c502780e23621475989d707cd8128e4506362e5fed6ac0c00a`. 
You should have 1 GUM.