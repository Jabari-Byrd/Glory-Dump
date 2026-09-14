# Game design contract

This document records the v3 rules chosen before bot tuning. Constants live in `crates/glory-dump-core/src/lib.rs`; the Anchor program calls that crate rather than duplicating game math.

## Design goals

1. The winning strategy must be recognizably “reverse wealth”: carrying less DUMP for less time is always better.
2. Starting chaos must force early decisions without letting a last-block trick erase the epoch.
3. Actions must create tradeoffs, not merely rename token transfers.
4. Dogpiles are part of the social game, but automatic anti-dogpile benefits must not be cheaply farmable with Sybil accounts.
5. A player who actually participated wins an exact tie over an inactive lucky wallet.
6. Common actions must remain bounded and parallelizable on Solana.
7. GLORY may leave the game and trade normally; DUMP must not.

## Fixed epoch schedule

| Phase | Length | Purpose |
|---|---:|---|
| Registration | 7 days | Discover the event, commit entropy, and post the activity bond. |
| Reveal | 12 hours | Reveal commitments after registration membership is fixed. |
| Planning | 1 hour | Claim allocations, inspect the map, and arm existing Guard. |
| Active | 30 days | Score accrues and PvP actions execute. |
| Settlement | permissionless | Process one registered player per transaction. |
| Claims | 7 days for SOL bond | Pull GLORY and SOL; GLORY claims do not expire. |

There is no mid-epoch late join. Spectators can watch and prepare for the next registration window. This preserves the event feeling and avoids inventing a scoring handicap that sophisticated wallets would optimize around.

An epoch needs 20 registrations to proceed and two valid reveals to form a seed. If either threshold fails, it can be cancelled and all registration bonds are refundable. An epoch with fewer than 20 ultimately eligible players can still finish, but emits no GLORY.

## Starting chaos

Each valid revealer receives one of ten discrete tiers with equal hash-space probability:

```text
1B, 2B, 3B, ... 10B DUMP
```

The allocation is derived from the sealed epoch seed, player public key, epoch number, and a domain label. A participant that does not reveal receives 10B, cannot rank, and cannot recover the activity portion of its bond after a completed epoch.

The random spread is intentionally large. A 10B player faces immediate pressure to offload burden, while a 1B player becomes an obvious target. Equal starts would encourage an opening stalemate and make the first move mostly arbitrary.

Commit/reveal reduces simple address grinding but retains selective-withholding risk; see `SECURITY.md`.

## Scoring

For elapsed fraction `x = t / T`, the instantaneous weight is:

```text
w(x) = 1 + 3x^2
```

Its epoch average is `2`, so the normalized score is:

```text
score = integral(balance(t) * w(t)) / (2T)
```

The implementation uses the exact integer antiderivative, not periodic sampling:

```text
area(a,b) = (b-a)T^2 + b^3 - a^3
```

Every lane checkpoints `balance * area` before its balance changes. Four lane accumulators are summed during settlement and divided by `2T^3`.

This curve makes an equal interval near the deadline worth more without turning the opening into dead time. The second half has 2.2 times the weight of the first half. A constant balance scores itself exactly. Holding 1B for 99% of the epoch and zero for the final 1% scores roughly 980M.

Unclaimed allocations count from the active boundary. Refusing to click “claim” therefore cannot manufacture a zero-balance history; settlement applies the burden retroactively from the start.

## Four balance lanes

Every player has four DUMP lane PDAs. The starting allocation is split exactly across them.

- DUMP selects the target lane from the epoch seed, actor, target, and ordered-rivalry action count. The sender cannot choose the easiest unguarded lane.
- ABSORB lets the actor choose both its destination lane and the rival source lane.
- REDIRECT is armed per lane.
- Scoring sums all lanes, so splitting never changes the objective.

The lanes are primarily a Solana concurrency tool. Independent attackers that hash to different target lanes do not all request the same writable balance account. The target's player account is read-only during DUMP and ABSORB.

## Heat instead of a fixed cooldown

DUMP and ABSORB each have a 10,000-unit Heat channel. Heat falls linearly from full to zero in six hours.

```text
actionHeat = ceil(amount / startingAllocation * 10,000)
```

A player can therefore move roughly one original allocation per channel per six-hour recovery cycle. Moving 10% costs 10% Heat; moving 100% consumes an empty channel. Because Heat is normalized to the chaotic starting allocation, a 10B start gets more absolute throughput but not more proportional throughput.

