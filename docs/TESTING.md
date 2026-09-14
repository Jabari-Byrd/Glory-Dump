# Testing and simulation gates

Testing is split by what each layer can actually prove. A pure-rule test, host Anchor compile, SBF execution test, economic simulation, and audit are not substitutes for one another.

## Gate A — deterministic rule mechanics

Run:

```bash
cargo test --workspace
```

The current Rust suite covers:

- exact ten-tier allocation and conservative four-lane splits;
- constant-balance weighted-score identity;
- late weighting and the 99%-hold counterexample;
- Heat capacity, recovery, and split behavior;
- Guard conversion/cap and exact redirect conservation;
- per-opponent impact caps;
- sublinear participation emission, clamps, halving, and lifetime supply bounds;
- winner-count bounds and exact rank-reward conservation for every supported winner count;
- tie-break ordering and achievement thresholds;
- commitment-domain client test vector;
- planning-claim timestamp clamping;
- exact fixed-account serialization sizes;
- top-128 heap retention and maximum leaderboard serialization.

These are fast host tests. They prove rule arithmetic and selected data-structure behavior, not Solana runtime integration.

## Gate B — static and client parity

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p glory_dump --features idl-build
pnpm run check:frontend
pnpm audit --audit-level moderate
```

The frontend suite checks the cross-language commitment vector, weighted score, last-minute behavior, Heat math, minimum action, exact human-number parsing, and formatting. TypeScript is strict with unchecked array indexing enabled. The production build proves the generated IDL and wallet gateway bundle together; it does not prove a wallet will approve or the program will execute. The current lockfile also passes the npm advisory audit with no known findings; `pnpm-workspace.yaml` records explicit compatibility-tested patches for transitive packages whose Anchor/Web3 ranges have not caught up.

After any interface change:

```bash
pnpm run idl
git diff -- frontend/src/idl
```

Review every instruction/account/type change rather than accepting generated output blindly.

## Gate C — SBF and local-validator lifecycle

This gate requires the pinned Solana CLI and Anchor CLI:

```bash
anchor build
anchor test --validator legacy
```

Current implementation evidence (2026-09-12): `cargo-build-sbf` from Solana CLI 4.1.2 produced a current 603 KiB production-rules program (`SHA-256 04cc35409a0d0212bdaef43d540d0fb30bebf700576f41e0c808cef48339ad44`). A fresh loopback validator loaded the program at the declared development address. The initializer and registration smoke create the Protocol, epoch, leaderboard, six-decimal GLORY mint, player, exact 2,000,000-lamport bond, and four lane PDAs through real runtime instructions.

`scripts/lifecycle-localnet.mjs` adds a managed, 21-wallet end-to-end suite. It uses an SBF build with `test-fast`, whose protocol version has the high bit set (`0x8003`) so the production client rejects it. Production constants are unchanged. The suite waits through shortened real clock windows and verifies 20 reveals and eligible players, one penalized withholder, seeded allocations, DUMP/ABSORB/REDIRECT, Guard and locks, session delegation, atomic failure cases, exact global DUMP conservation, all-player settlement, GLORY and SOL claims, rent cleanup, a cancelled successor, and sequential opening through epoch 3. It also records per-instruction fee and compute-unit totals. The passing test-only SBF was 603 KiB (`SHA-256 00aeb36b0adba847f682ddc44fdb95f916e854d90865661c1f6f7ea6f12c1819`).

The latest clean fresh-ledger run conserved 101B DUMP and minted exactly 44,700 GLORY, equal to the committed reward pool. Average observed compute was about 47.6k CU per registration, 10.1k per reveal, 25.8k per allocation claim, 29.3k per eligibility DUMP, and 44.5k per player settlement. These are local-validator observations, not Mainnet capacity guarantees.

```bash
cargo-build-sbf \
  --manifest-path programs/glory_dump/Cargo.toml \
  --sbf-out-dir /private/tmp/glory-dump-sbf-test-fast \
  --features test-fast

GLORY_DUMP_TEST_PROGRAM_SO=/private/tmp/glory-dump-sbf-test-fast/glory_dump.so \
  pnpm run test:lifecycle:localnet
