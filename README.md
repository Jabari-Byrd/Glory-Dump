# GLORY/DUMP

> **How Not to Do Money** — a Solana PvP strategy game where the lowest weighted DUMP burden wins.

GLORY/DUMP is deliberately backwards:

- **DUMP** is an internal game burden. It cannot be bought, sold, bridged, or transferred outside an active match.
- **GLORY** is the ordinary six-decimal SPL reward token. It can move through wallets and external Solana markets like another fungible token.
- Giving DUMP away improves your position and hurts the recipient.
- ABSORB takes a rival's burden onto yourself in exchange for temporary defensive Guard.
- REDIRECT spends Guard to ricochet an incoming attack back to its sender.

This repository is the v3 Solana/Anchor rewrite. The previous Solidity/Base prototype and its placeholder bridge, oracle, buyback, and fee machinery have been removed rather than carried into a new chain with old assumptions.

## Status

The game rules, Anchor instruction surface, generated IDL, wallet client, playable demo, and Strategy Room interface are implemented. Host Rust tests, strict TypeScript checks, frontend tests, IDL generation, Clippy, the production web build, and an optimized SBF build under Solana 4.1.2 pass. A loopback validator bootstrap smoke also loaded the program and created the protocol, first epoch, leaderboard, and six-decimal GLORY mint through the real Token Program.

It is still **experimental, unaudited, and not mainnet-ready**. The full local-validator lifecycle/failure suite, adversarial economic simulation, independent security audit, and legal review remain release gates. A successful build and bootstrap transaction are not evidence that GLORY has investment value or that the game economy is manipulation-resistant.

## The game loop

```text
7-day registration
  -> commit a private random secret + post a 0.002 SOL activity bond
12-hour reveal
  -> reveal the secret; withholding means 10B DUMP and no ranking eligibility
1-hour strategy window
  -> claim a deterministic 1B–10B starting burden and inspect the field
30-day active epoch
  -> DUMP, ABSORB, and arm REDIRECT across four independent balance lanes
permissionless settlement
  -> one player per transaction; each call earns a fixed SOL bounty
claims and cleanup
  -> winners claim GLORY, active players reclaim 80% of the bond, rent is recoverable
```

At least 20 players are required to open a competitive epoch. Only players who reveal and complete at least one meaningful action can rank or receive an activity-bond refund. If registration or reveal participation is insufficient, anyone can cancel the epoch and every registrant can recover the full bond.

### Weighted score

The objective is the lowest balance over time, not the lowest last block. Balance weight rises smoothly from `1x` at the opening to `4x` at the deadline:

```text
weight(t) = 1 + 3(t / epochDuration)^2
score     = integral(balance(t) * weight(t)) / integral(weight(t))
```

Late play matters, but it does not erase the month. Holding 1B DUMP for 99% of the epoch and then reaching zero still scores about 980M.

### Three actions

| Action | Strategic role | Constraint |
|---|---|---|
| `DUMP` | Move unlocked burden to a rival's deterministic lane. | DUMP Heat scales with the amount relative to your starting allocation. |
| `ABSORB` | Pull burden from a rival and convert 50% into Guard. | The absorbed DUMP locks for six hours; Guard is capped at 25% of starting allocation. |
| `REDIRECT` | Arm one lane and return only the portion of one incoming DUMP covered by its Guard. | Overflow always lands; used Guard is consumed and the lane disarms for at least 15 minutes. |

DUMP and ABSORB have independent Heat channels. Each can move roughly one starting allocation per six-hour recovery cycle. A meaningful action is at least 0.1% of the player's starting allocation, which keeps dust spam from deciding eligibility or tie-breaks.

There is intentionally no automatic “victim cooldown boost.” A player could manufacture incoming attacks with Sybil wallets and turn such a boost into a last-minute escape. Counterplay instead comes from explicit, lossy ABSORB/Guard/REDIRECT decisions.

