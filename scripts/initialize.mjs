import { readFileSync } from "node:fs";

import { AnchorProvider, Program, Wallet } from "@anchor-lang/core";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";

const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);
const walletPath = process.env.SOLANA_WALLET_PATH;
if (!walletPath) {
  throw new Error("Set SOLANA_WALLET_PATH to the absolute path of a dedicated deployment keypair");
}

const idl = JSON.parse(
  readFileSync(new URL("../frontend/src/idl/glory_dump.json", import.meta.url), "utf8"),
);
const rpcUrl = process.env.SOLANA_RPC_URL ?? "http://127.0.0.1:8899";
const programId = new PublicKey(process.env.GLORY_DUMP_PROGRAM_ID ?? idl.address);
const secret = JSON.parse(readFileSync(walletPath, "utf8"));
if (!Array.isArray(secret) || secret.length !== 64) {
  throw new Error("SOLANA_WALLET_PATH must contain a 64-byte Solana keypair JSON array");
}

const payer = Keypair.fromSecretKey(Uint8Array.from(secret));
const connection = new Connection(rpcUrl, "confirmed");
const wallet = new Wallet(payer);
const provider = new AnchorProvider(connection, wallet, {
  commitment: "confirmed",
  preflightCommitment: "confirmed",
});
const program = new Program({ ...idl, address: programId.toBase58() }, provider);

const [protocol] = PublicKey.findProgramAddressSync([Buffer.from("protocol")], programId);
const [gloryMint] = PublicKey.findProgramAddressSync([Buffer.from("glory_mint")], programId);
const epochBytes = Buffer.alloc(8);
epochBytes.writeBigUInt64LE(1n);
const [epoch] = PublicKey.findProgramAddressSync(
  [Buffer.from("epoch"), epochBytes],
  programId,
);
const [leaderboard] = PublicKey.findProgramAddressSync(
  [Buffer.from("leaderboard"), epochBytes],
  programId,
);

if (await connection.getAccountInfo(protocol, "confirmed")) {
  throw new Error(`Protocol is already initialized at ${protocol.toBase58()}`);
}

const signature = await program.methods
  .initializeProtocol()
  .accountsPartial({
    payer: payer.publicKey,
    protocol,
    gloryMint,
    epoch,
    leaderboard,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

console.log(JSON.stringify({
  programId: programId.toBase58(),
  protocol: protocol.toBase58(),
  gloryMint: gloryMint.toBase58(),
  epoch: epoch.toBase58(),
  leaderboard: leaderboard.toBase58(),
  signature,
}, null, 2));
