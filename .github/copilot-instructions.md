## Copilot instructions for Glory-Dump

Purpose: Make AI agents productive quickly in this Solana/Anchor project by capturing the repo’s architecture, workflows, and house patterns. Keep answers concrete and tie to these files.

Project snapshot
- Stack: Solana + Anchor (Rust) program with Node/TypeScript tests and a minimal JS frontend.
- Key paths: `programs/glory-dump-game/src/{lib.rs,constants.rs,errors.rs,state.rs,instructions/}`, `Anchor.toml`, `package.json`, `scripts/{deploy.js,initialize.js}`, `tests/**/*.ts`, `frontend/app.js`.
- Program ID: placeholder `GDgame111…` in `lib.rs::declare_id!` and `Anchor.toml`. Must be replaced after real deploys.

Architecture and data model
- Single Anchor program `glory_dump_game` exposes handlers in `lib.rs` (initialize_game, epoch lifecycle, transfer/steal, tracking, rewards/claims, bug bounty, admin pause).
- Accounts in `state.rs`:
	- `GameState` (globals, epoch timings, mints, vaults, supply counters).
	- `EpochState` (participants, finalization, metrics, rewards/merkle root flags).
	- `PlayerState` (stake, balances, cooldowns, time-weighted sum, stats).
	- `ClaimStatus`, `BugReport` plus enums `RewardTier`, `BugSeverity`.
- PDA seeds in `constants.rs` (e.g., `GAME_STATE_SEED`, `PLAYER_STATE_SEED`, `DUMP_MINT_SEED`, `GLORY_MINT_SEED`, `FEE_VAULT_SEED`, `TREASURY_SEED`, `CLAIM_STATUS_SEED`). Derive with these exact byte seeds.
- Token decimals: DUMP=6, GLORY=9. GLORY supply cap enforced via `GLORY_SUPPLY_CAP`.

Conventions and patterns
- Keep PDA seed usage consistent across instructions and tests (see `tests/glory-dump-game.ts` and `scripts/*.js` for examples using `findProgramAddressSync`).
- Every account struct defines a static size constant (`LEN`/`MAX_LEN`) for rent; update if fields change.
- Time-weighted average: update before any balance change via the tracking instruction; persisted in `PlayerState.time_weighted_sum` and `last_update_time`.
- Cooldowns are tracked separately for give/take (`give_cooldown_end_time`, `take_cooldown_end_time`).
- Errors are centralized in `errors.rs` and reused across handlers.

Build, test, and run
- Build: `anchor build` or `npm run build`.
- Tests: `anchor test` or `npm test` (ts-mocha). Tests assume local validator and derive PDAs from the same seeds.
- Local validator: `npm run localnet` (then set `solana config set --url localhost` as needed).
- Deploy: `npm run deploy[:devnet|:mainnet]` (Anchor), then initialize on-chain state with `npm run initialize` (Node script calling `initializeGame`).
- After deploy: update program id in three places: `lib.rs::declare_id!`, `Anchor.toml [programs.<cluster>].glory_dump_game`, and `frontend/app.js` (`programId`). Rebuild after changing Rust ids.

Integration points
- Tokens: Program-controlled DUMP/GLORY mints are PDAs; fees collected to `FEE_VAULT_SEED` and SOL join fees to `TREASURY_SEED`.
- Rewards: Merkle-root based claim flow is represented in `EpochState` (root + flag) and `ClaimStatus`; admin sets root; users claim with proof.
- Frontend (`frontend/app.js`): vanilla web3.js + Phantom. It derives PDAs with the same byte seeds; it’s a scaffold—actual IDL-driven deserialization is not wired.
- Node scripts (`scripts/deploy.js`, `scripts/initialize.js`): show canonical PDA derivations and program method calls via Anchor workspace.

Examples to mirror
- PDA derivation (TS):
	`[gameState, bump] = PublicKey.findProgramAddressSync([Buffer.from('game_state')], programId)`
- Player state PDA (TS):
	`[playerState] = PublicKey.findProgramAddressSync([Buffer.from('player_state'), playerPk.toBuffer()], programId)`
- Transfer fee calc matches constants: `TRANSFER_FEE_BASIS_POINTS = 30` (0.3%). Tests assert example math.

Gotchas
- Provider cluster in `Anchor.toml` defaults to Localnet; keep CLI and tests on the same cluster.
- Keep program id in sync across Rust, Anchor.toml, and frontend to avoid “account not found”/IDL mismatch.
- Account sizes: changing vectors or max participants requires revisiting `MAX_LEN` and may impact rent.

When adding/changing instructions
- Add a module under `src/instructions/`, define `#[derive(Accounts)]` context with exact seed constraints, export handler; wire it in `lib.rs` under `#[program]`.
- Use existing seeds and error variants; update account size constants as needed; add/adjust tests in `tests/**/*.ts`.

Where to look first
- `constants.rs` for seeds/parameters, `state.rs` for data shapes, `lib.rs` to find entrypoints, `scripts/*.js` and `tests/**/*.ts` for canonical client patterns, and `Anchor.toml`/`package.json` for commands.

# GLORY/DUMP Solana Development Guide

## Architecture Overview

