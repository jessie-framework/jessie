# Writing tests

In order to create a test, simply create a new file with a filetype in the `tests/` directory that our test suite currently supports (`.rs`,`.jessie`,etc.) : 

```sh
  touch tests/random/random_test.rs
```

Once you create the test, you have to specify what type of output you want from the compiler. In the case of Rust, you have to start your `.rs` file with a [shebang](https://doc.rust-lang.org/reference/shebang.html):

```rust
#!emit tokens
```

The test suite will read what is after the shebang and output to `.stderr` accordingly.

## Blessing tests

Tests do not have an `.stderr` file by deafult. Running the following command will update all `.stderr` files and create new ones in case one is missing:

```sh
  ./x bless
```

