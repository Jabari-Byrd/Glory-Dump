# One-world arena research

## Decision

The intended long-term game is one global target namespace. A visual tile, search result, leaderboard page, or indexer shard is only a view over that namespace; it is not a region, server, or separate economy. Any enrolled wallet should be able to target any other enrolled wallet.

This is a design target, not a claim about the current program:

- the committed Anchor v3 program remains capped at 2,560 players;
- the exact v4 simulator permits up to 100,000 in-memory players;
- the analytical scale model accepts `u64` populations, including seven billion;
- no aggregate prover, verifier, sequencer, or data-availability service exists in this repository.

## Why cooldowns are not enough

The scale model defaults to one action per player every three days. Even that deliberately slow cadence creates:

| Players | Average signed intents/s | Reveal tx/s over 12h | Minimum direct interval at the modeled 1,000 actions/s | 10,000-intent aggregate settlements/s | Intent data per round |
|---:|---:|---:|---:|---:|---:|
| 50,000 | 0.193 | 1.16 | 50 seconds | 0.000019 | 6.4 MB |
| 1,000,000 | 3.86 | 23.15 | 16m 40s | 0.000386 | 128 MB |
| 1,000,000,000 | 3,858 | 23,148 | 11.57 days | 0.386 | 128 GB |
| 7,000,000,000 | 27,006 | 162,037 | 81.02 days | 2.701 | 896 GB |

The 1,000 direct actions/s figure is a configurable planning assumption, not a Solana throughput claim. The observed 29,264 compute units per representative v3 DUMP action came from the local validator, not Mainnet. Current Solana documentation separately describes per-transaction, per-block, and per-writable-account compute limits; those limits do not promise application throughput or eliminate account contention.

The conclusion is structural: making seven billion players wait three days still leaves about 27,006 actions every second. Stretching the interval until direct transactions fit would turn the game into an 81-day click timer. Aggregation is the more faithful path if the game ever reaches that order of magnitude.

## Population-adaptive cadence without regions

The fallback is a global action-slot clock, not separate rooms. At each epoch boundary, the protocol derives one cadence for every player from the previous enrollment count and a conservatively measured settlement budget:

```text
minimum interval = ceil(expected active players / safe settled actions per second)
epoch interval   = max(gameplay floor, minimum interval)
```

The `minimum_direct_action_interval_seconds` column in the scale report evaluates that rule for the direct-transaction counterfactual. If an aggregate prover is introduced, its measured end-to-end capacity—including proof latency and data publication—replaces the direct budget in the same formula. The cadence cannot change mid-epoch, and an operator cannot selectively slow one player.

Every wallet receives the same action-slot schedule and every valid slot can target any enrolled wallet worldwide. If demand exceeds the safety budget, intents wait for the next deterministic global batch; they do not fall into a regional economy. A chapter playbook may contain fallback targets and conditions, but the scale model counts one executed action per player every three days so alternatives cannot silently multiply the throughput assumption.

Reproduce the table:

```bash
cargo run -p glory-dump-sim -- scale

cargo run -p glory-dump-sim -- scale \
  --scale-populations 50000,1000000,1000000000,7000000000 \
  --action-interval 259200 \
  --modeled-direct-aps 1000 \
  --aggregate-size 10000 \
  --intent-bytes 128
```

## Proposed scheduled-intent flow

```text
one global enrolled-wallet tree
          │
          ├── wallet signs an intent
          │     epoch + window + nonce + expiry
          │     action + amount + any target wallet
          │     max fee + current state-root guard
          │
          ├── public batch data is frozen for the window
          │
          ├── deterministic simultaneous-resolution rules
          │     ignore arrival-time priority
          │     reject duplicate nonces and invalid balances
          │     resolve DUMP / ABSORB / REDIRECT
          │
          ├── prover commits old root -> new root + action/event commitment
          │
          └── Solana verifier accepts the proof and updates the canonical root
                GLORY and final claims remain canonical Solana assets
```

Players may prepare a conditional playbook and leave. A sophisticated bot can choose targets and conditions more intelligently, but equal action slots plus chapter stamina—not polling frequency—limit total action volume. Batches use a seed-committed deterministic resolution order so paying for lower network latency does not buy first position inside a window.

The scheduled window should be a game object:

- pending attacks are visible at an intentionally chosen information level;
- a wallet can queue fallback targets or “execute only if” conditions;
- REDIRECT becomes a trap/bluff decision rather than a click-race;
- casual players can prepare a chapter without remaining online;
- the interface can page, search, and summarize the global field without changing whom a player may target.

## What aggregation does not solve

A Merkle root or compressed account reduces on-chain storage. It does not make billions of actions free or self-authenticating. A production design must resolve:

- sequencer censorship, ordering, equivocation, and forced inclusion;
- public data availability sufficient to reconstruct the state;
- validity-proof versus fraud-proof assumptions and challenge timing;
- prover cost and worst-case proof latency;
- batch failure, escape, and recovery when the operator disappears;
- global hot-target rules and deterministic concurrent conflict resolution;
- registration/reveal aggregation, which is a larger burst than steady gameplay;
- indexer forks, stale views, pagination, and proof delivery to wallets;
- the exact trust and upgrade boundary for every off-chain component.

The official Solana [compute-budget documentation](https://solana.com/docs/core/fees/compute-budget) is the current source for transaction/block/account limits. Solana's [state-compression overview](https://solana.com/developers/courses/state-compression/generalized-state-compression) explains the on-chain-root/off-chain-data tradeoff, but the page is marked as no longer maintained and is useful here only as a conceptual reference.

## Promotion gates

Before replacing v3 with this architecture:

1. specify canonical intent bytes, nonces, expiries, condition semantics, and batch ordering;
2. implement a deterministic state-transition reference with adversarial property tests;
3. benchmark proof generation and verification at 50K, 1M, and larger synthetic populations;
4. design forced inclusion, public data availability, and an operator-disappearance escape;
5. measure Solana verifier contention and root-update failure recovery;
6. run independent protocol, circuit, client, and economic audits;
7. rehearse a full accelerated season on Devnet before any value-bearing deployment.

Until those gates pass, “seven billion players” means the analytical model did not overflow or allocate seven billion objects. It does not mean the game can serve seven billion people today.
