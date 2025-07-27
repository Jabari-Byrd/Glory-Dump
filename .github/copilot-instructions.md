# GLORY/DUMP Development Guide

## Architecture Overview

This is a "reverse wealth" PvP meme token game where holding LESS DUMP tokens ranks you higher. The system consists of four interconnected contracts:

- **`DumpToken.sol`** - Core ERC20 with epoch-based gameplay, cooldown mechanics, and theft systems
- **`GloryToken.sol`** - Reward token distributed to winners + bug bounty system  
- **`FeePot.sol`** - Fee collection, Uniswap integration, and automated GLORY buyback/burn
- **`BridgeGatekeeper.sol`** - Cross-chain transfer validation with rate limiting

## Key Game Mechanics

### Epoch System (30-day cycles + 7-day waiting periods)
- Players sign up during waiting periods with escalating fees (0.01-1 ETH based on timing)
- At epoch start, all participants receive random DUMP amounts (1-10B tokens)
- Goal: Maintain lowest time-weighted average DUMP balance to win GLORY rewards

### Transfer Mechanics with Cooldowns
- Both transfers and thefts incur 0.3% fees collected by `FeePot`
- **Amount-scaled cooldowns**: `cooldown = sqrt(amount) * scalingFactor`
- **Separate cooldowns**: `giveCooldownEndTime` (transfers) vs `takeCooldownEndTime` (thefts)
- Late-epoch actions can lock players out for remainder of game

### Critical State Management
- `isActiveParticipant[address]` - Determines who can play
- `isWaitingPeriod` - Controls when signups vs gameplay is allowed
- `nextEpochStartTime` - Epoch transition timing
- Time-weighted averages calculated via integral tracking

## Development Workflows

### Local Development
```bash
npm run compile      # Compiles all contracts
npm test            # Runs comprehensive test suite (530 lines)
```

### Deployment (Base Network)
```bash
npm run deploy:testnet   # Base Goerli testnet
npm run deploy          # Base mainnet
```

**Critical**: Update contract addresses in `frontend/app.js` after deployment.

### Testing Patterns
Tests extensively use time manipulation helpers:
```javascript
await fastForward(seconds)  # EVM time travel
await signup(user, fee)     # Helper for epoch signup
await startNextEpoch()      # Transition between epochs
```

## Project-Specific Conventions

### Contract Interaction Patterns
- Always check `isWaitingPeriod` and `isActiveParticipant` before state changes
- Use `nonReentrant` modifier on all external value transfers
- Fee collection happens via `FeePot.collectFee()` callback pattern

### Solidity Patterns
- Heavy use of time-based calculations: `block.timestamp >= epochStartTime + EPOCH_DURATION`
- OpenZeppelin imports: ReentrancyGuard, Ownable, ERC20 base contracts
- Constants defined with clear units: `30 days`, `1 hours`, basis points for percentages

### Frontend Integration
- Uses ethers.js v5 (not v6) - see `frontend/index.html` CDN import
- Contract ABIs auto-generated in `artifacts/` directory after compilation
- Real-time epoch countdown and balance tracking required

### Gas Optimization Focus
- Hardhat config uses aggressive optimization: `runs: 500`, `viaIR: true`
- Yul optimizer enabled for high-frequency game transactions
- Batch operations preferred for multi-user epoch transitions

## Critical Integration Points

### Uniswap V2 Integration (FeePot)
- TWAP price feeds for automated buybacks
- Hardcoded Base network addresses in deployment script
- Circuit breaker patterns for emergency pauses

### Cross-Chain Bridge (Base-focused)
- Single canonical bridge enforced by `BridgeGatekeeper`
- Rate limiting: max 1M DUMP per user per epoch
- 1-hour cooldowns between bridge transfers

### Bug Bounty System (GloryToken)
- On-chain bug report submission with severity levels
- Automated GLORY payouts: 10K (LOW) to 100K (CRITICAL)
- Structured reporting: description + proof-of-concept required

## Common Pitfalls

- **Epoch timing**: Ensure proper state transitions between waiting/game periods
- **Cooldown conflicts**: Check both give/take cooldowns before transfers
- **Active participant validation**: Many functions require `isActiveParticipant[msg.sender]`
- **Fee calculation edge cases**: Handle zero amounts and overflow in percentage calculations
- **Frontend sync**: Contract addresses must be manually updated after deployment

## Testing Anti-Patterns to Avoid

- Don't test during waiting periods without proper epoch setup
- Always use `fastForward()` helper instead of raw EVM time manipulation  
- Test cooldown mechanics with realistic amounts (small amounts = minimal cooldowns)
- Remember random DUMP assignment only happens for signed-up participants