```

This is a substantial Gate C lifecycle pass, not the complete release matrix. Maximum-room registration, late/unclaimed score parity, repeated-pair caps, reordered settlement, bond sweeping, canonical GLORY transfer, and every close-safety edge below still need dedicated validator cases after the placeholder program ID is replaced.

The validator suite must create independent wallets and verify at least:

### Bootstrap and identity

- one caller initializes the fixed protocol, GLORY mint, epoch 1, and leaderboard;
- a second initialization fails;
- every PDA matches its documented seeds;
- the mint authority is only the protocol PDA and freeze authority is absent;
- the executable program ID, IDL address, and client configuration match.

### Registration, reveal, and cancellation

- registration transfers exactly 0.002 SOL in addition to rent;
- duplicate registration and zero commitment fail;
- participant 2,561 fails without corrupting counts;
- reveal cannot start early or below the registration threshold;
- correct secrets reveal once; wrong secrets and cross-wallet/epoch backups fail;
- late reveals fail;
- underfilled registration and reveal phases cancel only after their deadlines;
- every cancelled player can reclaim its full bond exactly once.

### Allocation and score

- allocations are always a seeded tier from 1B through 10B and split conservatively;
- a planning-phase claim succeeds and starts scoring at the active boundary;
- a late active claim includes the missed active interval;
- a non-revealer receives 10B and remains ineligible;
- an unclaimed player is force-allocated during settlement with the same final score as an equivalent start-time claim;
- frontend projected scores equal program settlement scores across multi-action histories.

### Gameplay

- owner and valid session delegate actions succeed; strangers, expired delegates, and exhausted delegates fail;
- DUMP and ABSORB Heat are independent and rollback completely on failure;
- deterministic DUMP lane prediction rejects stale, caller-chosen, and out-of-range lanes;
- many actors can target different lanes without changing conservation or scores;
- DUMP cannot spend locked burden;
- ABSORB moves exact burden, extends live locks, releases expired locks, and caps Guard;
- REDIRECT requires Guard, can arm in planning, returns exact burden, consumes Guard, disarms on a hit, and respects rearm time;
- self-actions, zero/dust actions, inactive phases, unclaimed players, insufficient balances, and arithmetic boundary cases fail atomically;
- repeated pair activity never exceeds its impact credit cap while still moving valid DUMP.

### Settlement, claims, and cleanup

- nobody can settle before the fixed deadline;
- each player settles once and each successful call pays exactly 0.0004 SOL;
- interrupted and reordered settlement yields the same final winner set;
- completion fails until all registrations settle;
- fewer than 20 eligible players emit no GLORY;
- winner, rank, keeper, and total pools conserve exactly;
- winners and keepers claim once into canonical token accounts; non-winners cannot claim;
- GLORY transfers normally through the Token Program and supply never exceeds the cap;
- eligible activity bonds refund 80%, ineligible completed bonds forfeit, and double claims fail;
- excess bonds sweep only after expiry and only into a newer current epoch;
- rivalry rent returns to its original payer;
- player/lane accounts cannot close before bond/reward safety conditions;
- keeper credit cannot close before its nonzero reward claim;
- a complete or cancelled epoch opens exactly one sequential successor.

Record transaction compute units, account rent, signatures, and final account hashes. A “transaction succeeded” assertion without state and lamport checks is insufficient.

## Gate D — contention and operational load

Against a controlled validator or Devnet room, measure:

- registration account creation at 20, 100, 500, and 2,560 players;
- simultaneous DUMP attacks distributed across one through four target lanes;
- same-actor, same-rivalry, and same-lane conflict/retry rates;
- RPC filtered account reads and UI refresh latency;
- one-player settlement compute, fee, and throughput;
- whether SOL plus GLORY keeper compensation exceeds actual caller cost;
- indexer catch-up, fork handling, stale-height display, and deterministic rebuild;
- wallet confirmation latency and dropped/expired transaction recovery.

The four-lane design is successful only if measured scheduler/retry behavior improves, not because account diagrams look parallel.

## Gate E — bot-driven economic simulation

`crates/glory-dump-sim` is a deterministic simulator that imports `glory-dump-core` and models heterogeneous agents:

- random and inactive baselines;
- small-packet scattering as a non-coalition action-granularity control;
- greedy lowest-score dumping;
- leader hunters using public standings;
- quiet “small fry” that avoid visibility;
- Guard builders and redirect bluffers;
- last-window actors;
- reciprocal alliances and rotating coalitions;
- sacrificial Sybil fleets;
- self-dogpile attempts;
- bribed-coalition controllers, with optional GLORY-price net sensitivity;
- reveal-withholding coalitions.

Run the canonical mixed baseline and the standard adversarial matrix with:

```bash
cargo run -p glory-dump-sim -- run --population 100 --epochs 100 \
  --output /private/tmp/glory-dump-baseline.json
