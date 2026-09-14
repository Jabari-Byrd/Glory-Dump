# Solana architecture

## Why the rewrite is native

The v3 system is not Solidity translated line by line. Solana's scheduling model rewards small explicit accounts and parallel writable sets; Ethereum's global contract storage encourages a different shape. The rewrite separates immutable math, on-chain authority, and presentation:

```text
glory-dump-core (no_std Rust)
    deterministic scoring, Heat, Guard, rewards, ordering, badges
                      │
                      v
Anchor program (25 instructions)
    validates accounts/phases, moves lamports, mints committed GLORY, emits events
                      │
               generated IDL
                      │
                      v
TypeScript Strategy Room
    wallet transactions, projected state, inverse atlas, action previews
```

The core crate has no Anchor or RPC dependency. Rule tests execute quickly on the host; the same functions are linked into the program.

## Asset boundary

### DUMP

DUMP is represented by `u64` balances inside program-owned `BalanceLane` PDAs. It has no mint account, decimals, wallet balance, token program, allowance, transfer hook, AMM interface, or bridge representation.

That is an intentional security and performance property:

- all movement must pass current-epoch checks;
- REDIRECT can conserve an attempted transfer exactly;
- a generic token transfer cannot bypass Heat or scoring;
- wallets do not accumulate inert old-epoch DUMP;
- no DUMP fee swap or oracle runs in the action hot path.

### GLORY

GLORY is a conventional SPL Token mint with six decimals. Its authority is the `Protocol` PDA. Player and keeper claims invoke the Token Program with protocol signer seeds.

`Protocol.glory_committed` reserves complete epoch pools before claims. Emission uses remaining committed supply rather than current minted supply, so abandoned claims cannot be reallocated into a later epoch and then exceed the cap if eventually collected.

## Accounts

| Account | Cardinality | Writable hot path | Role |
|---|---:|---|---|
| `Protocol` | one | epoch completion/opening | Version, current epoch, GLORY mint, lifetime commitments. |
| `Epoch` | one per epoch | phase/settlement/claims | Clock, entropy, counts, pools, bond accounting, worst score. |
| `Leaderboard` | one per epoch | settlement and reward claims | Max-heap retaining the best 128, then sorted winner vector. |
| `PlayerEpoch` | one per registration | actor actions/settlement | Commitment, eligibility, Heat, activity, session key, final score, badges. |
| `BalanceLane` | four per player | actions/settlement | DUMP, Guard, lock, REDIRECT, exact weighted accumulator. |
| `Rivalry` | one per ordered actor-target pair used | actions | Per-pair impact ceiling, target-lane nonce, rent recipient. |
| `KeeperCredit` | one per epoch/caller pair | settlement/claim | Settled-player count and one-time GLORY claim. |

Fixed accounts are allocated to their exact Borsh size and covered by serialization tests. The leaderboard reserves 7,573 bytes including its discriminator, below the 10 KiB threshold. Player-created rent remains material but is recoverable through explicit close instructions.

The 2,560-player v3 cap keeps the five-percent winner set at or below 128 and bounds total settlement work for the current Anchor implementation. It must not be silently increased. The intended research direction is now one global target namespace—not regional or MMO-style rooms—using scheduled signed intents, off-chain data availability, and authenticated aggregate settlement. That is a separate architecture and is not implemented by the current program; see [GLOBAL_ARENA.md](GLOBAL_ARENA.md).

## PDA identities

```text
protocol:     ["protocol"]
glory mint:   ["glory_mint"]
epoch:        ["epoch", epoch_u64_le]
leaderboard:  ["leaderboard", epoch_u64_le]
player:       ["player", epoch_u64_le, owner]
lane:         ["lane", epoch_u64_le, owner, lane_u8]
rivalry:      ["rivalry", epoch_u64_le, actor, target]
keeper:       ["keeper", epoch_u64_le, keeper]
```

Every relevant account also stores its epoch/owner/index and is constrained against the PDA inputs. The redundancy gives readable state and defense in depth.

## Instruction groups

### Bootstrap and lifecycle

- `initialize_protocol`
- `open_next_epoch`
- `begin_reveal`
- `seal_randomness`
- `cancel_epoch`
- `begin_active`
- `begin_settlement`
- `complete_epoch`

All phase transitions are permissionless and time/phase constrained. Opening a new epoch requires the prior current epoch to be complete or cancelled and its number to increment exactly.

### Player lifecycle

- `register`
- `reveal`
- `claim_allocation`
- `claim_bond`
- `claim_player_reward`
- `refresh_badges`

Registration creates the player and four lanes in one transaction and transfers the fixed activity bond. Allocation is a player pull, avoiding an all-player initialization loop. Settlement force-applies an unclaimed allocation with full-epoch scoring, closing the inactivity loophole.

