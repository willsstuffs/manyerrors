manyerrors
============
[<img alt="docs" src="https://img.shields.io/docsrs/manyerrrors?logo=docs.rs">](https://docs.rs/manyerrors/)
[<img alt="crates" src="https://img.shields.io/crates/v/manyerrors?logo=rust
">](https://crates.io/manyerrors/)
[<img alt="github" src="https://img.shields.io/badge/github-willsstuffs/manyerrors-cyan?logo=github
">](https://github.com/willsstuffs/manyerrors/)

A library for when many errors may occur at one time, but you don't want to short circuit on
the first error found.

```rust
use manyerrors::{manyerrors,stash,Errors,err}

#[manyerrors(String)]
fn my_fn(a: i32, b: i32) -> Result<i32,Errors<String>> {
    stash!(some_other_fallible_fn());

    if b == 0 {
        return Err(err(String::from("Cant divide by zero!")));
    }

    Ok(a/b)
} 

```

For easy incorporation of the [`anyhow`] crate, use the crate feature `anyhow`.

[`anyhow`]: https://crates.io/anyhow