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
