# GLORY/DUMP: The Reverse Wealth, PvP Meme Token on Solana

> **"How Not to Do Money" Edition** - A token that hates being held, a leaderboard of the chronically unwealthy, a reward system for staying poor. Now with **ultra-low fees and lightning-fast transactions** on Solana!

## 🎯 Concept

GLORY/DUMP flips the entire idea of a "wealth" token upside down:
- **The less DUMP you have, the higher you rank**
- **The more DUMP you receive, the more you must scramble to dump it on someone else**
- **Periodic leaderboards award GLORY to those who held the least DUMP (on average!)**
- **It's PvP: you can sabotage others by force-feeding them DUMP, but everyone can grief back**
- **All tokenomics are hard-coded—no governance votes, no DAOs, no admin keys**
- **Built on Solana for virtually free transactions and instant settlement**

## 🏗️ Architecture

### Core Programs

1. **`glory-dump-game`** - Main Anchor program containing all game logic
   - DUMP token mechanics with PvP features and epoch resets
   - GLORY token rewards for epoch winners
   - Fee collection and management
   - Bug bounty system integration

### Key Features

#### ⚡ Solana Advantages
- **Ultra-low fees**: Transactions cost fractions of a penny instead of dollars
- **Instant settlement**: No waiting for block confirmations
- **High throughput**: Handle thousands of DUMP transfers and thefts per second
- **Built-in SPL token support**: Native token functionality without custom implementations

#### 🕐 Epoch System & Waiting Period
- **30-day epochs**: Fixed periods for competition
- **7-day waiting period**: After each epoch, a 7-day "lobby" lets new players sign up for the next round
- **Sign-up fee**: The later you join during the waiting period, the higher the fee (from 0.01 SOL up to 1 SOL)
- **Inactive participants expire**: Only those who sign up for the next epoch are included

#### 🎲 Random DUMP Assignment
- **At epoch start, all active participants receive a random amount of DUMP** (from 1 to 10 billion DUMP)
- **Very large supply**: There is always enough DUMP for any number of players
- **No demurrage**: DUMP does not decay over time; your challenge is to dump it before the epoch ends

#### 🦹‍♂️ Theft & Transfer System
- **Steal DUMP from others**: Force tokens from any active participant
- **0.3% fee**: Both transfers and thefts incur a 0.3% fee (collected in DUMP)
- **Amount-scaled cooldowns**: The more you dump or steal, the longer you must wait before acting again
- **Epoch-aware cooldowns**: Big moves late in the epoch can lock you out for the rest of the game
- **Action-specific cooldowns**: Stealing and giving have separate cooldowns—chain your chaos!

#### 🏆 Ranking & Rewards
- **Time-weighted average DUMP**: Your rank is based on the average amount of DUMP you held during the epoch (not just your final balance)
- **Late joiners are penalized**: Their average starts high, so it's hard to win by joining late
- **GLORY rewards**: At epoch end, GLORY is distributed to those with the lowest average DUMP
- **Bonus Epochs**: Special epochs with extra GLORY rewards, triggered by rare on-chain events

#### 💰 Fee & Reward System
- **0.3% transfer fee**: Collected in DUMP tokens
- **On-chain oracle**: Uses Solana slot hashes for on-chain randomness
- **Automated GLORY**: Rewards distributed automatically via program logic

#### 🌉 Cross-Chain Future
- **Planned feature**: Cross-chain DUMP transfers will be implemented later
- **Security first**: Multi-signature validation planned
- **Rate limiting**: Transfer limits to prevent abuse

## 🚀 Quick Start

### Prerequisites
- Node.js 18+
- Rust 1.70.0+
- Solana CLI 1.18.17+
- Anchor Framework 0.30.1+

### Installation
```bash
npm install
```

### Compile Program
```bash
anchor build
```

### Run Tests
```bash
anchor test
```

### Deploy to Solana Devnet
```bash
# Set your wallet and cluster
solana config set --url devnet
solana-keygen new  # if you don't have a keypair

# Airdrop SOL for testing
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet
```

## 🎮 How to Play

### 1. Join the Game (During Waiting Period)
```typescript
// Sign up for the next epoch (paying the join fee in SOL)
await program.methods
  .signUpForEpoch(new anchor.BN(joinFee))
  .accounts({
    player: wallet.publicKey,
    gameState: gameStatePda,
    epochState: epochStatePda,
    playerState: playerStatePda,
    treasury: treasuryPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### 2. The Objective
- **Hold the LEAST DUMP (on average) over the 30-day epoch**
- **Dump DUMP on others** to sabotage their ranking
- **Avoid receiving DUMP** from griefers
- **Win GLORY rewards** at epoch end

### 3. Game Mechanics
```typescript
// Transfer DUMP (with cooldown and fees)
await program.methods
  .transferDump(new anchor.BN(amount))
  .accounts({
    from: wallet.publicKey,
    to: targetPlayer,
    fromPlayerState: fromPlayerStatePda,
    toPlayerState: toPlayerStatePda,
    // ... other required accounts
  })
  .rpc();

// Steal DUMP from others (with cooldown, fees, and costs)
await program.methods
  .stealDump(new anchor.BN(amount))
  .accounts({
    thief: wallet.publicKey,
    victim: victimPlayer,
    thiefPlayerState: thiefPlayerStatePda,
    victimPlayerState: victimPlayerStatePda,
    // ... other required accounts
  })
  .rpc();