Each action must be at least 0.1% of the actor's starting allocation. This prevents zero-impact calls from earning eligibility, distinct-opponent credit, or cheap session-action spam. Integer ceiling also ensures tiny splits cannot evade Heat.

Separate channels create combinations: a player may DUMP and then ABSORB while DUMP Heat recovers, but ABSORB deliberately worsens its own score.

## Actions

### DUMP

Moves unlocked DUMP from one actor lane to the deterministic rival lane.

If the destination has REDIRECT armed, up to its current Guard returns to the actor's source lane. The attempted amount leaves the source first; `landed + redirected == attempted` always. A successful ricochet consumes that amount of Guard, disarms the lane, and starts the rearm timer.

### ABSORB

Takes unlocked DUMP from a selected rival lane and puts it in the actor's selected lane.

- The burden is score-negative immediately.
- The newly absorbed amount is locked for six hours.
- Repeated absorption extends the lane lock, so old locked burden cannot be laundered by adding a fresh amount.
- 50% of the absorbed amount becomes Guard.
- Each lane's Guard caps at 6.25% of starting allocation, for a 25% four-lane maximum.

ABSORB is a tactical sacrifice. A leader can take DUMP to build a shield before an expected raid; a heavily burdened player can steal from a leader to help that leader while gaining defense. That ambiguity is intentional.

### REDIRECT

Arms a lane that has nonzero Guard. It may be armed during planning or active play, uses one authorized session action, and has no Heat cost because Guard had to be earned through a harmful ABSORB.

It is single-use on a nonzero ricochet and rearms after 15 minutes. An attack larger than Guard partly lands; a smaller attack consumes only the returned amount.

## Dogpiles and counterplay

The game does not prohibit coordinated targeting. Public standings and territory size intentionally make leaders visible.

Counterplay is explicit rather than victim-triggered:

- preserve multiple spendable lanes;
- ABSORB before a predicted attack to forge Guard;
- arm REDIRECT on likely target lanes;
- counter-DUMP while attackers' own Heat is high;
- spread meaningful actions across rivals for tie-break credit.

There is no incoming-attack meter that reduces a victim's Heat. Such a meter would let a player command Sybil wallets to attack it just before the deadline, refill its outgoing capacity, and dump at the moment of maximum score weight.

The first 500,000 parts-per-million of normalized volume against each ordered opponent can count toward the activity tie-break. Further attacks still move DUMP but add no impact credit. This limits one colluding pair's tie-break contribution without pretending to stop the underlying PvP.

## Eligibility and tie-breaks

A player is eligible only if it:

1. revealed its committed secret; and
2. completed at least one minimum-sized DUMP or ABSORB action.

REDIRECT alone does not establish meaningful economic participation because arming without an incoming hit may change nothing.

Standings sort by:

1. lowest weighted DUMP score;
2. highest capped normalized impact;
3. highest number of distinct impacted opponents;
4. lowest seed-derived random value;
5. public key bytes.

Activity cannot compensate for a worse primary score. It only decides the rare integer-score tie, matching the preference for an engaged player over a lucky inactive wallet.

## Bonds and liveness

Registration transfers a 0.002 SOL bond into the epoch account, separate from rent for player-created accounts.

- Every settlement pays its caller 0.0004 SOL.
- An eligible player can reclaim 0.0016 SOL after completion.
- Ineligible players forfeit the remainder after the claim window.
- A cancelled epoch refunds the full 0.002 SOL.
- After seven days and a newer epoch, anyone can sweep spendable leftovers forward while preserving rent.

All player, lane, rivalry, and keeper account rent has a close path after its claims are safe. The fixed amounts are initial playtest values; bot simulations must compare them with observed transaction fees and Sybil economics.

## GLORY rewards

Eligible winner count is `ceil(eligible * 5%)`, bounded from one to 128 once there are at least 20 eligible players.

The base emission is 100,000 GLORY at 100 eligible players. Participation changes it sublinearly:

```text
multiplier = clamp(sqrt(eligible / 100), 0.25, 4.0)
```

The base halves every 12 epochs and all commitments stop at the 10,000,000 GLORY cap. One percent of each emitted pool goes to settlement keepers. The keeper pool is rounded to an exact per-settlement amount so all keeper claims conserve it.

The player pool uses a gentle descending weight. First place earns more than the mean and last winning place less, but first remains under twice the final winner for normal multi-winner epochs. Integer remainder goes to first place so claims sum exactly to the pool.

The square-root curve rewards growing the player base without making a 100x Sybil population generate 100x tokens.

## Achievements

Badges carry no GLORY bonus and cannot override standings:

- **Escape Artist** — start at 8B or more and finish with a score at or below 25% of the start.
- **Human Shield** — finish with at least 150% of the starting burden.
- **Ricochet** — redirect at least 10% of the starting allocation.
- **Last Laugh** — earn at least 100,000 impact ppm during the final 10% of the epoch.
- **Garbage Emperor** — settle with the epoch's worst score, with a deterministic address fallback.

They make losing histories legible and give the community bragging rights without turning side quests into pay-to-win stat bonuses.

## V4 simulator candidate

The committed program still follows every v3 rule above. The following candidate exists only in `glory-dump-sim` so pacing and coalition ideas can be rejected cheaply before they touch accounts or the IDL:

- divide the 30-day active phase into six five-day chapters;
- weight every chapter equally instead of increasing time weight from `1x` to `4x`;
- replace six-hour Heat with independent DUMP and ABSORB chapter stamina;
- calculate each channel's allowance from `15% * starting DUMP + 85% * 5.5B`, reducing the absolute-capacity range to 4.825B–6.175B and the 10B-to-1B ratio from 10:1 to about 1.28:1;
- grant 100% of that blended basis per chapter and carry at most one unused chapter;
- let each scheduled window accept up to four pre-signed intents, then use seed-committed resolution order rather than network-arrival priority;
- cap the DUMP that all helpers combined may ABSORB away from one target during a chapter at 25% of that target's starting allocation;
- report an explicit 100,000-lamport coordination-cost assumption for every coalition helper epoch.

The action budget is independent of polling frequency. Checking hourly may improve target selection, but it does not refill stamina. A player who skips one chapter can bank it; older unused capacity expires. The intended client model is a chapter playbook: a player can sign conditional targets and fallbacks, leave, and let later global windows resolve them. The simulator currently models the actions and deterministic batch order, not wallet-side conditional syntax or proof settlement.

The target-wide ABSORB budget is deliberately narrow. It does not give a dogpiled wallet extra outgoing power: Sybil helpers cannot multiply how much burden they pull off their controller. A separate target-wide incoming-DUMP cap was implemented as an ablation and left disabled in the candidate. Stress bots manufactured that “protection” by dogpiling their own controller, exactly the exploit the design wanted to avoid.

The current candidate is not balanced or promotion-ready. Four 100-epoch mixed runs on fresh seeds put corrected last-three-day counterfactual winner turnover between 19.0% and 21.8%, down from 87.0% in the canonical reference, and the pure wait-until-late bot won zero times. The full final five-day chapter still replaced 66.4%–70.4% of the projected winners, so the endgame remains meaningful.

The same evidence rejects any claim that the candidate is finished. In the mixed population, strongest-to-weakest nonzero starting-tier win rates were 42.7x–86.5x: the near-fixed stamina basis overcorrects and leaves the largest starting burdens with too little escape capacity under strategic pressure. Four separate 300-epoch random-only runs were much less extreme but still ranged from 2.95x to 3.84x. Small-packet DUMP won at 2.20x–2.69x the population rate, while sacrificial and bribed controllers won 7%–13% of their epochs. Self-dogpile controllers won none, but their helper groups remained overrepresented. Those are remaining mechanic questions, not acceptable final bands.

## Parameters reserved for simulation

The next test phase should challenge, not assume, the current values:

- six-hour Heat recovery and 0.1% minimum action;
- 50% Guard conversion, 25% cap, six-hour lock, and 15-minute rearm;
- four lanes and target-lane distribution under coordinated load;
- 0.002 SOL bond and 80/20 refund/bounty split;
- 5% winner share and descending rank curve;
- square-root emission floor, ceiling, and 12-epoch halving;
- 500,000 ppm per-opponent tie-break cap;
- score curve's `1x` to `4x` late weighting;
- information advantage from the public atlas and feed.

No constant should be promoted merely because it sounds fair. The bot phase should measure dominant strategies, player recovery after dogpiles, action diversity, idle equilibria, Sybil profitability, winner turnover, transaction contention, and GLORY inflation before retuning anything.

The v3 baseline and first v4 ablations now exist. V4 materially reduces pure last-window play and pull-based controller coalitions, but the starting lottery and small-packet/sacrificial-sink behavior are still outside the desired bands. No simulator-only rule has been copied into the program. See [SIMULATION.md](SIMULATION.md) for the measurements and [GLOBAL_ARENA.md](GLOBAL_ARENA.md) for the one-world scaling boundary.
