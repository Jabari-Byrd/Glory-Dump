# Bot and economic simulation

The simulator is a deterministic research harness around the same `glory-dump-core` arithmetic used by the Anchor program. It is designed to find incentives worth investigating, not to certify that people or markets will behave like its bots.

## Run it

```bash
cargo run -p glory-dump-sim -- run \
  --population 100 \
  --epochs 100 \
  --seed 0x474c4f525944554d \
  --scenario mixed \
  --output /private/tmp/glory-dump-baseline.json

cargo run -p glory-dump-sim -- sweep \
  --epochs 100 \
  --seed 0x474c4f525944554d \
  --output /private/tmp/glory-dump-sweep.json

cargo run -p glory-dump-sim -- run --v4 \
  --population 100 \
  --epochs 100 \
  --scenario mixed \
  --output /private/tmp/glory-dump-v4.json

cargo run -p glory-dump-sim -- v4-sweep \
  --epochs 100 \
  --output /private/tmp/glory-dump-v4-sweep.json

cargo run -p glory-dump-sim -- scale \
  --output /private/tmp/glory-dump-global-scale.json
```

`run --help` lists every override. A report says `canonical_rules: true` only when all game parameters exactly match `glory-dump-core`. Changing Heat, Guard, lock, rearm, or minimum-action values marks the report as experimental; it never silently relabels a tuned run as the committed game. “Canonical” does not mean byte-for-byte transaction replay: bot decisions and the simulator's deterministic samples of hashed lane/tie outcomes remain research-model inputs.

Reports are written through a sibling temporary file and rename. Supplying the same configuration and seed produces the same JSON.

The RNG has reference-vector and nested-stream regression tests. That second check matters because an early harness draft accidentally replayed portions of the parent stream inside successive epochs; corrected reports have the expected near-uniform revealed allocation counts rather than overlapping pseudo-samples.

## Bot population

The mixed room includes:

- random and inactive controls;
- a small-packet control that scatters 3%–9% DUMP moves without coalition behavior;
- a greedy dumper and a public-standings leader hunter;
- a quiet small-fry strategy that avoids unnecessary visibility;
- Guard builders and redirect bluffers;
- a last-window specialist;
- reciprocal and rotating coalitions;
- sacrificial Sybil, self-dogpile, and bribed-coalition adversaries;
- a reveal-withholding adversary.

Dedicated `random_only`, `leader_hunt`, `guard_heavy`, `last_window`, and `sybil_stress` scenarios isolate particular effects. The built-in sweep varies room size, one-hour information lag, a 5% submission failure rate, 30% Sybil control, Guard caps, Heat recovery, and last-window pressure.

## What is measured

Each report includes:

- eligibility, winners, strategy win rate, and controller-only coalition advantage so helper losses cannot hide an exploit;
- starting-tier win rate and final-score ratio;
- action mix, failed Heat/stamina/target-cap actions, late actions, and DUMP moved;
- maximum target share, chapter-boundary turnover, and final-window winner turnover;
- Guard created, used through ricochets, and stranded;
- GLORY issuance and player-level Gini;
- estimated signature fees, bond losses, strategy break-even GLORY price, and optional hypothetical proceeds;
- exact DUMP and reward-pool conservation checks;
- machine-readable watch/risk signals.

The hypothetical GLORY price is an input for sensitivity analysis, not a price forecast. Economic net and break-even price are explicitly reported before account rent, market slippage, and off-chain coordination/bribe costs because the pure simulator does not execute or observe those systems.

## Predeclared exploratory bands

These are the first-pass review bands for a 100-player mixed room across at least 100 epochs. They were chosen before treating a full baseline report as a tuning result:

| Signal | Healthy exploratory band | Investigate |
|---|---:|---:|
| Dominant strategy or coalition controller advantage | below `2x` population win rate | `2x` watch, `3x` risk |
| Strongest/weakest winning start tier | below `3x` | `3x` watch, `5x` risk |
| Final-window winner turnover | `10%–50%` | over `50%` is unstable; near zero is stale |
| Guard converted into ricochets | at least `10%` | below `10%` suggests dead defense inventory |
| Maximum target share | below `25%` | higher values suggest dogpile capture |
| GLORY Gini across persistent bot identities | below `0.65` | higher values suggest repeated capture |
| Inactive reward epochs | `0` when population is at least 20 | any occurrence |

A band violation is a research lead, not permission to change the protocol. Parameter candidates need a separate training-seed sweep and confirmation on held-out seeds and altered bot mixtures.

## Current exploratory evidence

The turnover metric now compares the final winner set with the winners projected at the boundary if nobody acted again. Earlier reports compared a partial score directly with the final score and could count turnover that later time passage—not later play—caused. Starting-tier win rates now divide by eligible player-epochs; the separate all-player rate still exposes withholding and inactivity. Results before those corrections should not be compared numerically with the following runs.

