const anchor = require("@coral-xyz/anchor");
const { PublicKey, Keypair, SystemProgram } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");

async function main() {
  // Configure the client to use the selected cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.GloryDumpGame;

  console.log("Deploying to cluster:", provider.connection.rpcEndpoint);
  console.log("Deployer wallet:", provider.wallet.publicKey.toString());
  console.log("Program ID:", program.programId.toString());

  // Check wallet balance
  const balance = await provider.connection.getBalance(provider.wallet.publicKey);
  console.log("Wallet balance:", balance / anchor.web3.LAMPORTS_PER_SOL, "SOL");

  if (balance < 0.1 * anchor.web3.LAMPORTS_PER_SOL) {
    throw new Error("Insufficient SOL balance for deployment. Need at least 0.1 SOL.");
  }

  // Derive PDAs
  const [gameStatePda, gameStateBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_state")],
    program.programId
  );

  const [dumpMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("dump_mint")],
    program.programId
  );

  const [gloryMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("glory_mint")],
    program.programId
  );

  const [treasuryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    program.programId
  );

  const [feeVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_vault")],
    program.programId
  );

  console.log("\nDerived Program Addresses:");
  console.log("Game State:", gameStatePda.toString());
  console.log("DUMP Mint:", dumpMintPda.toString());
  console.log("GLORY Mint:", gloryMintPda.toString());
  console.log("Treasury:", treasuryPda.toString());
  console.log("Fee Vault:", feeVaultPda.toString());

  try {
    // Check if game is already initialized
    try {
      const gameState = await program.account.gameState.fetch(gameStatePda);
      console.log("\nGame already initialized!");
      console.log("Current epoch:", gameState.currentEpoch.toString());
      console.log("Admin:", gameState.admin.toString());
      console.log("Is waiting period:", gameState.isWaitingPeriod);
      console.log("Is paused:", gameState.isPaused);
      return;
    } catch (err) {
      // Game not initialized, proceed with initialization
      console.log("\nInitializing new game...");
    }

    // Initialize the game
    const tx = await program.methods
      .initializeGame(gameStateBump)
      .accounts({
        admin: provider.wallet.publicKey,
        gameState: gameStatePda,
        dumpMint: dumpMintPda,
        gloryMint: gloryMintPda,
        treasury: treasuryPda,
        feeVault: feeVaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("Game initialized! Transaction signature:", tx);

    // Verify initialization
    const gameState = await program.account.gameState.fetch(gameStatePda);
    console.log("\nGame State Verification:");
    console.log("Admin:", gameState.admin.toString());
    console.log("Current epoch:", gameState.currentEpoch.toString());
    console.log("DUMP mint:", gameState.dumpMint.toString());
    console.log("GLORY mint:", gameState.gloryMint.toString());
    console.log("Treasury:", gameState.treasury.toString());
    console.log("Fee vault:", gameState.feeVault.toString());
    console.log("Is waiting period:", gameState.isWaitingPeriod);
    console.log("Is paused:", gameState.isPaused);
    console.log("Total participants:", gameState.totalParticipants.toString());

  } catch (error) {
    console.error("Deployment failed:", error);
    throw error;
  }

  console.log("\n🎉 Deployment successful!");
  console.log("\nNext steps:");
  console.log("1. Update frontend/app.js with the program ID:", program.programId.toString());
  console.log("2. Update the network endpoint if deploying to mainnet");
  console.log("3. Test the game functions through the frontend or CLI");
}

// Error handling
main().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});
  console.log("BridgeGatekeeper deployed to:", await bridgeGatekeeper.getAddress());

  // Set up permissions and relationships
  console.log("Setting up contract relationships...");
  
  // TODO: Set bridge address in DumpToken
  // TODO: Set fee pot address in DumpToken
  // TODO: Set bridge gatekeeper permissions

  console.log("Deployment complete!");
  console.log("=== Contract Addresses ===");
// Error handling
main().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});