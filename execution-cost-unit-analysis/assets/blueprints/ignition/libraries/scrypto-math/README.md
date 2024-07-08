# scrypto_math

## Why
Radix Scrypto currently is lacking more advanced mathematical operations like `exp`, `log` or `pow`.

`scrypto_math` aims to provide an alternative until these functionalities are provided upstream. The ultimate goal of `scrypto_math` however is to make itself obsolete.

## Usage
Add `scrypto_math` to your depdencies in the `Cargo.toml` of your Scrypto blueprint.
```rust
[dependencies]
scrypto_math = { git = "https://github.com/ociswap/scrypto-math", tag = "v0.4.0" }
```
Import the module:
```rust
use scrypto_math::*;
```

## Featues

### Exponential Function
The exponential function is provided for `Decimal` and `PreciseDecimal` with a polynomial approximation error lower than ~ 18 significant digits.
Background: the final result is calculated via `exp(x) = 2**k * R(r)` and the approximation `R(r)` is bound by an maximum error of `2**-59` (~ 18 decimal places).

For `Decimal`:
```rust
let exp: Option<Decimal> = dec!(4).exp();
```

For `PreciseDecimal`:
```rust
let exp: Option<PreciseDecimal> = pdec!(4).exp();
```

You can see a full blueprint example including tests here [AdvancedMathDemo](examples/advanced_math/src/lib.rs).

### Logarithm Function
Logarithm is available for `Decimal` and `PreciseDecimal` with a maximum polynomial approximation error bound by `2**-58.45` (~ 18 decimal places).

For `Decimal`:
```rust
let ln: Option<Decimal> = dec!(2).ln();
let log2: Option<Decimal> = dec!(3).log2();
let log10: Option<Decimal> = dec!(4).log10();
let log8: Option<Decimal> = dec!(5).log_base(base: dec!(8));
```

For `PreciseDecimal`:
```rust
let ln: Option<PreciseDecimal> = pdec!(2).ln();
let log2: Option<PreciseDecimal> = pdec!(3).log2();
let log10: Option<PreciseDecimal> = pdec!(4).log10();
let log8: Option<PreciseDecimal> = pdec!(5).log_base(base: pdec!(8));
```

You can see a full blueprint example including tests here [AdvancedMathDemo](examples/advanced_math/src/lib.rs).

### Power Function
The power function internally uses both `exp` and `ln` and also covers various special cases like `0**0` or `-2**3`.

For `Decimal`:
```rust
let pow: Option<Decimal> = dec!("3.14").pow("-14.12");
```

For `PreciseDecimal`:
```rust
let pow: Option<PreciseDecimal> = pdec!("3.14").pow("-45.97");
```

You can see a full blueprint example including tests here [AdvancedMathDemo](examples/advanced_math/src/lib.rs).

## Contributions
We are happy to collaborate and review and merge pull requests :)

## Disclaimer
Though covered by an extensive test suite, use at your own risk.