// Update your time-weighted average
await program.methods
  .updatePlayerAverage()
  .accounts({
    player: wallet.publicKey,
    playerState: playerStatePda,
    gameState: gameStatePda,
    clock: SYSVAR_CLOCK_PUBKEY,
  })
  .rpc();
```

### 4. Epoch Finalization & Reset
```typescript
// Anyone can finalize epoch after 30 days
await program.methods
  .finalizeEpoch()
  .accounts({
    // Required accounts for epoch finalization
  })
  .rpc();

// 7-day waiting period begins; sign up for next round!
// After waiting period, admin can start the next epoch
await program.methods
  .startEpoch()
  .accounts({
    // Required accounts for epoch start
  })
  .rpc();
```

## 📊 Tokenomics

### DUMP Token
- **Very large supply**: Always enough for all players
- **No demurrage**: DUMP does not decay
- **Transfer/steal fee**: 0.3%
- **Random assignment**: Each epoch, all active players get a random DUMP amount

### GLORY Token
- **Supply Cap**: 1,000,000 GLORY (capped on-chain; program enforces mint cap)
- **Bug Bounty**: Fixed-tier bounties; total mints respect cap
- **Epoch Rewards**: Distributed per epoch with tier split: 40% Winner, 35% Top Tier, 20% Middle Tier, 5% Bottom Tier
- **Distribution Mechanism**: Winner direct mint; broader tiers claimed permissionlessly via on-chain Merkle proof under a per-epoch `merkle_root` with one-claim-per-epoch enforcement

### Fee Collection
- **Source**: 0.3% of all DUMP transfers
- **Collection**: Automatic via program instruction
- **Storage**: Program-controlled fee vault account (DUMP) and treasury PDA (SOL join fees)
- **Note**: Theft uses secure delegated transfers; ensure you staked to approve the game as delegate

## 🔧 Technical Details

### Random DUMP Assignment
```rust
// At epoch start, each participant gets random DUMP using on-chain randomness
let clock = Clock::get()?;
let slot = clock.slot;
let participant_seed = &[participant.key().as_ref(), &slot.to_le_bytes()];
let random_value = solana_program::hash::hash(participant_seed).to_bytes();
let amount = (u64::from_le_bytes([random_value[0], random_value[1], random_value[2], random_value[3], 0, 0, 0, 0]) % MAX_DUMP_ASSIGNMENT) + MIN_DUMP_ASSIGNMENT;
```

### Average DUMP Calculation
```rust
// On every balance change (Rust implementation in tracking.rs):
player_state.cumulative_dump_time += player_state.last_balance.checked_mul(time_delta)?;
player_state.last_update_time = current_time;
player_state.last_balance = new_balance;
// At epoch end:
// average = cumulative_dump_time / epoch_duration
```

### Cooldown Formula
```rust
// Amount-scaled cooldown, epoch-aware (simplified)
let transfer_amount_ratio = transfer_amount.checked_div(total_supply)?;
let cooldown_factor = transfer_amount_ratio.checked_mul(1000)?; // Scale factor
let base_cooldown = if is_theft { THEFT_COOLDOWN_MIN } else { TRANSFER_COOLDOWN_MIN };
let max_cooldown = if is_theft { THEFT_COOLDOWN_MAX } else { TRANSFER_COOLDOWN_MAX };
let calculated_cooldown = base_cooldown + cooldown_factor;
let final_cooldown = std::cmp::min(calculated_cooldown, max_cooldown);
```

## 🛡️ Security Features

### Anti-Grief Measures
- **Epoch-aware cooldowns**: Big moves late in the epoch are heavily penalized
- **Sybil resistance**: High join fee for late joiners, average-based ranking
- **Circuit breakers**: Emergency pause functions

### Bridge Security (Future Feature)
- **Cross-chain support**: Planned for future implementation
- **Rate limiting**: Will include per-address limits
- **Security validation**: Multi-signature requirements planned

### Oracle Security (Future Implementation)
- **On-chain randomness**: Uses Solana slot hashes for randomness
- **No external oracles**: Self-contained randomness generation
- **Deterministic**: Reproducible results for transparency

## 🚨 Important Notes

### ⚠️ Risk Warnings
- **Experimental**: This is a novel token design
- **No investment value**: Pure utility/game token
- **High volatility**: Expect wild swings in DUMP balances
- **Complex mechanics**: May be difficult to understand

### 🔒 Program Security
- **Immutable program**: Recommend deploying with upgrade authority removed for mainnet
- **Admin controls**: Limited to epoch management and emergency pause; rewards are capped and single-use per tier; join fees routed to treasury PDA
- **Open source**: All code publicly auditable
- **Bug bounty**: On-chain reward system for finding vulnerabilities with capped supply enforcement

### 💸 Zero Budget Design
- **No paid audits**: Open source + bug bounty
- **No oracles**: On-chain price feeds only
- **No keepers**: Community-driven automation
- **No seed liquidity**: Market-driven price discovery

## 📝 License

MIT License - Use at your own risk.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

---

**Remember**: This is a game, not an investment. The goal is to have the LEAST DUMP (on average), not the most. Welcome to the reverse wealth experiment! 🎭
