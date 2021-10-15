## Crowdfunding with insurance

#### Assumptions
1. Component implements a fund of N units and defines their price
2. Component holds initial XRD for insurance
3. Component defines final time for selling all units
4. Unless all units are sold, component returns payments with a defined premium. Funds for a premium come from the insurance tokens.

#### Usage
1. Collect payments in exchange for units of a fund API
2. Return change if the amount was not exact
3. Return change if there was not enough tokens
4. Simulate progress of time with method tick
5. When current height/round/epoch matches final one check if all tokens are sold
6. Let owner to cash out 
7. [TODO] ...or return funds with premium 

## Console log

` $> rev2 publish . --address 05ba3e961b9abef8e533f68888f2460cebd302aaa838e8f2a74f37`
```
    Finished release [optimized] target(s) in 2.39s
New package: 05ba3e961b9abef8e533f68888f2460cebd302aaa838e8f2a74f37
```
` $> rev2 call-function 05ba3e961b9abef8e533f68888f2460cebd302aaa838e8f2a74f37 InsuredCrowdFund new`

```
Transaction Status: SUCCESS
Instructions:
├─ ReserveBuckets { n: 0 }
├─ CallFunction { package: 05ba3e961b9abef8e533f68888f2460cebd302aaa838e8f2a74f37, blueprint: "InsuredCrowdFund", function: "new", args: [] }
├─ DepositAll { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "deposit_batch" }
└─ Finalize
Results:
├─ Ok([])
├─ Ok([129, 27, 0, 0, 0, 6, 189, 113, 120, 15, 233, 230, 59, 7, 225, 215, 240, 181, 144, 54, 2, 36, 231, 62, 30, 192, 215, 163, 188, 140, 220, 200])
├─ Ok([])
└─ Ok([])
Logs: 0
New Addresses: 2
├─ Resource: 033e9f3549202530c1a640192019b12fa232e9ae11322daf8a4fad
└─ Component: 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8
Execution Time: 7 ms
```

`rev2 call-method 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8 get_current_height`

```
Transaction Status: SUCCESS
Instructions:
├─ ReserveBuckets { n: 0 }
├─ CallMethod { component: 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8, method: "get_current_height", args: [] }
├─ DepositAll { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "deposit_batch" }
└─ Finalize
Results:
├─ Ok([])
├─ Ok([9, 1, 0, 0, 0])
├─ Ok([])
└─ Ok([])
Logs: 1
└─ [INFO ] Current height is now: 1
New Addresses: 0
Execution Time: 4 ms
```

`$> rev2 call-method 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8 tick`

```
Transaction Status: SUCCESS
Instructions:
├─ ReserveBuckets { n: 0 }
├─ CallMethod { component: 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8, method: "tick", args: [] }
├─ DepositAll { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "deposit_batch" }
└─ Finalize
Results:
├─ Ok([])
├─ Ok([0])
├─ Ok([])
└─ Ok([])
Logs: 1
└─ [INFO ] Current height is now: 2
New Addresses: 0
Execution Time: 3 ms

```

`$> rev2 call-method 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8 buy_fund_units '400,01'`

```
Transaction Status: SUCCESS
Instructions:
├─ ReserveBuckets { n: 1 }
├─ CallMethod { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "withdraw", args: [[128, 32, 0, 0, 0, 144, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [129, 1, 0, 0, 0, 1]] }
├─ NewBucket { offset: 0, amount: 400, resource: 01 }
├─ CallMethod { component: 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8, method: "buy_fund_units", args: [[20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 0, 0, 0, 0]] }
├─ DepositAll { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "deposit_batch" }
└─ Finalize
Results:
├─ Ok([])
├─ Ok([20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 1, 0, 0, 0])
├─ Ok([])
├─ Ok([19, 2, 0, 0, 0, 16, 1, 20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 4, 0, 0, 0, 16, 1, 20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 0, 0, 0, 0])
├─ Ok([0])
└─ Ok([])
Logs: 4
├─ [INFO ] There is 100 units left in the fund
├─ [INFO ] Compontent received 400 xrd
├─ [INFO ] Current value of exact_payment: 400, change: 0, final_units: 2
└─ [INFO ] Current value of collected_xrd: 400, fund_units: 98
New Addresses: 0
Execution Time: 13 ms
```

` $> rev2 call-method 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8 buy_fund_units '777,01'`

```
Transaction Status: SUCCESS
Instructions:
├─ ReserveBuckets { n: 1 }
├─ CallMethod { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "withdraw", args: [[128, 32, 0, 0, 0, 9, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [129, 1, 0, 0, 0, 1]] }
├─ NewBucket { offset: 0, amount: 777, resource: 01 }
├─ CallMethod { component: 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8, method: "buy_fund_units", args: [[20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 0, 0, 0, 0]] }
├─ DepositAll { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "deposit_batch" }
└─ Finalize
Results:
├─ Ok([])
├─ Ok([20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 1, 0, 0, 0])
├─ Ok([])
├─ Ok([19, 2, 0, 0, 0, 16, 1, 20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 4, 0, 0, 0, 16, 1, 20, 22, 1, 0, 0, 0, 131, 4, 0, 0, 0, 0, 0, 0, 0])
├─ Ok([0])
└─ Ok([])
Logs: 4
├─ [INFO ] There is 98 units left in the fund
├─ [INFO ] Compontent received 777 xrd
├─ [INFO ] Current value of exact_payment: 600, change: 177, final_units: 3
└─ [INFO ] Current value of collected_xrd: 1000, fund_units: 95
New Addresses: 0
Execution Time: 9 ms
```

` $>  rev2 call-method 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8 tick`
...

` $>  rev2 call-method 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8 tick`

```
Transaction Status: SUCCESS
Instructions:
├─ ReserveBuckets { n: 0 }
├─ CallMethod { component: 06bd71780fe9e63b07e1d7f0b590360224e73e1ec0d7a3bc8cdcc8, method: "tick", args: [] }
├─ DepositAll { component: 0631a41f7d669e86609b04f32633b6fb11590734bd2309032f48ee, method: "deposit_batch" }
└─ Finalize
Results:
├─ Ok([])
├─ Ok([0])
├─ Ok([])
└─ Ok([])
Logs: 2
├─ [INFO ] The time is up! is_open: false, is_success: false
└─ [INFO ] Current height is now: 5
New Addresses: 0
Execution Time: 6 ms
```


