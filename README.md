# DUMP/GLORY: The Reverse Wealth, Self-Draining, PvP Meme Token

> **"How Not to Do Money" Edition** - A token that hates being held, a leaderboard of the chronically unwealthy, a reward system for staying poor.

## 🎯 Concept

DUMP/GLORY flips the entire idea of a "wealth" token upside down:
- **The less DUMP you have, the higher you rank**
- **The more DUMP you receive, the more you must scramble to dump it on someone else**
- **Periodic leaderboards award GLORY to those who held the least DUMP**
- **It's PvP: you can sabotage others by force-feeding them DUMP, but everyone can grief back**
- **All tokenomics are hard-coded—no governance votes, no DAOs, no admin keys**

## 🏗️ Architecture

### Core Contracts

1. **`DumpToken.sol`** - The main token with demurrage mechanics
2. **`GloryToken.sol`** - Reward token for epoch winners
3. **`FeePot.sol`** - Handles fee collection and buyback mechanics
4. **`BridgeGatekeeper.sol`** - Enforces cross-chain transfer rules

### Key Features

#### 🕐 Demurrage System
- **1% daily decay**: Your balance auto-decays by 1% each day (compounded)
- **Continuous calculation**: Applied on every interaction
- **Staked amounts also decay**: No escape from the rot

#### ⏱️ Cooldown Mechanics
- **Amount-scaled cooldowns**: The more you dump, the longer you wait
- **Formula**: `cooldown = T_min + (T_max - T_min) * (amount/supply)^k`
- **Quadratic scaling**: Small dumps = short cooldown, large dumps = long cooldown

#### 🛡️ Sybil Resistance
- **Minimum stake required**: 0.05% of total supply to participate
- **Staked DUMP decays**: Makes large-scale attacks expensive
- **Withdrawable stake**: Can exit but lose decayed amounts

#### 🏆 Epoch System
- **30-day epochs**: Fixed periods for competition
- **Snapshot-based rewards**: Average DUMP held over epoch determines rank
- **Top 5% rewarded**: Logarithmic distribution of GLORY

#### 💰 Fee & Buyback System
- **0.3% transfer fee**: Collected in DUMP
- **On-chain oracle**: Uses Uniswap V2 TWAP for price feeds
- **Automated buyback**: Converts fees to GLORY and burns it

#### 🌉 Bridge Security
- **Official Base bridge only**: One canonical bridge
- **Gatekeeper enforcement**: All transfers must pass validation
- **Cooldown enforcement**: Prevents rapid cross-chain attacks

## 🚀 Quick Start

### Prerequisites
- Node.js 18+
- npm or yarn
- Hardhat

### Installation
```bash
npm install
```

### Compile Contracts
```bash
npm run compile
```

### Run Tests
```bash
npm test
```

### Deploy to Base Testnet
```bash
# Set your private key
export PRIVATE_KEY=your_private_key_here

# Deploy
npm run deploy:testnet
```

## 🎮 How to Play

### 1. Join the Game
```solidity
// Stake minimum amount to become active participant
uint256 minStake = dumpToken.getMinimumStake();
dumpToken.stakeForParticipation(minStake);
```

### 2. The Objective
- **Hold the LEAST DUMP** over the 30-day epoch
- **Dump DUMP on others** to sabotage their ranking
- **Avoid receiving DUMP** from griefers
- **Win GLORY rewards** at epoch end

### 3. Game Mechanics
```solidity
// Transfer DUMP (with cooldown and fees)
dumpToken.transfer(target, amount);

// Check your rank
int256 rank = gloryToken.getUserRank(yourAddress);

// View leaderboard
address[] memory leaders = gloryToken.getLeaderboard();
```

### 4. Epoch Finalization
```solidity
// Anyone can finalize epoch after 30 days
gloryToken.finalizeEpoch();
```

## 📊 Tokenomics

### DUMP Token
- **Initial Supply**: 1,000,000 DUMP
- **Daily Demurrage**: 1% (26% monthly)
- **Transfer Fee**: 0.3%
- **Sybil Stake**: 0.05% of supply (500 DUMP)

### GLORY Token
- **Initial Supply**: 1,000,000 GLORY
- **Bug Bounty**: 5% (50,000 GLORY)
- **Epoch Rewards**: 10,000 GLORY per epoch
- **Distribution**: Top 5% of participants

### Fee Pot
- **Source**: 0.3% of all DUMP transfers
- **Use**: Buyback GLORY from DEX
- **Burn**: All bought GLORY is burned
- **Oracle**: Uniswap V2 TWAP

## 🔧 Technical Details

### Demurrage Implementation
```solidity
// Continuous decay calculation
uint256 demurrageMultiplier = DEMURRAGE_BASIS_POINTS;
for (uint256 i = 0; i < daysSinceUpdate; i++) {
    demurrageMultiplier = demurrageMultiplier
        .mul(DEMURRAGE_BASIS_POINTS.sub(DAILY_DEMURRAGE_RATE))
        .div(DEMURRAGE_BASIS_POINTS);
}
```

### Cooldown Formula
```solidity
// Amount-scaled cooldown
uint256 scaled = amount.mul(1e18).div(totalSupply);
uint256 cooldown = tMin.add(
    tMax.sub(tMin).mul(scaled.pow(k)).div(1e18.pow(k))
);
```

### Grief Tax
```solidity
// Epoch-weighted grief multiplier
// grief_multiplier(t) = (T / t)^k
uint256 ratio = epochDuration.mul(1e18).div(timeUntilEpochEnd);
uint256 multiplier = ratio.pow(GRIEF_TAX_EXPONENT);
```

## 🛡️ Security Features

### Anti-Grief Measures
- **Epoch-weighted grief tax**: Late attacks exponentially expensive
- **Cooldown enforcement**: Prevents rapid-fire dumping
- **Sybil resistance**: Staking requirement with decay
- **Circuit breakers**: Emergency pause functions

### Bridge Security
- **Single canonical bridge**: Official Base bridge only
- **Gatekeeper validation**: All transfers checked
- **Rate limiting**: Per-address and global limits
- **Emergency pause**: Can halt bridge operations

### Oracle Security
- **On-chain TWAP**: No external dependencies
- **Liquidity checks**: Minimum pool liquidity required
- **Price movement limits**: Max 50% price change allowed
- **Fallback mechanisms**: Graceful degradation

## 🚨 Important Notes

### ⚠️ Risk Warnings
- **Experimental**: This is a novel token design
- **No investment value**: Pure utility/game token
- **High volatility**: Demurrage creates constant selling pressure
- **Complex mechanics**: May be difficult to understand

### 🔒 No Admin Controls
- **Immutable contracts**: No upgrades after deployment
- **No governance**: All parameters hard-coded
- **No admin keys**: Fully decentralized
- **Community-driven**: Success depends on adoption

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

**Remember**: This is a game, not an investment. The goal is to have the LEAST DUMP, not the most. Welcome to the reverse wealth experiment! 🎭