### Committed v3 reference

The corrected canonical run used 100 players for 100 epochs at seed `0x474c4f525944554d`. It completed 10,000 player-epochs with exact DUMP and reward conservation and no rewardless epoch. Its notable outputs were:

- greedy DUMP won at `2.49x` the population-wide rate;
- the sacrificial-Sybil controller won `33%` of its epochs (`6.60x` advantage) and the bribed controller won `25%` (`5.00x`);
- strongest-to-weakest nonzero eligible starting-tier win rate was `4.34x`;
- `87.0%` of final winners differed from the no-more-actions projection at the final-three-day boundary;
- the pure wait-until-late bot won zero times, so the turnover comes from broad endgame activity rather than that one bot;
- `46.7%` of forged Guard was consumed, mean maximum target share was `8.62%`, and GLORY Gini was `0.574`.

A 200-epoch random-only v3 control at seed `0x52414e444f4d3032` retained the structural starting-tier warning: 1B won zero of 1,992 player-epochs, while 7B won `8.85%`. The strongest-to-weakest nonzero ratio was `34.56x`. Public leader hunting therefore cannot be the only cause.

### Simulator-only v4 candidate

V4 changes temporal scoring, action capacity, and batch execution only inside this harness. Four 100-epoch mixed runs used seeds `0x5634534545443031` through `...34`:

| Measurement | Observed range |
|---|---:|
| Final-three-day counterfactual winner turnover | `19.0%–21.8%` |
| Final-five-day chapter turnover | `66.4%–70.4%` |
| Pure wait-until-late wins | `0 / 2,800 player-epochs` |
| Small-packet relative win advantage | `2.20x–2.69x` |
| Sacrificial controller win rate | `7%–10%` |
| Bribed controller win rate | `9%–13%` |
| Self-dogpile controller win rate | `0%` |
| Strongest/weakest nonzero starting-tier rate | `42.7x–86.5x` |
| GLORY Gini | `0.623–0.645` |

This is a mixed result, not a selected winner. Chapters and stamina substantially reduce final-three-day dependence and the original controller exploit. But near-fixed capacity strands high initial burdens under strategic pressure, and scattershot small DUMPs remain too effective. Four 300-epoch random-only runs at seeds `0x51a`, `0x62b`, `0x73c`, and `0x84d` gave every starting tier wins, but per-run tier ratios still ranged from `2.95x` to `3.84x` (`3.14x` after pooling all 1,200 epochs).

The rejected incoming-DUMP-cap ablation illustrates the Sybil trap. In otherwise identical 200-epoch 30%-Sybil runs at seed `0xdead`, limiting incoming DUMP to one starting allocation per chapter reduced final-three-day turnover from `15.5%` to `0.9%`, but let the self-dogpile controller win `20%` instead of `0%` and raised the small-packet advantage from `2.27x` to `4.19x`. The v4 candidate therefore keeps that cap disabled.

The optimized exact simulator also completed its configured maximum of 100,000 players for one epoch with six five-day decision windows, 74,226 eligible players, and 945,016 successful actions. That is evidence about harness complexity and arithmetic bounds only. It is not a validator, network, prover, or human-concurrency result.

Local reports belong under ignored `simulation-output/` or a temporary directory. The next economic experiment should add adaptive/start-aware adversaries and a human-attention/playbook model, then test opening-only capacity versus a per-action stamina overhead on fresh training seeds before reserving another confirmation set.

## Boundaries

The exact simulator does not model validator account-lock scheduling, blockhash expiry, rent recovery, wallet abandonment, an event indexer, off-chain communication, real GLORY liquidity, or human novelty. RPC failures and information delay are injected abstractions. Its scheduled-batch mode resolves precomputed intents in seeded order; conditional wallet playbooks, concurrent-conflict semantics, proof generation, forced inclusion, and data availability remain unimplemented.

The separate `scale` command performs arithmetic without allocating one object per global player. At seven billion players and one action per player every three days, it reports about 27,006 signed intents/s, 700,000 ten-thousand-intent proof settlements per round, 2.70 proof transactions/s averaged across the round, and 896 GB of signed intent data. Those values are scenario math—not a throughput benchmark or proof that a suitable prover exists. See [GLOBAL_ARENA.md](GLOBAL_ARENA.md).

The bots also know only what their strategy is explicitly given. The withholding bot applies the on-chain maximum-burden/ineligibility penalty but does not search alternative reveal subsets to grind the final epoch seed; selective-abort randomness remains a separate adversarial experiment. Public Guard remains public, matching the v3 fairness decision; a truly hidden ricochet would require a different delayed-settlement protocol and a separate simulator.
