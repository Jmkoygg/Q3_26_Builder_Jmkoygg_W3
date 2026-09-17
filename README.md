# Turbin3 Q3 2026 - Week 3 Assignment: Automated Market Maker (AMM)

Comprehensive implementation for the **Turbin3 Builders - Week 3 Assignment (AMMs)** on Solana using the **Anchor framework**.

This repository implements a full-featured **Automated Market Maker (AMM)** following the Constant Product formula ($X \times Y = K$). It includes liquidity provision, token swaps with slippage protection, a protocol **Treasury (PDA)** with automated fee collection, administrative emergency controls (`lock` / `unlock`), and a high-performance **LiteSVM** test suite covering 100% of the instructions.

---

## 📋 Mandatory Tasks Overview

| Task | Description | Status |
|---|---|---|
| **1. Write the AMM program** | Core AMM instructions: `initialize`, `deposit`, `withdraw`, `swap`, and emergency controls (`lock`/`unlock`). | ✅ Completed |
| **2. Add fees and treasury account** | Configurable trading fee in basis points, automated fee routing to dedicated Treasury token accounts (PDA-owned), and an administrative `withdraw_fees` instruction. | ✅ Completed |
| **3. Write tests covering all the instructions** | Comprehensive integration tests using `LiteSVM`, validating all instructions and edge cases in under 1 second. | ✅ Completed |

---

## 🏛️ Architecture & Account Structure

In Solana, smart contracts are stateless programs that operate on data accounts. This AMM utilizes deterministic Program Derived Addresses (PDAs) to custody funds and enforce permissions:

```
                  ┌──────────────────────────────────┐
                  │          Initializer             │
                  └────────────────┬─────────────────┘
                                   │ initialize
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                          Pool Config (PDA)                             │
│                  seeds = [b"config", seed.to_le_bytes()]               │
│  - mint_x: Pubkey                                                      │
│  - mint_y: Pubkey                                                      │
│  - fee: u16 (basis points)                                             │
│  - locked: bool                                                        │
│  - authority: Option<Pubkey>                                           │
└──────┬───────────────────────┬─────────────────────────┬───────────────┘
       │ authority             │ authority               │ mint authority
       ▼                       ▼                         ▼
┌──────────────┐        ┌──────────────┐         ┌──────────────┐
│   Vault X    │        │   Vault Y    │         │   LP Mint    │
│    (ATA)     │        │    (ATA)     │         │    (PDA)     │
└──────────────┘        └──────────────┘         └──────────────┘

                                   │ linked seed
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        Treasury Authority (PDA)                        │
│                  seeds = [b"treasury", config.key()]                   │
└──────────────────────┬─────────────────────────────────────────────────┘
                       │ authority
          ┌────────────┴────────────┐
          ▼                         ▼
   ┌──────────────┐          ┌──────────────┐
   │  Treasury X  │          │  Treasury Y  │
   │    (ATA)     │          │    (ATA)     │
   └──────────────┘          └──────────────┘
```

### Account Breakdown
- **`config` (PDA)**: Holds the pool parameters: mints $X$ and $Y$, swap fee (bps), lock state, authority, and bump seeds.
- **`vault_x` & `vault_y` (ATAs)**: Custody accounts for pool liquidity, owned by the `config` PDA.
- **`mint_lp` (Mint PDA)**: SPL Token mint representing fractional shares of the liquidity pool.
- **`treasury` (PDA)**: Keyless program-derived authority governing protocol fee revenue.
- **`treasury_x` & `treasury_y` (ATAs)**: Token accounts designated for collecting fees in Token X and Token Y, owned by the `treasury` PDA.

---

## ⚙️ Instruction Specifications

### 1. `initialize`
- Seeds the `config` PDA with pool parameters and trading fee (e.g., `30` bps = 0.3%).
- Initializes the LP Token Mint (`seeds = [b"lp", config.key()]`).
- Initializes token vaults (`vault_x`, `vault_y`) with authority `config`.
- Initializes protocol treasury token accounts (`treasury_x`, `treasury_y`) with authority `treasury`.

