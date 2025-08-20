import { PublicKey, SystemProgram } from "@solana/web3.js";

describe.skip("rewards claims (merkle)", () => {
  it("sets root and claims (single leaf)", async () => {
    const gameState = PublicKey.findProgramAddressSync(
      [Buffer.from("game_state")],
      new PublicKey("GDgame1111111111111111111111111111111111111"),
    )[0];

    const epochNumber = 1;
    const epochState = PublicKey.findProgramAddressSync(
      [
        Buffer.from("epoch_state"),
        Buffer.from(Buffer.alloc(8, 0).fill(Number(epochNumber), 0, 8)),
      ],
      new PublicKey("GDgame1111111111111111111111111111111111111"),
    )[0];

    const amount = BigInt(1_000_000_000); // 1,000 GLORY (9 decimals)
    const claimer = new PublicKey("11111111111111111111111111111111");

    const leaf = Buffer.concat([
      claimer.toBuffer(),
      Buffer.from(new Uint8Array(new BigUint64Array([amount]).buffer)),
    ]);
    const root = Buffer.alloc(32); // placeholder in scaffold

    // set_rewards_root (requires admin signer in real flow)
    // Placeholder: setting root requires Anchor provider; omitted in skipped test

    // claim_rewards (empty proof for single-leaf tree)
    // Placeholder: claiming requires Anchor provider; omitted in skipped test
    //   epochState,
    //   gameState,
    //   claimStatus: PublicKey.findProgramAddressSync([
    //     Buffer.from("claim_status"),
    //     Buffer.from(Buffer.alloc(8, 0).fill(Number(epochNumber), 0, 8)),
    //     claimer.toBuffer(),
    //   ], program.programId)[0],
    //   gloryMint: PublicKey.findProgramAddressSync([Buffer.from("glory_mint")], program.programId)[0],
    //   claimer: claimer,
    //   claimerGloryAccount: PublicKey.default,
    //   systemProgram: SystemProgram.programId,
    // }).rpc();

    // This test is a scaffold; implement full flow when program builds.
  });
});
