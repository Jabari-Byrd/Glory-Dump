# Security policy

GLORY/DUMP is experimental game and token software. It has not received an independent audit, formal verification, economic audit, or legal review. Do not deploy it with real-value expectations.

## Reporting a vulnerability

Do not publish an exploit, proof of concept, wallet secret, reveal secret, or transaction-signing request in an issue, on-chain memo, or public game feed.

Until a dedicated private disclosure address and signed policy are published, contact the repository owner privately through their verified GitHub profile and share only enough information to establish a secure channel. A real bounty program must define scope, severity, safe harbor, response times, identity, and funded payouts before accepting reports.

## Security invariants implemented

- DUMP is account state, not an SPL token. No generic transfer, approval, AMM, bridge, or external mint path can bypass epoch rules.
- The GLORY mint authority is the protocol PDA. The program exposes only committed player and keeper reward claims and enforces a 10,000,000 GLORY lifetime cap.
- No instruction accepts an administrator, arbitrary mint authority, pause authority, oracle, treasury, or configurable ruleset.
- Registration, allocation, and actions are constant-size. Settlement handles exactly one player per transaction; the winner structure is bounded below 10 KiB.
- Every score-affecting balance change checkpoints first. DUMP and REDIRECT conserve burden exactly; ABSORB only moves burden and mints no DUMP.
- DUMP and ABSORB Heat are amount-normalized, independently tracked, and capped. Every meaningful action has a nonzero minimum.
- A session delegate is limited by expiry and action count. The owner can revoke it at any time.
- Reward and bond claims are pull-based and one-time. Program-owned lamport transfers preserve each account's rent floor.
- Player, lane, rivalry, and keeper accounts have explicit post-epoch rent-recovery paths.

These properties are backed by host tests plus an SBF/local-validator bootstrap smoke, not yet by a complete lifecycle or validator adversarial suite.

## Known non-production boundaries

### Commit/reveal is not casino-grade randomness

Starting allocations use commitments from registered players. Revealed contributions are combined and domain-separated into the epoch seed. Withholding a reveal assigns that player the maximum 10B DUMP, removes reward eligibility, and forfeits most of the activity bond in a completed epoch.

The penalty discourages manipulation but does not eliminate it. A last revealer can compare “reveal” and “withhold” outcomes and sacrifice its wallet to influence collaborators' allocations. The two-player reveal minimum also permits a low-entropy but non-rewarding epoch. Resolve this with a reviewed randomness source or a formally analyzed alternative before real-value deployment.

### Sybil and collusion behavior is unproven

The 0.002 SOL activity bond is friction, not identity. Fleets can still coordinate targets, sacrifice accounts, manipulate visibility, rent many rivalry accounts, or trade favors outside the program. Pairwise impact caps protect tie-break activity from one repeated matchup; they do not claim to stop bullying or collusion.

No victim-triggered cooldown reduction exists because self-attacking Sybil wallets could convert it into a deadline escape. Whether Guard, Heat, minimum actions, eligibility, and bond economics are sufficient must be answered by the planned agent simulations.

### Economic and market risk

GLORY is freely transferable and may acquire an external market price. The program makes no representation about price, liquidity, redemption, stability, profit, or future utility. There is no protocol treasury, buyback, oracle, seed liquidity, price floor, or revenue claim.

External GLORY markets can create bribery and reward-farming incentives that are absent from unit tests. DUMP must remain excluded from those markets; wrapping account-state DUMP into a tradable asset would create a new security model.

### Upgrade authority remains operational until removed

“No admin instruction” does not mean an ordinary Solana deployment is immutable. The deployment upgrade authority can replace the executable until it is deliberately revoked. An immutable deployment should occur only after binary verification, audits, rehearsal, and incident planning because revocation is permanent.

### Session delegates are intentionally powerful

A delegate can issue any gameplay action for the player until its expiry or action limit. The limit is not a spend allowance: a single permitted action may move a large amount if Heat and balance allow it. Use a dedicated ephemeral key, authorize the shortest practical duration, and revoke it when automation stops. Bot safety will receive its own test phase.

### Frontend and indexing remain trust surfaces

The program is authoritative for transactions and final ranking. Aliases, territory visuals, projected scores, historical feeds, RPC responses, wallet extensions, and hosted JavaScript are not. Users must inspect wallet simulations and program IDs. A production frontend needs dependency review, CSP, deployment integrity, monitored indexing, and multiple RPC comparisons.

### Liveness is incentivized, not guaranteed

Phase transitions are permissionless. Each player settlement pays a fixed SOL bounty plus a pro-rata GLORY keeper claim, but no theorem guarantees that the reward covers fees or operational effort in every market condition. If nobody calls the instructions, the next phase waits.

## Removed attack surface

The Solidity prototype's bridge gatekeeper, DUMP ERC-20 surface, transfer fee, fee pot, TWAP oracle, automated swap, GLORY buyback/burn, public on-chain bug reports, and emergency pause were not ported. Several were placeholders or contradictory to the cheap high-frequency game. Reintroducing any one of them requires a separate specification, threat model, tests, and audit.

## Release policy

Localnet and Devnet are the only intended environments before the gates in `docs/TESTING.md` pass. A clean compile and green unit suite establish implemented mechanics; they do not establish economic fairness, randomness security, production availability, frontend safety, or legal compliance.
