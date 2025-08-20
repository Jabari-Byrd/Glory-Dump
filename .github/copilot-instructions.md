# Glory/Dump AI Coding Agent Instructions

This document provides guidance for AI coding agents working on the Glory/Dump project.

## 1. Big Picture: Solana-Based Reverse Wealth Game

The core of this project is `glory-dump-game`, a Solana Anchor program written in Rust. It's a "reverse wealth" game where the goal is to have the *lowest* average balance of a token called `DUMP`.

- **`programs/glory-dump-game`**: Contains the on-chain game logic.
- **`tests/`**: Contains TypeScript tests that simulate player interactions.
- **`frontend/`**: A simple vanilla JS frontend for interacting with the game.
- **`scripts/`**: Contains deployment and initialization scripts.

The game operates in epochs. Players sign up, receive a random amount of `DUMP`, and then try to get rid of it before the epoch ends. Winners are rewarded with `GLORY` tokens.

## 2. Key Files and Directories

- **`programs/glory-dump-game/src/lib.rs`**: The main entry point for the Anchor program. It defines the program's public interface.
- **`programs/glory-dump-game/src/instructions/`**: This directory contains the core logic for each of the program's instructions (e.g., `transfer.rs`, `claims.rs`, `epoch.rs`). Each file corresponds to a specific action a player can take.
- **`programs/glory-dump-game/src/state.rs`**: Defines the data structures used to store the game's state on-chain (e.g., `GameState`, `PlayerState`).
- **`tests/glory-dump-game.ts`**: The primary integration test file. It's a great place to see how the different program instructions are used in practice.
- **`Anchor.toml`**: The configuration file for the Anchor project. It defines the program's dependencies and other settings.

## 3. Developer Workflow

The primary development workflow involves editing the Rust program, building it, and then running the TypeScript tests.

**Build the program:**
```bash
anchor build
```

**Run the tests:**
```bash
anchor test
```

**Deploy to Devnet:**
```bash
anchor deploy --provider.cluster devnet
```

The tests in `tests/glory-dump-game.ts` are the best way to understand how to interact with the program from a client-side application.

## 4. Project-Specific Conventions

- **Time-Weighted Average DUMP**: A player's score is not their final `DUMP` balance, but their average balance over the entire epoch. This logic is primarily handled in `programs/glory-dump-game/src/instructions/tracking.rs`.
- **Instruction-Level Logic**: Each player action is encapsulated in its own file in the `instructions` directory. This is a standard Anchor pattern.
- **Fees**: A 0.3% fee is taken on all `DUMP` transfers and thefts. This is a key part of the game's tokenomics.
- **Vanilla JS Frontend**: The frontend is intentionally simple. It uses the `@solana/web3.js` library to interact directly with the Solana network. There is no complex framework.

When working on this project, please adhere to these conventions.

## 5. Maintaining a Clean Workspace

The `anchor build` command creates a `target` directory with build artifacts. This directory can become large over time. If you need to free up space or ensure a completely fresh build, you can run:

```bash
anchor clean
```

This will remove the `target` directory. Remember to run `anchor build` again before running tests.


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
await new Promise((resolve) => setTimeout(resolve, 1000));
// Account creation patterns
const [gameState] = anchor.web3.PublicKey.findProgramAddressSync(
  [GAME_STATE_SEED],
  program.programId,
);
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