This is a "reverse wealth" PvP meme token game built on Solana where holding LESS DUMP tokens ranks you higher. The system is implemented as a single Anchor program (`glory-dump-game`) containing all game logic:

- **Core Game Logic** - Epoch management, participation staking, and time-weighted average tracking
- **Token Mechanics** - DUMP/GLORY SPL token operations with transfer/theft cooldowns
- **Reward Distribution** - Automated GLORY payouts to epoch winners based on lowest DUMP averages
- **Bug Bounty System** - On-chain bug report submission with severity-based payouts

## Key Game Mechanics

### Epoch System (30-day cycles + 7-day waiting periods)
- Players stake DUMP during waiting periods with escalating fees (0.01-1 SOL based on timing)
- At epoch start, all participants receive random DUMP amounts (1M-10B tokens via on-chain randomness)
- Goal: Maintain lowest time-weighted average DUMP balance to win GLORY rewards

### Transfer Mechanics with Solana-Optimized Cooldowns
- Both transfers and thefts incur 0.3% fees (see `TRANSFER_FEE_BASIS_POINTS` in `constants.rs`)
- **Amount-scaled cooldowns**: Min 15s transfers, 30s thefts; Max 30m/1h respectively
- **Separate cooldown tracking**: Different PDAs for give/take actions per player
- Late-epoch actions can lock players out due to cooldown extending past epoch end

### Critical State Management (Anchor Account Structure)
- `GameState` - Global state with epoch timing, participant counts, admin controls
- `EpochState` - Per-epoch participant list, finalization status, reward pools
- `PlayerState` - Individual balances, averages, cooldowns, staking status
- All state uses Program Derived Addresses (PDAs) for deterministic account generation

## Development Workflows

### Local Development
```bash
anchor build         # Compiles Rust program to BPF bytecode
anchor test          # Runs TypeScript test suite with local validator
npm run localnet     # Starts solana-test-validator for development
```

### Deployment (Solana Networks)
```bash
anchor deploy --provider.cluster devnet     # Solana Devnet
anchor deploy --provider.cluster mainnet    # Solana Mainnet-Beta
```

**Critical**: Update `programId` in `frontend/app.js` and `declare_id!` in `lib.rs` after deployment.

### Testing Patterns
Tests use Anchor's TypeScript client with time manipulation:
```typescript
// Time travel in Solana tests
await new Promise(resolve => setTimeout(resolve, 1000));
// Account creation patterns
const [gameState] = anchor.web3.PublicKey.findProgramAddressSync([GAME_STATE_SEED], program.programId);
// Transaction simulation before execution
```

## Project-Specific Conventions

### Anchor Program Patterns
- All instructions use `Context<T>` with account validation via `#[account]` constraints
- PDA seeds defined in `constants.rs` (e.g., `GAME_STATE_SEED`, `PLAYER_STATE_SEED`)
- Error handling via custom error enum in `errors.rs`
- Modular instruction organization in `instructions/` directory (9 modules)

### Solana-Specific Patterns
- **Account Size Calculations**: Each account struct defines `LEN` constant for rent calculation
- **SPL Token Integration**: Uses `anchor_spl` for token operations, associated token accounts
- **Clock Access**: `Clock::get()?.unix_timestamp` for all time-based logic
- **PDA Derivation**: Consistent seed patterns for cross-instruction account lookup

### Frontend Integration (Solana Web3.js)
- Uses `@solana/web3.js` v1.95.2 and `@coral-xyz/anchor` v0.30.1
- Phantom wallet integration with auto-connect on page load
- Real-time balance updates via `connection.onAccountChange()` subscriptions
- Program IDL auto-generated in `target/idl/` after compilation

## Critical Integration Points

### SPL Token Operations
- DUMP/GLORY tokens created via program-controlled mints (PDAs)
- All transfers go through program instruction handlers (not direct SPL transfers)
- Fee collection via program-controlled vault accounts
- Associated Token Account (ATA) creation handled automatically

### Time-Weighted Average Calculation
- Implemented in `instructions/tracking.rs` using cumulative time integrals
- Must call `update_player_average()` before any balance-changing operation
- Formula: `average = cumulativeSum / epochDuration` where sum tracks `balance * timeHeld`

### Bug Bounty System Integration
- On-chain bug reports stored as PDA accounts with severity enum
- Automated GLORY payouts: 10K (LOW) to 100K (CRITICAL) tokens
- References actual token amounts in `constants.rs` with 9-decimal precision

## Common Pitfalls

- **PDA Seed Collisions**: Always include user pubkey in player-specific PDA seeds
- **Account Size Limits**: Solana accounts have 10MB limit; epoch participant vectors may need pagination
- **Instruction Limits**: Complex operations may hit Solana's 1.4M compute unit limit per transaction
- **Clock Drift**: Use `Clock::get()?.unix_timestamp` consistently, not `block.timestamp`
- **Frontend Program ID Sync**: Manual update required in multiple files after redeployment
- **Test Validator Reset**: Local accounts don't persist between `anchor test` runs

## Testing Anti-Patterns to Avoid

- Don't test during waiting periods without proper epoch setup
- Always use `fastForward()` helper instead of raw EVM time manipulation  
- Test cooldown mechanics with realistic amounts (small amounts = minimal cooldowns)
- Remember random DUMP assignment only happens for signed-up participants
