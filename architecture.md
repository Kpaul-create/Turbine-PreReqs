# Turbine Pre-Req Vault — Architecture

## Overview

The Vault program lets a user deposit SOL into a PDA-owned vault and later withdraw it. As part of **withdraw**, the vault performs a **Cross-Program Invocation (CPI)** into the already-deployed Registration program (`q3_pre_reqs_rs`), which records the user's GitHub username.

Deployed cluster: **devnet**.

```
User (wallet)
  │
  ├─► Vault program (pre_req_vault)
  │     ├─ initialize   → creates vault_state + vault PDAs
  │     ├─ deposit      → transfers SOL user → vault
  │     ├─ withdraw     → transfers SOL vault → user
  │     │               └─ CPI → Registration program.initialize(github)
  │     └─ close        → returns all vault SOL + closes vault_state
  │
  └─► Registration program (q3_pre_reqs_rs, TRBQyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM)
        └─ initialize   → creates application_account PDA, records github
```

---

## Mermaid architecture diagram

```mermaid
flowchart TD
    %% Actors
    U((User / Wallet)):::actor
    DEVNET[("Devnet Cluster")]:::cluster

    %% Programs
    VP[/Vault Program<br/>pre_req_vault/<br/>DBoobRVq.../]:::program
    RP[/Registration Program<br/>q3_pre_reqs_rs<br/>TRBQyQHB.../]:::program

    %% Vault PDAs / accounts
    VS[(vault_state PDA<br/>seeds: ["state", user])]:::pda
    V[(vault PDA<br/>seeds: ["vault", vault_state])]:::pda

    %% Registration PDA / account
    AA[(application_account PDA<br/>seeds: ["prereqs", user]<br/>program: registration)]:::pda

    %% Vault instructions
    I1[initialize]:::ix
    I2[deposit]:::ix
    I3[withdraw]:::ix
    I4[close]:::ix

    %% CPI
    CPI((CPI<br/>initialize<br/>"Kpaul-create")):::cpi

    U -->|1. initialize| VP
    VP --> I1
    I1 -->|creates| VS
    I1 -->|derives| V

    U -->|2. deposit (SOL)| VP
    VP --> I2
    I2 -->|transfers SOL into| V

    U -->|3. withdraw (SOL + github)| VP
    VP --> I3
    I3 -->|transfers SOL out of| V
    I3 -->|4. CPI| CPI
    CPI -->|calls| RP
    RP -->|initializes & records github| AA

    U -->|5. close| VP
    VP --> I4
    I4 -->|returns remaining SOL + closes| V
    I4 -->|closes| VS

    %% Program deployment boundary
    subgraph Devnet [Devnet]
        VP
        RP
        VS
        V
        AA
    end

    classDef actor fill:#f96,stroke:#333,stroke-width:2px,color:#fff
    classDef cluster fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef program fill:#4caf50,stroke:#1b5e20,stroke-width:2px,color:#fff
    classDef pda fill:#fff9c4,stroke:#fbc02d,stroke-width:2px
    classDef ix fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    classDef cpi fill:#f3e5f5,stroke:#6a1b9a,stroke-width:2px,color:#fff,stroke-dasharray: 5 5
```

---

## Purpose of each instruction

| Instruction | Purpose | State change |
|-------------|---------|--------------|
| `initialize` | Creates the vault for a user. | Derives `vault_state` (seeds `["state", user]`) and `vault` (seeds `["vault", vault_state]`) PDAs; stores bumps in `vault_state`. |
| `deposit` | User deposits SOL into the vault. | Transfers lamports from `user` → `vault` PDA (vault signs via its PDA seeds). Vault balance increases. |
| `withdraw` | User withdraws SOL and registers. | Transfers lamports from `vault` → `user` (vault signs via PDA). **Then CPis into the Registration program's `initialize`**, passing `user`, `application_account`, `system_program`, and the GitHub username. The registration program creates/initializes `application_account` (seeds `["prereqs", user]`) and records the GitHub username. |
| `close` | Closes the vault, returning all SOL. | Transfers the entire vault balance to `user` and closes `vault_state` (rent refunded to user). |

## Accounts involved

**Vault program (`pre_req_vault`)**
- `user` — signer, the wallet owner.
- `vault_state` — `Account<VaultState>`, PDA `["state", user]`, stores `vault_bump` + `state_bump`.
- `vault` — `SystemAccount`, PDA `["vault", vault_state]`, holds the deposited SOL.
- `application_account` — `UncheckedAccount`, PDA `["prereqs", user]` under the registration program; initialized by the CPI.
- `application_program` — `Program<registration::program::Q3PreReqsRs>`, the registration program (`TRBQyQHB...`).
- `system_program` — Solana System Program.

**Registration program (`q3_pre_reqs_rs`)**
- `user` — signer.
- `account` (`application_account`) — PDA `["prereqs", user]`, stores `user`, `bump`, `pre_req_ts`, `pre_req_rs`, `github`.
- `system_program`.

## How program state changes over time

1. **Before `initialize`**: no vault state exists for the user.
2. **After `initialize`**: `vault_state` and `vault` PDAs exist; `vault_state` records both bumps; `vault` balance = 0.
3. **After `deposit`**: `vault` balance increases by the deposited amount.
4. **After `withdraw`**: `vault` balance decreases by the withdrawn amount; `user` receives the SOL; the registration program's `application_account` PDA is created and the GitHub username is recorded (`pre_req_rs = true`).
5. **After `close`**: remaining `vault` balance returns to `user`; `vault_state` is closed (rent refunded).

## Overall application flow

```
initialize → deposit → withdraw (with CPI to registration.initialize) → close
```

The CPI inside `withdraw` is the key extension: instead of the client calling the registration program directly, the vault program itself invokes it, so the registration record is created atomically as part of the withdrawal. The registration program (`TRBQyQHB...`) is already deployed on devnet and is **not** re-deployed here — only the vault program is deployed.

## GitHub username

The CPI passes the GitHub username `"Kpaul-create"` (the configured git user). To use a different username, edit the `GITHUB_USERNAME` constant in `programs/pre-req-vault/src/instructions/withdraw.rs`.

## Deployment & verification

- `anchor deploy` — deploys the vault program to devnet (the registration program is already deployed and is **not** re-deployed).
- `anchor test` — runs the TypeScript test suite (`initialize → deposit → withdraw → close`) against devnet.
- Success: after withdrawal, the registration program's `application_account` PDA (`["prereqs", user]`) exists and contains the GitHub username, confirmed via the CPI inner instruction (not a direct client call to the registration program).
