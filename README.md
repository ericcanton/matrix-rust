# Matrix [![Version][version-img]][version-url] [![Status][status-img]][status-url]

The package provides a matrix laboratory.

## [Documentation][documentation]

## Example

```rust
#[macro_use]
extern crate matrix;

use matrix::prelude::*;

let mut sparse = Compressed::zero((2, 4));
sparse.set((0, 0), 42.0);
sparse.set((1, 3), 69.0);

let dense = Conventional::from(&sparse);
assert_eq!(
    &*dense,
    &*matrix![
        42.0, 0.0, 0.0,  0.0;
         0.0, 0.0, 0.0, 69.0;
    ]
);
```

## Contribution

Your contribution is highly appreciated. Do not hesitate to open an issue or a
pull request. Note that any contribution submitted for inclusion in the project
will be licensed according to the terms given in [LICENSE.md](LICENSE.md).

[documentation]: https://docs.rs/matrix
[status-img]: https://travis-ci.org/stainless-steel/matrix.svg?branch=master
[status-url]: https://travis-ci.org/stainless-steel/matrix
[version-img]: https://img.shields.io/crates/v/matrix.svg
[version-url]: https://crates.io/crates/matrix