### 2. `deposit`
- Allows Liquidity Providers (LPs) to deposit Token X and Token Y.
- Enforces pool lock checks (`!config.locked`).
- Emits newly minted LP tokens to the provider based on Constant Product math.

### 3. `swap`
- Executes a trade between Token X and Token Y using the constant product curve invariant:
  $$X \times Y = K$$
- Computes swap fee: $\text{fee} = \text{amount\_in} \times \frac{\text{fee\_bps}}{10,000}$.
- Splits incoming tokens:
  1. **Net Deposit** ($\text{amount\_in} - \text{fee}$) is transferred from user to the **Pool Vault**.
  2. **Fee Amount** ($\text{fee}$) is transferred directly from user to the **Treasury ATA** (`treasury_x` or `treasury_y`).
- Transfers output tokens from the opposing vault to the user with slippage control (`min_amount_out`).

### 4. `withdraw`
- Burns user's LP tokens and calculates proportional shares of Token X and Token Y.
- Transfers proportional reserves from the vaults back to the user.

### 5. `withdraw_fees`
- Allows the designated `authority` to withdraw accumulated protocol fees from `treasury_x` and `treasury_y`.
- Uses CPI with PDA signer seeds (`&[&[b"treasury", config.key().as_ref(), &[config.treasury_bump]]]`).

### 6. `lock` & `unlock`
- Administrative circuit breaker allowing the `authority` to halt deposits, withdrawals, and swaps during extreme volatility or upgrades.

---

## 🧪 Automated Testing Suite (LiteSVM)

The program is thoroughly tested using **`LiteSVM`** (Solana's lightweight in-memory SVM runtime), avoiding the overhead and flakiness of spinning up a full test validator.

### Running Tests

```bash
cargo test
```

### Test Results

```text
running 6 tests
test test_initialize ... ok
test test_deposit ... ok
test test_swap_and_fee_to_treasury ... ok
test test_withdraw ... ok
test test_withdraw_fees ... ok
test test_lock_and_unlock_pool ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
```

### Test Coverage Highlights
1. **`test_initialize`**: Verifies deterministic PDA derivation, account creation, and initial state layout.
2. **`test_deposit`**: Tests initial liquidity seeding and verifies vault balances.
3. **`test_swap_and_fee_to_treasury`**: Swaps 10 tokens with a 0.3% fee (30 bps), verifying that exactly 30,000 units are deposited into the Treasury and the remainder enters the vault.
4. **`test_withdraw`**: Burns LP tokens and verifies proportional asset redemption.
5. **`test_withdraw_fees`**: Verifies that the pool authority successfully claims accumulated treasury fees via PDA signer seeds.
6. **`test_lock_and_unlock_pool`**: Asserts that swaps revert with `AmmError::PoolLocked` while locked, and resume successfully once unlocked.

---

## 💡 Quick Presentation Guide / Evaluator FAQs

- **Why use a PDA for the Treasury instead of a standard wallet?**
  > *A PDA has no private key and is programmatically owned. Fee withdrawals can only occur under strict contract logic (via the `withdraw_fees` instruction verifying `authority`), making protocol revenue trust-minimized and auditable.*

- **How are fees split during swaps?**
  > *Fees are calculated in basis points (e.g. 30 bps = 0.3%). When swapping, incoming tokens are split: the net amount enters the liquidity vault to maintain the invariant curve $X \times Y = K$, while the fee is routed directly to the treasury token account.*

- **Why use LiteSVM instead of standard TypeScript tests?**
  > *LiteSVM executes the SBF program directly inside the Rust test runner's memory space. It is deterministic, fast (executing all 6 end-to-end integration tests in ~0.66s), and avoids validator setup overhead.*
