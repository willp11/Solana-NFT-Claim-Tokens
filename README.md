# Claim Tokens with NFT Program

This is a Solana program that allows projects to airdrop tokens to holders of a specified NFT project.

The UI for interacting with this program can be found at (https://github.com/willp11/Solana-claim-tokens-ui).

### Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool

### Build and test for program compiled natively
```
$ cargo build
$ cargo test
```

### Build and test the program compiled for BPF
```
$ cargo build-bpf
$ cargo test-bpf
```
