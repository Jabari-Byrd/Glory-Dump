const anchor = require("@coral-xyz/anchor");
const { PublicKey, SystemProgram } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");

async function initializeGame() {
  // Configure the client to use the selected cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.GloryDumpGame;

  console.log("Initializing game on cluster:", provider.connection.rpcEndpoint);
  console.log("Admin wallet:", provider.wallet.publicKey.toString());
  console.log("Program ID:", program.programId.toString());

  // Derive PDAs
  const [gameStatePda, gameStateBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_state")],
    program.programId,
  );

  const [dumpMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("dump_mint")],
    program.programId,
  );

  const [gloryMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("glory_mint")],
    program.programId,
  );

  const [treasuryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    program.programId,
  );

  const [feeVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_vault")],
    program.programId,
  );

  try {
    // Check if already initialized
    try {
      const gameState = await program.account.gameState.fetch(gameStatePda);
      console.log("Game already initialized!");
      console.log("Admin:", gameState.admin.toString());
      return;
    } catch (err) {
      // Not initialized, proceed
      console.log("Initializing new game...");
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

    // Display important addresses for frontend configuration
    console.log("\n=== IMPORTANT: UPDATE FRONTEND CONFIG ===");
    console.log("Program ID:", program.programId.toString());
    console.log("Game State PDA:", gameStatePda.toString());
    console.log("DUMP Mint:", dumpMintPda.toString());
    console.log("GLORY Mint:", gloryMintPda.toString());
    console.log("==========================================");
  } catch (error) {
    console.error("Initialization failed:", error);
    throw error;
  }
}

initializeGame().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});
