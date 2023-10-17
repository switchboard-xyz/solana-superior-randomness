# Solana SGX-Verified Randomness

This repository demonstrates a robust method of generating randomness on Solana using the Switchboard oracle, leveraging a Trusted Execution Environment (TEE) provided by SGX.

## Overview

Traditional methods of randomness generation are prone to vulnerabilities where oracles could manipulate or withhold random outputs. Our enhanced approach mitigates these issues by decentralizing the randomness derivation process.

Instead of relying solely on the oracle for the final random number, we incorporate user involvement:

1. A user triggers a request with a unique, non-repeatable pubkey.
2. The oracle, running in an SGX environment (ensuring genuine randomness and code integrity), picks a random seed.
3. The oracle reports the seed on-chain.
4. Users derive the final random number using `sha256(ed25519_sign(privateKey, seed))`.

## Why SGX?

SGX (Software Guard Extensions) provides a Trusted Execution Environment (TEE). When a binary runs inside SGX, its integrity and output are verifiable. In our case, this ensures the genuine randomness of the seed provided by the Switchboard oracle.

## Steps

### 1. User-Derived Non-Repeatable PubKey

Users initiate the randomness request using a unique pubkey. This ensures each request is distinct, preventing any pre-computation attacks or replay vulnerabilities.

### 2. SGX-Verified Oracle Seed

Upon detecting a request, the Switchboard oracle derives a random seed inside the SGX TEE. The seed's randomness is genuine and verifiable thanks to the SGX environment.

### 3. On-Chain Seed Reporting

The derived seed is then reported on-chain, ensuring transparency and traceability. This step also guarantees that once a seed is associated with a request, it cannot be altered or withheld by the oracle.

### 4. User-Derived Final Random Number

Users, with their private key and the reported seed, can then generate the final random number. This ensures that the random number is both verifiable and unpredictable, as it's derived from the combination of a secret (private key) and a publicly known value (seed).

## Security Considerations

- **Private Key Security**: It's crucial for users to safeguard their private keys. Exposure would allow the recreation of the random number by anyone with seed knowledge.

- **Oracle Trust**: While the SGX provides a secure environment, always ensure you're interfacing with a reputable and verified oracle service.

- **Unique PubKey**: The mechanism generating non-repeatable pubkeys must be foolproof to prevent any potential gaming of the system.

With these security aspects in mind, let's delve into the specific functions provided by Switchboard.

## Switchboard Functions

Switchboard Functions allow you to **_"callback"_** into your program with some arbitrary instruction. This means within your function you can make network calls to off-chain resources and determine which instruction on your program to respond with.

Switchboard oracles will read the emitted partially signed transaction, verify the code was executed within a Trusted Execution Environment (TEE), and relay the transaction on-chain. Switchboard oracles read a generated "quote" from the codes runtime when you emit the partially signed transaction from your container. This code is unique to the generated executable and relevant OS files. Any time you change the code or a dependency, your quote will change and you will need to update your Switchboard Function config. These quotes are known as `MrEnclaves` and represent a fingerprint of the code and the runtime. Within your Switchboard Function, you define a whitelist of MrEnclaves that are allowed to perform some action on your contract's behalf. **!! Make sure you validate the Switchboard accounts, as seen below with:** `switchboard_request.validate_signer()`

```rust
#[derive(Accounts)]
pub struct Settle<'info> {
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    #[account(
        constraint = switchboard_request.validate_signer(
            &switchboard_function.to_account_info(),
            &enclave_signer.to_account_info()
            )?
        )]
    pub switchboard_request: Box<Account<'info, FunctionRequestAccountData>>,
    pub enclave_signer: Signer<'info>,
}
```

## Publishing a Switchboard Function

Start by copying the env file to set your environment. To begin, you can use the default container for your program. When you're ready, you can make changes to the Switchboard Function and deploy to your own DockerHub organization.

```bash
echo 'DOCKER_IMAGE_NAME=mgild/superior-randomness' > .env
```

### Deployment and Execution

Set the anchor program IDs to your local keypairs so you can deploy this yourself:

```bash
anchor keys sync
```

Then deploy the contract with:

```bash
make superior-flip-deploy
# OR
anchor build -p superior_randomness
anchor deploy --provider.cluster devnet -p superior_randomness
```

Submit a guess and await the result:

```bash
make superior-flip
# OR
anchor run superior-flip
```


