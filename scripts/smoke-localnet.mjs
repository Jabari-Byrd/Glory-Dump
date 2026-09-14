import assert from "node:assert/strict";
import { createHash, randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";

import { AnchorProvider, Program, Wallet } from "@anchor-lang/core";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";

const walletPath = process.env.SOLANA_WALLET_PATH;
if (!walletPath) {
  throw new Error("Set SOLANA_WALLET_PATH to an isolated, funded local-validator keypair");
}

const rpcUrl = process.env.SOLANA_RPC_URL ?? "http://127.0.0.1:8899";
const idl = JSON.parse(
  readFileSync(new URL("../frontend/src/idl/glory_dump.json", import.meta.url), "utf8"),
);
const programId = new PublicKey(process.env.GLORY_DUMP_PROGRAM_ID ?? idl.address);
const secretKey = JSON.parse(readFileSync(walletPath, "utf8"));
if (!Array.isArray(secretKey) || secretKey.length !== 64) {
  throw new Error("SOLANA_WALLET_PATH must contain a 64-byte Solana keypair JSON array");
}

const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));
const connection = new Connection(rpcUrl, "confirmed");
const provider = new AnchorProvider(connection, new Wallet(payer), {
  commitment: "confirmed",
  preflightCommitment: "confirmed",
});
const program = new Program({ ...idl, address: programId.toBase58() }, provider);

const [protocolAddress] = PublicKey.findProgramAddressSync(
  [Buffer.from("protocol")],
  programId,
);
const protocol = await program.account.protocol.fetch(protocolAddress);
const epochNumber = BigInt(protocol.currentEpoch.toString());
const epochBytes = u64LittleEndian(epochNumber);
const [epochAddress] = PublicKey.findProgramAddressSync(
  [Buffer.from("epoch"), epochBytes],
  programId,
);
const [playerAddress] = PublicKey.findProgramAddressSync(
  [Buffer.from("player"), epochBytes, payer.publicKey.toBuffer()],
  programId,
);
const lanes = Array.from({ length: 4 }, (_, index) =>
  PublicKey.findProgramAddressSync(
    [Buffer.from("lane"), epochBytes, payer.publicKey.toBuffer(), Buffer.from([index])],
    programId,
  )[0]);

if (await connection.getAccountInfo(playerAddress, "confirmed")) {
  throw new Error("The smoke wallet is already registered; use a fresh validator ledger or wallet");
}

const beforeEpoch = await program.account.epoch.fetch(epochAddress);
const beforeLamports = await connection.getBalance(epochAddress, "confirmed");
const revealSecret = randomBytes(32);
const commitment = createHash("sha256")
  .update("glory-dump-commitment-v3")
  .update(revealSecret)
  .update(payer.publicKey.toBuffer())
  .update(epochBytes)
  .digest();

const signature = await program.methods
  .register([...commitment])
  .accountsPartial({
    payer: payer.publicKey,
    protocol: protocolAddress,
    epoch: epochAddress,
    player: playerAddress,
    laneZero: lanes[0],
    laneOne: lanes[1],
    laneTwo: lanes[2],
    laneThree: lanes[3],
    systemProgram: SystemProgram.programId,
  })
  .rpc();

const [afterEpoch, afterLamports, player, ...laneAccounts] = await Promise.all([
  program.account.epoch.fetch(epochAddress),
  connection.getBalance(epochAddress, "confirmed"),
  program.account.playerEpoch.fetch(playerAddress),
  ...lanes.map((lane) => program.account.balanceLane.fetch(lane)),
]);

assert.equal(
  afterEpoch.participantCount,
  beforeEpoch.participantCount + 1,
  "registration must increment the participant count exactly once",
);
assert.equal(
  afterLamports - beforeLamports,
  2_000_000,
  "registration must add exactly the 0.002 SOL activity bond to the epoch",
);
assert(player.owner.equals(payer.publicKey), "player owner must be the transaction signer");
assert.equal(player.epoch.toString(), epochNumber.toString());
assert.deepEqual(Buffer.from(player.commitment), commitment);
assert.equal(player.revealed, false);
assert.equal(player.allocationClaimed, false);

for (const [index, lane] of laneAccounts.entries()) {
  assert(lane.owner.equals(payer.publicKey), `lane ${index} owner mismatch`);
  assert.equal(lane.epoch.toString(), epochNumber.toString());
  assert.equal(lane.index, index);
  assert.equal(lane.balance.toString(), "0");
  assert.equal(lane.guard.toString(), "0");
}

console.log(JSON.stringify({
  signature,
  programId: programId.toBase58(),
  epoch: epochNumber.toString(),
  player: playerAddress.toBase58(),
  participantCount: afterEpoch.participantCount,
  bondLamports: afterLamports - beforeLamports,
  lanes: lanes.map((lane) => lane.toBase58()),
}, null, 2));

function u64LittleEndian(value) {
  if (value < 0n || value > 0xffff_ffff_ffff_ffffn) {
    throw new Error("Epoch number must fit in u64");
  }
  const bytes = Buffer.alloc(8);
  bytes.writeBigUInt64LE(value);
  return bytes;
}