Guard and armed-lane state are deliberately public in this version. Solana account data is public, so hiding those fields only in the Strategy Room would disadvantage human players while direct RPC clients and bots could still inspect them. A genuinely sealed ricochet would require a different protocol: an opaque precommitment plus delayed attack settlement and a reveal window, or an audited privacy/randomness dependency. That extra latency and liveness risk is not silently mixed into the fast-action baseline.

## Ranking and GLORY

The lowest 5% of eligible players win, rounded up and capped at 128 winners in a 2,560-player room. Exact score ties prefer:

1. more capped impact across actions;
2. more distinct opponents;
3. a seed-committed random tie-breaker;
4. the public key as a final deterministic fallback.

GLORY emission begins at 100,000 per epoch for 100 eligible players, scales with the square root of participation from `0.25x` to `4x`, and halves every 12 epochs. One percent is reserved for settlement callers. The mint has a hard 10,000,000 GLORY cap, no deployer allocation, and no program instruction that can mint outside committed epoch claims.

GLORY does not have an embedded price, treasury, presale, buyback, or DEX. Once rewards circulate, holders may create external markets without giving those markets control over DUMP gameplay.

## Solana architecture

```text
Protocol PDA ── controls the capped GLORY mint
     │
     └── Epoch PDA ── phase clock, entropy, reward and bond accounting
           ├── Leaderboard PDA ── bounded top-128 heap
           ├── PlayerEpoch PDA ── eligibility, Heat, activity, session authority
           │      └── four BalanceLane PDAs ── burden, score accumulator, Guard
           ├── ordered Rivalry PDAs ── per-opponent impact cap and target-lane nonce
           └── KeeperCredit PDAs ── pull-based settlement GLORY
```

Four lane accounts let unrelated attacks on one player land concurrently when Solana schedules them against different writable accounts. Population-dependent work is split into player-created registration accounts, one-player settlement calls, bounded winner storage, and pull claims. Historical feeds are emitted as events and belong in an indexer; they are not stored forever in one global account.

See [game design](docs/GAME_DESIGN.md), [architecture](docs/ARCHITECTURE.md), [rewrite notes](docs/V3_REWRITE.md), and [testing gates](docs/TESTING.md) for the full contracts.

## Strategy Room

The frontend is a responsive strategy interface rather than an address textbox. It includes:

- a clickable inverse territory atlas;
- projected standings, burden, Heat, four lanes, Guard, locks, and badges;
- exact action previews and human inputs such as `250M` or `1.2B`;
- commitment-secret download and verified restore;
- permissionless phase, settlement, reward, and bond controls;
- a local playable demo when no program ID is configured;
- lazy loading of the larger wallet/Anchor client so demo boot remains small.

```bash
pnpm install
pnpm run dev
```

Copy `frontend/.env.example` to `frontend/.env` only after deploying a matching program. Without `VITE_PROGRAM_ID`, the site intentionally stays in demo mode.

## Development

Pinned prerequisites:

- Node.js 22.12 or newer
- pnpm 11.19
- Rust 1.89 or newer
- Anchor CLI 1.2.0
- Solana CLI 4.1.2 for SBF builds and validator tests

```bash
pnpm install
pnpm run check
```

Useful focused commands:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm run check:frontend
pnpm run idl
```

The generated TypeScript and JSON IDL files under `frontend/src/idl/` are committed so the wallet client is bound to the reviewed instruction schema. Regenerate them after any program interface change.

Deployment is deliberately a separate, fail-loud procedure; see [DEPLOYMENT.md](DEPLOYMENT.md).

## Mainnet hold

Do not deploy this version with real-value expectations. Before mainnet consideration:

1. replace the placeholder program ID, synchronize every artifact, and reproduce the SBF build;
2. pass local-validator lifecycle, claim, cleanup, and failure-path tests;
3. run the planned bot/Sybil/collusion simulations and retune parameters from evidence;
4. obtain independent Solana-program and frontend audits;
5. resolve commit/reveal selective-withholding risk;
6. test RPC/indexer behavior and account contention at intended room size;
7. publish incident response and private disclosure procedures;
8. obtain jurisdiction-specific legal review for entry bonds, prizes, and public trading.

## License

MIT. Use at your own risk.