cargo run -p glory-dump-sim -- sweep --epochs 100 \
  --output /private/tmp/glory-dump-sweep.json
cargo run -p glory-dump-sim -- run --v4 --population 100 --epochs 100 \
  --output /private/tmp/glory-dump-v4.json
cargo run -p glory-dump-sim -- v4-sweep --epochs 100 \
  --output /private/tmp/glory-dump-v4-sweep.json
cargo run -p glory-dump-sim -- scale \
  --output /private/tmp/glory-dump-global-scale.json
```

The simulator checks DUMP and reward conservation on every epoch, labels any parameter override noncanonical, and reports action fees, bond losses, optional hypothetical GLORY value, strategy/tier outcomes, Guard utilization, targeting, chapter/endgame turnover, stamina and target-cap failures, coalition coordination assumptions, and GLORY concentration. Reproducibility, conservation, canonical-rule parity, legacy-config loading, inactive/withholding eligibility, chapter scoring, stamina carry, target-wide caps, scheduled-batch order, corrected coarse-cadence turnover, global-scale arithmetic, and experimental labeling have unit tests.

Sweep seeds, population, starting allocations, action latency, RPC failure, information delay, number of Sybils, and every tunable parameter listed in `GAME_DESIGN.md`.

Primary outputs:

- winner concentration and turnover by strategy, including coalition-controller outcomes separately from sacrificial helpers;
- correlation between starting tier and win probability;
- Gini/concentration of GLORY emission;
- fraction of inactive/degenerate epochs;
- action mix, target concentration, and recovery after dogpiles;
- benefit/cost of Sybil wallets after bonds, rent, and fees;
- frequency and value of last-window reversals;
- Guard created, consumed, and stranded;
- Heat saturation and idle time;
- liveness reward profitability;
- transaction conflicts and expected user cost;
- sensitivity to public versus delayed standings.

Predeclare acceptable bands before tuning. Use held-out seeds to confirm any parameter change; otherwise the simulator becomes a machine for overfitting one invented population.

See [SIMULATION.md](SIMULATION.md) for the bot definitions, predeclared exploratory bands, commands, and unmodeled boundaries. Validator contention, rent, keeper profitability, indexer behavior, real liquidity, and human behavior remain outside this pure model and keep Gate E open.

Current corrected evidence rejects both the committed v3 balance and automatic promotion of the v4 candidate. V4 reduced final-three-day counterfactual winner turnover from `87.0%` in the canonical reference to `19.0%–21.8%` across four fresh 100-epoch mixed runs, and reduced sacrificial-controller wins from `33%` to `7%–10%`. It also produced a severe `42.7x–86.5x` nonzero starting-tier spread in those mixed rooms and a `2.20x–2.69x` small-packet advantage. An incoming-DUMP cap was rejected because a 30%-Sybil fleet could manufacture the protection for its own controller. No simulator-only rule has been copied into the Anchor program.

The analytical `scale` command is not Gate D evidence. At seven billion players, its default one-action-per-three-days scenario still yields about 27,006 signed intents/s. Aggregating 10,000 intents per settlement reduces the modeled on-chain count to about 2.70 settlements/s, but no prover, verifier, sequencer, forced-inclusion path, or data-availability system exists. A one-world architecture must pass those implementation and adversarial gates before replacing v3; see [GLOBAL_ARENA.md](GLOBAL_ARENA.md).

## Gate F — independent review and release

Before mainnet consideration:

- independent Anchor/Solana program audit;
- frontend supply-chain and wallet-transaction audit;
- formal or property-based verification of conservation, one-time claims, cap accounting, phase reachability, and close safety;
- randomness decision and adversarial review;
- monitored Devnet rehearsal over at least one accelerated full lifecycle;
- reproducible binary and IDL verification;
- incident, disclosure, and upgrade-authority procedures;
- legal and consumer-protection review.

Only this final gate can change the designation from an experimental testnet game. Bot results alone cannot.
