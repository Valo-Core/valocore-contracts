# 📜 Valo-Core Protocol Contracts

The secure, immutable core of the Valo-Core ecosystem. Built with Rust for the Soroban Smart Contract Platform, these contracts manage decentralized milestone tracking, programmatic bounty escrows, and overflow-safe on-chain asset distribution.

## 🗺️ Ecosystem System Architecture

                   +---------------------------+
                   |   GitHub Webhook Events   |
                   +-------------+-------------+
                                 |
                                 v
                   +-------------+-------------+
                   |      valocore-relayer     |
                   +-------------+-------------+
                                 |
                                 v (Verified State Changes)
+--------------------------+   +-----+-----+   +---------------------------+
|    valocore-contracts    |-->| Smart State |<--|     valocore-dashboard    |
|   (You are here: Escrow) |   +-----------+   |  (Obsidian & Gold Engine) |
+--------------------------+                   +---------------------------+


---

## 🔒 On-Chain Security Features

- **Checked Arithmetic Primitives:** All token math computations utilize strict Rust checked execution bounds (`checked_add`, `checked_div`, `checked_mul`) ensuring the protocol is naturally immune to integer manipulation hacks.
- **Instance State Isolation:** Utilizes Soroban instance storage routines to prevent state collision vulnerabilities across milestone entries.

## 🛠️ Compilation and Testing

### Prerequisites
Install the target architecture compilation tools:
```powershell
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli
