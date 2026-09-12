# v3 rewrite notes

Date: 2026-09-12

Scope: full repository redesign from the recovered Base/Solidity prototype into a native Solana/Anchor strategy game. This is an engineering implementation review, not an independent security audit, economic proof, or mainnet approval.

## Why this was a rewrite

The core idea needs frequent, low-cost, stateful PvP actions. Porting the EVM contracts line by line would have retained global storage, ERC-20 bypass concerns, population loops, and market machinery that fights the game.

The new model treats:

- DUMP as ephemeral game state spread across Solana PDAs;
- GLORY as the only standard tradable token;
- scoring and cooldowns as pure deterministic Rust;
- actions as small instructions with explicit writable accounts;
- historical visuals as an indexed projection of typed events.

## What changed

| Area | Previous prototype | Solana v3 |
|---|---|---|
| Runtime | Solidity/Hardhat on Base | Rust/Anchor on Solana |
| DUMP | Restricted ERC-20 with fees | Internal four-lane `u64` burden, never a market token |
| GLORY | ERC-20 reward token | Standard six-decimal SPL token with protocol-PDA mint authority |
| Random start | Sequencer-influenceable block values | Registration commit/reveal with maximum-burden/non-eligibility penalty for withholding |
| Score | Uniform time average | Exact continuous weight rising from 1x to 4x |
| PvP | Give and steal | DUMP, tactical ABSORB, and one-shot REDIRECT |
| Cooldown | Wall-clock delay after action | Separate amount-scaled Heat channels with linear recovery |
| Dogpile response | Mostly cooldown constraints | Four writable lanes plus earned Guard; no Sybil-farmable victim bonus |
| Tie | Score/address | Score, capped impact, distinct opponents, committed randomness, address |
| Growth reward | Fixed pool | Square-root participation scaling, epoch eras, hard lifetime cap |
| Settlement | Bounded batches | Exactly one player per call with SOL and GLORY keeper rewards |
| Player automation | General wallet approvals | Expiring, action-limited session delegate |
| Interface | Form-oriented address actions | Strategy Room, inverse atlas, lanes, Heat, previews, feed, badges |

## Features added

### Strategy mechanics

- Uniform random 1B–10B starting tiers preserve the desired opening chaos.
- A planning hour lets the field see allocations before scoring actions begin.
- Four balance lanes allow real target uncertainty and reduce unrelated write contention.
- DUMP targets a seed-determined lane so attackers cannot always choose the weakest defense.
- ABSORB is intentionally score-negative but creates 50% Guard, capped at 25% of starting burden.
- Absorbed DUMP locks for six hours so defense cannot become an instant dump-and-profit loop.
- REDIRECT returns up to the lane's Guard, consumes it, disarms, and rearms after 15 minutes.
- Any incoming amount above available Guard still lands, so saved defense cannot guarantee a late lead.
- DUMP and ABSORB Heat recover independently over six hours and scale to starting burden.
- Minimum meaningful size is 0.1% of start.
- Per-opponent impact credit stops one pair from farming unlimited tie-break activity.
- Five achievement badges preserve interesting losing stories without changing rewards.

### Redirect visibility boundary

The v3 baseline does not pretend public state is secret. Guard balances and armed lanes are visible both in Solana accounts and in the Strategy Room; removing them only from the interface would give RPC-reading bots privileged information.

The partial-insurance model supplies the intended tradeoff without that asymmetry: players must first ABSORB score-negative, six-hour-locked DUMP; only half becomes Guard; total Guard is capped at 25% of starting burden; and a hit redirects only `min(incoming, guard)`, with every excess unit landing normally. The triggering hit consumes Guard and disarms the lane.

A true Minesweeper-style hidden ricochet remains a separate protocol experiment. A trustless version would need an indistinguishable pre-epoch commitment, pending attacks held through a defender reveal window, a no-new-attacks settlement buffer, and permissionless expiry. That makes every attack slower, creates liveness and account-rent concerns, and rewards always-online automation. It should be considered only as an explicit simulation-backed alternative, not implemented as cosmetic UI secrecy.

### Fair scoring and rewards

- The exact weighted integral prevents sampling/timing games.
- A 99%-epoch burden remains roughly 98% in the final score even after a last-moment exit.
- Unclaimed allocations score from the active boundary, removing “never click claim” as a winning strategy.
- Revealing and actually acting are required for ranking and the activity refund.
- Winners are the lowest 5% of eligible players, capped to the bounded room design.
- Exact score ties reward demonstrated breadth/impact before committed randomness.
- GLORY issuance scales sublinearly with real participation and halves by era.
- Player rank weights and keeper claims conserve the complete committed pool exactly.

### Liveness and cleanup

- All transitions are permissionless.
- The activity bond funds one fixed SOL bounty for every registered player's settlement.
- Cancellation returns full bonds when a room never reaches viable registration/reveal thresholds.
- Player GLORY, keeper GLORY, and bond refunds are one-time pull claims.
- Expired excess bond funds can move forward without draining account rent.
- Rivalry, player, lane, and keeper accounts can return rent after safe terminal conditions.

### Client safety and presentation

- Generated IDL types bind the client to all 25 instructions.
- Commitment bytes share a fixed Rust/TypeScript hash vector.
- Reveal secrets are saved before submission, automatically downloaded, and restorable only after local and on-chain commitment verification.
- Human amount parsing never uses floating point.
- Account scans use an epoch memcmp filter instead of reading all history.
- Anchor/Solana wallet code lazy-loads only in live mode; the default demo boots from a small chunk.
- Live mode refuses to invent a battle history when no indexer is configured.
- Locked transitive overrides remove the current Anchor/Web3 TOML and JSON-RPC advisory findings while preserving the tested client API.

## Important fixes during the rewrite

### Planning allocation timestamp

The first Solana implementation draft initialized a lane's score timestamp at the future active boundary and then tried to checkpoint it at the earlier planning timestamp. That is correctly rejected as time moving backwards, which would have made normal planning claims fail. Allocation now clamps the first checkpoint to the active boundary, while a late active claim still accrues the missed interval. A regression test covers both cases.

### Rent footprint

Initial Anchor accounts were generously overallocated. Removing event-derivable volume counters and using serialization-backed exact sizes now removes 503 unnecessary bytes from each player's player-plus-four-lane footprint, plus 148 bytes from each epoch and 43 bytes from every rivalry account. Keeper and protocol accounts also shrink. This reduces rent and hot-path writes without changing game outcomes.

### Historical account scans

The first wallet draft loaded every player and lane across every epoch and filtered in JavaScript, making performance degrade forever. RPC memcmp filters now select only the current epoch at the first post-discriminator field, and projected scores reuse per-owner lane groups rather than rescanning all lanes for every player.

### Reveal recovery

The first wallet draft saved the reveal secret only after RPC confirmation and had no restore UI. A confirmed transaction followed by a client/RPC failure could therefore strand the commitment. The client now reuses any existing epoch secret, persists it before transaction submission, downloads a versioned backup, and validates restored files against the on-chain commitment.

## What was removed

### Bridge and gatekeeper

The old bridge authorization was unsafe and a cross-chain DUMP asset contradicts epoch-contained game state. No bridge or external DUMP mint exists in v3.

### DUMP token trading and transfer fee

A standard DUMP token permits integrations that do not understand active epochs, Heat, Guard, locks, or scoring. It also makes a worthless-by-design burden look like the investment object. DUMP is now internal state, and the 0.3% fee is gone from the hot action path.

### Fee pot, oracle, buyback, and burn

The old TWAP/liquidity/swap code was placeholder logic, and automatic swaps add price manipulation, compute, account contention, failure modes, and transaction cost to a game that needs cheap actions. GLORY has a capped emission curve; any external market operates independently.

### Administrative pause and on-chain bug reports

An unrestricted emergency pause contradicted the no-admin claim, while public vulnerability text exposes users before triage. Neither exists. Operational disclosure stays private and program immutability is a deployment decision made only after review.

### EVM compatibility layer

Hardhat configuration, Solidity contracts/tests, ethers frontend code, Base deployment scripts, `package-lock.json`, and EVM-specific documentation were deleted. Git history remains the compatibility archive; mixed runtimes would make the repository's actual authority ambiguous.

## What remains intentionally unresolved

1. Commit/reveal still permits costly selective withholding and must not be described as unbiased randomness.
2. The optimized SBF build and a real local-validator bootstrap transaction pass under Solana 4.1.2; the full multi-wallet lifecycle and failure-path validator suite remains a release gate.
3. Bot-driven economic testing has not begun; the user explicitly reserved it for after this implementation commit.
4. The live feed needs a fork-aware event indexer.
5. Parameter choices are reasoned starting points, not simulation-validated equilibria.
6. GLORY liquidity, listings, price, and integrations are external and unspecified.
7. Session delegates need adversarial automation tests before use with unattended bots.
8. Independent program/frontend audits, reproducible deployment, incident response, and legal review remain mandatory.

The correct designation for this commit is **implemented Solana development prototype, quality and economics unproven**.
