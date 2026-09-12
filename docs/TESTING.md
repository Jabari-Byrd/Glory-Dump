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

Current implementation evidence (2026-09-12): `cargo-build-sbf` from Solana CLI 4.1.2 produced a 603 KiB optimized program (`SHA-256 5aea16462acc7e84e5474800abf965fab806d9c859a322d016c0c02dccab734d`). A fresh loopback validator loaded that exact binary at the declared development address, and the checked-in initializer successfully created the 59-byte Protocol account, 244-byte Epoch account, epoch-1 Leaderboard, and GLORY mint. The mint reported six decimals, zero initial supply, Protocol-PDA mint authority, and no freeze authority. `pnpm run smoke:localnet` then executed a real registration and asserted the exact participant-count increment, 2,000,000-lamport bond delta, commitment and owner, plus four correctly indexed zero-balance lane PDAs.

That is a bootstrap smoke, not a completed Gate C. The multi-wallet lifecycle and adversarial failure matrix below remains to be automated and passed after a real program ID is synchronized.

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

This is the next design phase after the implementation commit, not evidence already claimed by it.

Build a deterministic simulator that imports or faithfully mirrors `glory-dump-core` and model heterogeneous agents:

- random and inactive baselines;
- greedy lowest-score dumping;
- leader hunters using public standings;
- quiet “small fry” that avoid visibility;
- Guard builders and redirect bluffers;
- last-window actors;
- reciprocal alliances and rotating coalitions;
- sacrificial Sybil fleets;
- self-dogpile attempts;
- bribery driven by an external GLORY price;
- keeper participation under changing SOL fees;
- reveal-withholding coalitions.

Sweep seeds, population, starting allocations, action latency, RPC failure, information delay, number of Sybils, and every tunable parameter listed in `GAME_DESIGN.md`.

Primary outputs:

- winner concentration and turnover by strategy;
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