### Gameplay and automation

- `dump`
- `absorb`
- `arm_redirect`
- `authorize_session`
- `revoke_session`

An action signer must be the owner or its unexpired, action-limited session delegate. The delegate pays rent when it creates a new ordered Rivalry PDA and receives that rent back when the rivalry is later closed.

### Settlement and cleanup

- `settle_player`
- `claim_keeper_reward`
- `sweep_expired_bonds`
- `close_rivalry`
- `close_player_accounts`
- `close_keeper_credit`

Settlement processes exactly one player and checkpoints exactly four lanes. It updates a bounded heap and pays the SOL bounty atomically. The finalizer only sorts at most 128 entries. Winners and keepers claim later, so completion never loops over token recipients.

Cleanup constraints prevent closing a winner before its GLORY is claimed or an eligible player before its live bond window is resolved. Expired SOL is swept forward while preserving account rent floors.

## Concurrency model

A DUMP or ABSORB instruction writes:

```text
actor PlayerEpoch
actor lane
target lane
ordered Rivalry
```

It reads the target `PlayerEpoch` and epoch. This means:

- one actor's actions serialize because its Heat/activity state is shared;
- the same ordered actor-target pair serializes because its nonce/impact is shared;
- attacks by different actors can execute concurrently when they touch different target lanes;
- global protocol and leaderboard accounts are absent from the gameplay hot path.

DUMP chooses a deterministic target lane. The client predicts it from the same domain-separated hash, and the program rejects a stale or dishonest prediction. A race can therefore fail safely rather than land on a caller-selected lane.

The frontend filters `PlayerEpoch` and `BalanceLane` RPC scans by the first serialized epoch field at byte offset eight. It does not download every historical player and filter them in the browser.

## Score state

Each lane stores:

```text
balance
cumulative_weighted: u128
last_checkpoint_at
```

Before balance mutation, `checkpoint` clamps time to the active interval, adds the exact weighted area for the old balance, advances the timestamp, and releases an expired ABSORB lock. Settlement checkpoints at the fixed epoch end, then sums the four `u128` accumulators.

The maximum active duration, participant cap, and 10B starting tier leave large headroom beneath `u128`; every addition, subtraction, multiplication, and conversion that can fail returns a program error rather than wrapping. Release builds retain overflow checks.

## Randomness domains

Distinct labels prevent one hash output from being reinterpreted across protocols:

```text
glory-dump-commitment-v3
glory-dump-contribution-v3
glory-dump-epoch-seed-v3
glory-dump-allocation-v3
glory-dump-target-lane-v3
glory-dump-tie-break-v3
```

The commitment includes the secret, owner, and epoch. Contributions include the same identity and are XOR-aggregated; sealing hashes the aggregate, epoch, and program ID. Allocation, lane, and tie-break hashes each include the sealed seed and their own inputs.

Client and program commitment implementations share a fixed test vector. This prevents a silent byte-order or domain mismatch, but it does not solve selective withholding.

## Events and indexing

Every material transition, action, settlement, claim, and cleanup emits a typed Anchor event. Final state is authoritative on-chain; a historical battle feed should consume finalized events into an indexed read model.

The indexer is deliberately outside this commit because its infrastructure and trust policy are separate from program correctness. A production version should:

- bind every record to cluster, program ID, signature, slot, and log index;
- tolerate forks and confirmation upgrades;
- rebuild deterministically from a chosen start slot;
- compare projected standings with settled on-chain results;
- expose stale-height metadata to the UI;
- never hold gameplay or mint authority.

The demo feed is synthetic and labeled. In live mode without an indexer, the UI says that historical feed data is unavailable rather than inventing it.

## Frontend trust boundary

The generated IDL types bind method names, arguments, accounts, and errors. The client derives PDAs locally and presents exact action previews, but the wallet and program remain the enforcement points.

Reveal secrets are generated with Web Crypto, stored under program/owner/epoch in browser storage, and exported as a versioned JSON backup. Restore validates its metadata, recomputes the commitment, and compares it with the on-chain player account before saving. The file contains the reveal secret, not a wallet private key, but still must be kept private until reveal.

The wallet-only Anchor code is dynamically imported. A demo build boots from a small UI chunk; the heavier IDL and Solana codecs load only when a live program ID is configured.

## Excluded systems

There is no bridge, cross-chain mint, DUMP SPL wrapper, transfer tax, fee pot, oracle, automated market maker, buyback, burn controller, DAO, emergency pause, upgrade instruction, or on-chain vulnerability inbox.

These omissions are architectural decisions, not unfinished stubs. Adding one changes the security and economic model and requires its own review.
