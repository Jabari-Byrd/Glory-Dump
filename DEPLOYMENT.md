# GLORY/DUMP Solana Deployment GuideThis guide covers deploying the GLORY/DUMP game to Solana networks.## Prerequisites- **Rust** 1.70.0+- **Solana CLI** 1.18.17+- **Anchor Framework** 0.30.1+- **Node.js** 18+- **Phantom Wallet** (for testing)## Installation & Setup### 1. Install Solana CLI`bashsh -c "$(curl -sSfL https://release.solana.com/v1.18.17/install)"`### 2. Install Anchor`bashnpm install -g @coral-xyz/anchor-cli@0.30.1`### 3. Configure Solana CLI`bash# Set to devnet for testingsolana config set --url devnet# Generate a new keypair (save this securely!)solana-keygen new --outfile ~/.config/solana/id.json# Airdrop SOL for testingsolana airdrop 2`### 4. Install Dependencies`bashnpm install`## Development### 1. Build the Program`bashanchor build`### 2. Run Tests`bashanchor test`### 3. Start Local Validator (Optional)`bash# Terminal 1: Start local validatorsolana-test-validator# Terminal 2: Set CLI to localsolana config set --url localhost# Airdrop SOLsolana airdrop 10`## Deployment### Devnet Deployment1. **Ensure you have SOL for deployment fees**:`bashsolana balance# If low, airdrop more: solana airdrop 2`2. **Deploy the program**:`bashanchor deploy --provider.cluster devnet`3. **Initialize the game state**:`bash# You'll need to call the initialize_game instruction# This can be done through the frontend or CLIanchor run initialize-game --provider.cluster devnet`### Mainnet Deployment1. **Fund your wallet with real SOL**:`bash# Check balancesolana balance --url mainnet-beta# You'll need ~2-5 SOL for deployment`2. **Deploy to mainnet**:`bashanchor deploy --provider.cluster mainnet-beta`3. **Verify deployment**:`bashsolana program show <PROGRAM_ID> --url mainnet-beta`## Post-Deployment Setup### 1. Update Frontend ConfigurationUpdate the program ID in `frontend/app.js`:`javascriptthis.programId = new solanaWeb3.PublicKey('YOUR_DEPLOYED_PROGRAM_ID');`For mainnet, also update the endpoint:`javascriptthis.endpoint = 'https://api.mainnet-beta.solana.com';`### 2. Initialize Game StateThe first transaction after deployment should be calling `initialize_game`:`bash# Using Anchor CLI (adjust parameters as needed)anchor run initialize --provider.cluster devnet`### 3. Create Token AccountsPlayers will need DUMP and GLORY token accounts. These are created automatically when they first receive tokens, but you may want to pre-create some for testing.## Verification### 1. Verify Program Deployment`bashsolana program show <PROGRAM_ID>`### 2. Check Game State`bash# You can query the game state accountsolana account <GAME_STATE_PDA>`### 3. Test Basic FunctionsUse the frontend or write simple test scripts to verify:- Wallet connection- Token account creation - Basic game interactions## Monitoring & Maintenance### Transaction Monitoring`bash# Monitor recent transactionssolana transaction-history <WALLET_ADDRESS> --limit 10# Watch program logssolana logs <PROGRAM_ID>`### Account Monitoring`bash# Check game statesolana account <GAME_STATE_PDA># Check player statessolana account <PLAYER_STATE_PDA>`## Network Addresses### Devnet- **RPC**: `https://api.devnet.solana.com`- **Explorer**: `https://explorer.solana.com/?cluster=devnet`### Mainnet- **RPC**: `https://api.mainnet-beta.solana.com`- **Explorer**: `https://explorer.solana.com/`## Security Considerations### 1. Program Security- All admin functions are restricted to the admin pubkey- Emergency pause functionality is available- No upgrade authority after initial deployment### 2. Token Security- DUMP and GLORY mints are controlled by the program- Fee collection is automatic and transparent- No backdoors or hidden mint functions### 3. Economic Security- Time-weighted averages prevent last-minute manipulation- Cooldowns prevent spam attacks- Fees discourage dust attacks## Troubleshooting### Common Issues1. **Insufficient SOL for deployment**:

- Solution: Airdrop more SOL (devnet) or fund wallet (mainnet)

2. **Program compilation errors**:
   - Check Rust/Anchor versions
   - Run `anchor clean` and rebuild

3. **Transaction failures**:
   - Check program logs: `solana logs <PROGRAM_ID>`
   - Verify account permissions and balances

4. **Frontend connection issues**:
   - Verify program ID is correct
   - Check network endpoint
   - Ensure Phantom wallet is connected to correct network

### Getting Help

- Check Anchor documentation: https://www.anchor-lang.com/
- Solana developer resources: https://docs.solana.com/
- Discord communities for real-time help

## Gas Costs Comparison

| Action              | Ethereum (Base L2) | Solana    |
| ------------------- | ------------------ | --------- |
| Token Transfer      | ~$0.001-0.01       | ~$0.00025 |
| Complex Transaction | ~$0.01-0.10        | ~$0.001   |
| Game Interaction    | ~$0.005-0.05       | ~$0.0005  |

Solana provides **~10-100x lower costs** for the same operations, making frequent PvP interactions economically viable!
