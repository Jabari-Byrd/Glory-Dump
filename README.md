# GLORY/DUMP: The Reverse Wealth, PvP Meme Token

> **"How Not to Do Money" Edition** - A token that hates being held, a leaderboard of the chronically unwealthy, a reward system for staying poor.

## 🎯 Concept

GLORY/DUMP flips the entire idea of a "wealth" token upside down:
- **The less DUMP you have, the higher you rank**
- **The more DUMP you receive, the more you must scramble to dump it on someone else**
- **Periodic leaderboards award GLORY to those who held the least DUMP (on average!)**
- **It's PvP: you can sabotage others by force-feeding them DUMP, but everyone can grief back**
- **All tokenomics are hard-coded—no governance votes, no DAOs, no admin keys**

## 🏗️ Architecture

### Core Contracts

1. **`DumpToken.sol`** - The main token with PvP mechanics and epoch resets
2. **`GloryToken.sol`** - Reward token for epoch winners
3. **`FeePot.sol`** - Handles fee collection and buyback mechanics
4. **`BridgeGatekeeper.sol`** - Enforces cross-chain transfer rules

### Key Features

#### 🕐 Epoch System & Waiting Period
- **30-day epochs**: Fixed periods for competition
- **7-day waiting period**: After each epoch, a 7-day "lobby" lets new players sign up for the next round
- **Sign-up fee**: The later you join during the waiting period, the higher the fee (from 0.01 ETH up to 1 ETH)
- **Inactive participants expire**: Only those who sign up for the next epoch are included

#### 🎲 Random DUMP Assignment
- **At epoch start, all active participants receive a random amount of DUMP** (from 1 to 10 billion DUMP)
- **Very large supply**: There is always enough DUMP for any number of players
- **No demurrage**: DUMP does not decay over time; your challenge is to dump it before the epoch ends

#### 🦹‍♂️ Theft & Transfer System
- **Steal DUMP from others**: Force tokens from any active participant
- **0.3% fee**: Both transfers and thefts incur a 0.3% fee
- **Amount-scaled cooldowns**: The more you dump or steal, the longer you must wait before acting again
- **Epoch-aware cooldowns**: Big moves late in the epoch can lock you out for the rest of the game
- **Action-specific cooldowns**: Stealing and giving have separate cooldowns—chain your chaos!

#### 🏆 Ranking & Rewards
- **Time-weighted average DUMP**: Your rank is based on the average amount of DUMP you held during the epoch (not just your final balance)
- **Late joiners are penalized**: Their average starts high, so it's hard to win by joining late
- **GLORY rewards**: At epoch end, GLORY is distributed to those with the lowest average DUMP
- **Bonus Epochs**: Special epochs with extra GLORY rewards, triggered by rare on-chain events

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

### 1. Join the Game (During Waiting Period)
```solidity
// Sign up for the next epoch (paying the join fee)
dumpToken.signupForNextEpoch{value: fee}();
```

### 2. The Objective
- **Hold the LEAST DUMP (on average) over the 30-day epoch**
- **Dump DUMP on others** to sabotage their ranking
- **Avoid receiving DUMP** from griefers
- **Win GLORY rewards** at epoch end

### 3. Game Mechanics
```solidity
// Transfer DUMP (with cooldown and fees)
dumpToken.transfer(target, amount);

// Steal DUMP from others (with cooldown, fees, and costs)
dumpToken.stealDump(victim, amount);

// Check your average DUMP
dumpToken.getAverageDump(yourAddress);

// Check your rank (via GloryToken)
int256 rank = gloryToken.getUserRank(yourAddress);

// View leaderboard
address[] memory leaders = gloryToken.getLeaderboard();
```

### 4. Epoch Finalization & Reset
```solidity
// Anyone can finalize epoch after 30 days
dumpToken.finalizeEpoch();
// 7-day waiting period begins; sign up for next round!
// After waiting period, anyone can start the next epoch:
dumpToken.startNextEpoch();
```

## 📊 Tokenomics

### DUMP Token
- **Very large supply**: Always enough for all players
- **No demurrage**: DUMP does not decay
- **Transfer/steal fee**: 0.3%
- **Random assignment**: Each epoch, all active players get a random DUMP amount

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

### Random DUMP Assignment
```solidity
// At epoch start, each participant gets:
uint256 rand = uint256(keccak256(abi.encodePacked(blockhash(block.number-1), user, i, block.timestamp)));
uint256 amount = (rand % MAX_DUMP_PER_PLAYER) + 1 * 10**18;
```

### Average DUMP Calculation
```solidity
// On every balance change:
cumulativeDumpTime += lastBalance * (now - lastUpdateTime);
lastUpdateTime = now;
lastBalance = balanceOf(user);
// At epoch end:
average = cumulativeDumpTime / (now - epochStartTime);
```

### Cooldown Formula
```solidity
// Amount-scaled cooldown, epoch-aware
uint256 scaled = amount * 1e18 / _totalSupply;
uint256 scaledCubed = scaled * scaled / 1e18;
scaledCubed = scaledCubed * scaled / 1e18;
uint256 cooldown = tMin + (epochTimeLeft * scaledCubed / 1e18);
```

## 🛡️ Security Features

### Anti-Grief Measures
- **Epoch-aware cooldowns**: Big moves late in the epoch are heavily penalized
- **Sybil resistance**: High join fee for late joiners, average-based ranking
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
- **High volatility**: Expect wild swings in DUMP balances
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

**Remember**: This is a game, not an investment. The goal is to have the LEAST DUMP (on average), not the most. Welcome to the reverse wealth experiment! 🎭
