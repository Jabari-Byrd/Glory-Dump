# DUMP/GLORY Deployment Guide

This guide will walk you through deploying the DUMP/GLORY token system to Base network.

## Prerequisites

1. **Node.js 18+** installed
2. **MetaMask** or another Web3 wallet
3. **Base testnet ETH** for deployment (get from [Base Faucet](https://www.coinbase.com/faucets/base-ethereum-goerli-faucet))
4. **Private key** for deployment account

## Setup

### 1. Install Dependencies
```bash
npm install
```

### 2. Environment Setup
Create a `.env` file in the root directory:
```bash
PRIVATE_KEY=your_private_key_here
BASESCAN_API_KEY=your_basescan_api_key_here  # Optional, for verification
```

### 3. Compile Contracts
```bash
npm run compile
```

## Deployment Steps

### Step 1: Deploy to Base Testnet (Recommended First)

1. **Get testnet ETH**:
   - Visit [Base Faucet](https://www.coinbase.com/faucets/base-ethereum-goerli-faucet)
   - Connect your wallet and request testnet ETH

2. **Deploy contracts**:
   ```bash
   npm run deploy:testnet
   ```

3. **Verify deployment**:
   - Check the console output for contract addresses
   - Verify contracts on [Base Goerli Explorer](https://goerli.basescan.org/)

### Step 2: Test the System

1. **Run tests**:
   ```bash
   npm test
   ```

2. **Manual testing**:
   - Connect to the frontend (update contract addresses in `frontend/app.js`)
   - Test wallet connection
   - Test staking for participation
   - Test DUMP transfers
   - Test epoch finalization

### Step 3: Deploy to Base Mainnet

⚠️ **WARNING**: Mainnet deployment is irreversible. Test thoroughly on testnet first!

1. **Get mainnet ETH**:
   - Ensure you have enough ETH on Base mainnet for deployment

2. **Deploy contracts**:
   ```bash
   npm run deploy
   ```

3. **Verify contracts**:
   ```bash
   npx hardhat verify --network base CONTRACT_ADDRESS [constructor_args]
   ```

## Contract Addresses

After deployment, you'll get addresses like:
```
DUMP Token: 0x1234...
GLORY Token: 0x5678...
FeePot: 0x9abc...
BridgeGatekeeper: 0xdef0...
```

## Frontend Setup

### 1. Update Contract Addresses
Edit `frontend/app.js` and update the contract addresses:
```javascript
this.contractAddresses = {
    dumpToken: '0x1234...', // Your deployed DUMP token address
    gloryToken: '0x5678...', // Your deployed GLORY token address
    feePot: '0x9abc...'      // Your deployed FeePot address
};
```

### 2. Deploy Frontend
You can deploy the frontend to:
- **Vercel**: Drag and drop the `frontend` folder
- **GitHub Pages**: Push to a GitHub repository
- **IPFS**: Use a service like Fleek or Pinata

## Post-Deployment Checklist

### ✅ Smart Contracts
- [ ] All contracts deployed successfully
- [ ] Contract addresses recorded
- [ ] Contracts verified on block explorer
- [ ] Initial supply minted correctly
- [ ] Fee pot initialized

### ✅ Frontend
- [ ] Contract addresses updated in `app.js`
- [ ] Frontend deployed and accessible
- [ ] Wallet connection working
- [ ] All UI elements functional

### ✅ Testing
- [ ] Staking for participation works
- [ ] DUMP transfers with cooldowns work
- [ ] Demurrage calculation correct
- [ ] Epoch finalization works
- [ ] Leaderboard updates correctly

### ✅ Security
- [ ] No admin keys in production
- [ ] Bridge gatekeeper configured
- [ ] Emergency pause functions tested
- [ ] Bug bounty claimable

## Important Notes

### 🔒 Security Considerations
- **Never share your private key**
- **Test thoroughly on testnet first**
- **Verify all contract addresses**
- **Keep deployment account secure**

### 💰 Gas Optimization
- Base network has low gas fees
- Deployment should cost < $10 in ETH
- Consider gas optimization for user transactions

### 🌉 Bridge Integration
- Update bridge contract address in `BridgeGatekeeper`
- Test cross-chain transfers
- Configure bridge permissions

## Troubleshooting

### Common Issues

1. **"Insufficient funds"**
   - Get more testnet/mainnet ETH

2. **"Contract verification failed"**
   - Check constructor arguments
   - Ensure compiler version matches

3. **"Transaction failed"**
   - Check gas limits
   - Verify contract addresses
   - Check user permissions

4. **"Frontend not connecting"**
   - Verify contract addresses in `app.js`
   - Check network configuration
   - Ensure MetaMask is on correct network

### Getting Help

- **Discord**: [Join our community](https://discord.gg/dumpglory)
- **GitHub Issues**: Report bugs and issues
- **Documentation**: Check the README for details

## Next Steps

After successful deployment:

1. **Community Launch**:
   - Announce on social media
   - Share contract addresses
   - Encourage community testing

2. **Liquidity Provision**:
   - Create DUMP/ETH pool on Uniswap
   - Create GLORY/ETH pool on Uniswap
   - Let market set initial prices

3. **Monitoring**:
   - Monitor contract interactions
   - Track epoch progress
   - Watch for potential issues

4. **Iteration**:
   - Gather community feedback
   - Plan future improvements
   - Consider additional features

---

**Remember**: This is an experimental token system. Deploy responsibly and always test thoroughly! 🎭