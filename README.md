# Wu-Pay SPL

## Transfer SPL(WSOL) token via third party escrow wallet. Demonstrated use of PDA and SPL token. 

This source code is primarily based on follwing online tutorial:

- https://betterprogramming.pub/using-pdas-and-spl-token-in-anchor-and-solana-df05c57ccd04
- https://101blockchains.com/transfer-sol-and-spl-tokens-using-anchor/

It is inteded solely for educational purpose and not for commercial use. The use of this code is subject to the terms and conditions as outlined by the original source material from their website. The author of this code bears no liability for any unintended consequences that arise from its use.


## Enviornment setup

### Install Anchor
- https://www.anchor-lang.com/docs/installation

### Solana enviornment configuration
```shell 
solana-install init 1.16.25
```

### Install SPL Typescript lib
  ```shell
  npm install @solana/spl-token
  ```

### Make sure the Cargo.toml in programs/wu-pay-spl/Cargo.toml including;
```[dependencies]
anchor-lang = "0.28.0"
anchor-spl = "0.28.0"
solana-program = "1.16.0"
ahash = "=0.8.6"
```

## Run Test

Run test on local solana validator. Keep validator running after the test case is terminated. 
```shell
anchor test --detach
```
View validator logs
```shell
solana logs
```
