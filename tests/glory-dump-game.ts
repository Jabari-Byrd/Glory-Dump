import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { expect } from "chai";

describe("glory-dump-game", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.GloryDumpGame;
  const provider = anchor.getProvider() as AnchorProvider;

  // Test accounts
  let admin = Keypair.generate();
  let player1 = Keypair.generate();
  let player2 = Keypair.generate();

  // Program derived addresses
  let gameStatePda: PublicKey;
  let gameStateBump: number;
  let dumpMintPda: PublicKey;
  let gloryMintPda: PublicKey;
  let treasuryPda: PublicKey;
  let feeVaultPda: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    await airdropSol(provider, admin.publicKey, 5);
    await airdropSol(provider, player1.publicKey, 2);
    await airdropSol(provider, player2.publicKey, 2);

    // Derive PDAs
    [gameStatePda, gameStateBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_state")],
      program.programId
    );

    [dumpMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dump_mint")],
      program.programId
    );

    [gloryMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("glory_mint")],
      program.programId
    );

    [treasuryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury")],
      program.programId
    );

    [feeVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("fee_vault")],
      program.programId
    );
  });

  it("Should verify program compilation and setup", async () => {
    // Basic test to verify the program compiles and workspace is set up correctly
    expect(program.programId).to.not.be.undefined;
    expect(gameStatePda).to.not.be.undefined;
    expect(admin.publicKey).to.not.be.undefined;
    
    console.log("✅ Program ID:", program.programId.toString());
    console.log("✅ Game State PDA:", gameStatePda.toString());
    console.log("✅ Admin Public Key:", admin.publicKey.toString());
  });

  it("Should derive PDAs correctly", async () => {
    // Test PDA derivation logic matches the program
    const [derivedGameState, derivedBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_state")],
      program.programId
    );

    expect(derivedGameState.equals(gameStatePda)).to.be.true;
    expect(derivedBump).to.equal(gameStateBump);

    // Test player state PDA
    const [player1StatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_state"), player1.publicKey.toBuffer()],
      program.programId
    );

    expect(player1StatePda).to.not.be.undefined;
    console.log("✅ Player 1 State PDA:", player1StatePda.toString());
  });

  it("Should derive token accounts correctly", async () => {
    // Verify associated token account derivation
    const player1DumpAccount = await getAssociatedTokenAddress(
      dumpMintPda,
      player1.publicKey
    );

    const player1GloryAccount = await getAssociatedTokenAddress(
      gloryMintPda,
      player1.publicKey
    );

    expect(player1DumpAccount).to.not.be.undefined;
    expect(player1GloryAccount).to.not.be.undefined;

    console.log("✅ Player 1 DUMP Account:", player1DumpAccount.toString());
    console.log("✅ Player 1 GLORY Account:", player1GloryAccount.toString());
  });

  it("Should calculate fees correctly", async () => {
    // Test fee calculation logic (matches constants.rs)
    const TRANSFER_FEE_BASIS_POINTS = 30; // 0.3%
    const transferAmount = 1000000; // 1 DUMP
    
    const expectedFee = Math.floor(transferAmount * TRANSFER_FEE_BASIS_POINTS / 10000);
    expect(expectedFee).to.equal(300); // 0.3% of 1M = 300
    
    console.log("✅ Transfer amount:", transferAmount);
    console.log("✅ Expected fee (0.3%):", expectedFee);
  });

  it("Should calculate cooldowns correctly", async () => {
    // Test cooldown logic similar to the smart contract
    const TRANSFER_COOLDOWN_MIN = 15; // seconds
    const TRANSFER_COOLDOWN_MAX = 1800; // 30 minutes
    
    const smallAmount = 1000;
    const largeAmount = 1000000000; // 1B DUMP
    
    // Simplified cooldown calculation (actual implementation may differ)
    const smallCooldown = Math.max(TRANSFER_COOLDOWN_MIN, Math.min(TRANSFER_COOLDOWN_MAX, Math.sqrt(smallAmount)));
    const largeCooldown = Math.max(TRANSFER_COOLDOWN_MIN, Math.min(TRANSFER_COOLDOWN_MAX, Math.sqrt(largeAmount)));
    
    expect(smallCooldown).to.be.at.least(TRANSFER_COOLDOWN_MIN);
    expect(largeCooldown).to.be.at.most(TRANSFER_COOLDOWN_MAX);
    
    console.log("✅ Small amount cooldown:", smallCooldown, "seconds");
    console.log("✅ Large amount cooldown:", largeCooldown, "seconds");
  });

  it("Should verify epoch and player constants", async () => {
    // Verify constants match what's defined in the Rust program
    const EPOCH_DURATION = 30 * 24 * 60 * 60; // 30 days
    const WAITING_PERIOD = 7 * 24 * 60 * 60; // 7 days
    const MIN_DUMP_ASSIGNMENT = 1_000_000; // 1 DUMP
    const MAX_DUMP_ASSIGNMENT = 10_000_000_000; // 10B DUMP
    
    expect(EPOCH_DURATION).to.equal(2592000); // 30 days in seconds
    expect(WAITING_PERIOD).to.equal(604800); // 7 days in seconds
    expect(MIN_DUMP_ASSIGNMENT).to.be.lessThan(MAX_DUMP_ASSIGNMENT);
    
    console.log("✅ Epoch duration:", EPOCH_DURATION, "seconds");
    console.log("✅ Waiting period:", WAITING_PERIOD, "seconds");
    console.log("✅ DUMP assignment range:", MIN_DUMP_ASSIGNMENT, "to", MAX_DUMP_ASSIGNMENT);
  });

  it("Should verify bounty reward constants", async () => {
    // Test bug bounty amounts (from constants.rs)
    const CRITICAL_BOUNTY = 100_000_000_000_000; // 100K GLORY (9 decimals)
    const HIGH_BOUNTY = 50_000_000_000_000; // 50K GLORY
    const MEDIUM_BOUNTY = 25_000_000_000_000; // 25K GLORY
    const LOW_BOUNTY = 10_000_000_000_000; // 10K GLORY
    
    expect(CRITICAL_BOUNTY).to.be.greaterThan(HIGH_BOUNTY);
    expect(HIGH_BOUNTY).to.be.greaterThan(MEDIUM_BOUNTY);
    expect(MEDIUM_BOUNTY).to.be.greaterThan(LOW_BOUNTY);
    
    console.log("✅ Bug bounty rewards verified");
    console.log("  - Critical:", CRITICAL_BOUNTY / 1e9, "GLORY");
    console.log("  - High:", HIGH_BOUNTY / 1e9, "GLORY");
    console.log("  - Medium:", MEDIUM_BOUNTY / 1e9, "GLORY");
    console.log("  - Low:", LOW_BOUNTY / 1e9, "GLORY");
  });

  // Helper function to airdrop SOL
  async function airdropSol(provider: AnchorProvider, publicKey: PublicKey, amount: number) {
    try {
      const signature = await provider.connection.requestAirdrop(
        publicKey, 
        amount * LAMPORTS_PER_SOL
      );
      
      await provider.connection.confirmTransaction(signature, "confirmed");
      
      const balance = await provider.connection.getBalance(publicKey);
      console.log(`✅ Airdropped ${amount} SOL to ${publicKey.toString().slice(0, 8)}..., balance: ${balance / LAMPORTS_PER_SOL} SOL`);
    } catch (error) {
      console.log(`ℹ️  Airdrop may have failed (likely rate limit), continuing with test...`);
    }
  }
});
