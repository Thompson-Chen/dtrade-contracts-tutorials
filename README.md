# ink! contracts
The repo contains the contracts written on ink!

### How to use:
Set up rust using the [installtion guide](https://doc.rust-lang.org/cargo/getting-started/installation.html). Set up the cargo contract cli tool using:

```
cargo install cargo-contract --force
```

To test out the example ballot contract, run:
```
cd examples/ballot
cargo +nightly contract build // this compiles the contract and generates wasm file
cargo +nightly contract test // to execute contract tests
```
