import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, resolve } from "node:path";
import { spawn } from "node:child_process";

import { AnchorProvider, BN, Program, Wallet } from "@anchor-lang/core";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

const PROGRAM_ID = new PublicKey("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
const TOKEN_PROGRAM_ID = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const UPGRADEABLE_LOADER_ID = new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111");
const TEST_PROTOCOL_VERSION = 0x8003;
const PLAYER_COUNT = 21;
const REVEALED_COUNT = 20;
const BOND_LAMPORTS = 2_000_000;
const REFUND_LAMPORTS = 1_600_000;
const SETTLEMENT_BOUNTY_LAMPORTS = 400_000;
const GLORY_SCALE = 1_000_000n;
const RPC_PORT = Number(process.env.GLORY_DUMP_TEST_RPC_PORT ?? "18899");
const RPC_URL = process.env.SOLANA_RPC_URL ?? `http://127.0.0.1:${RPC_PORT}`;
const MANAGE_VALIDATOR = process.env.GLORY_DUMP_MANAGE_VALIDATOR !== "0";
const idl = JSON.parse(
  readFileSync(new URL("../frontend/src/idl/glory_dump.json", import.meta.url), "utf8"),
);

let validator;
let ledgerDirectory;
const validatorLog = [];
const lifecyclePayer = Keypair.generate();

try {
  if (MANAGE_VALIDATOR) {
    ({ child: validator, ledger: ledgerDirectory } = await startValidator(lifecyclePayer.publicKey));
  }
  const report = await runLifecycle(lifecyclePayer, MANAGE_VALIDATOR);
  console.log(JSON.stringify(report, null, 2));
} catch (error) {
  process.exitCode = 1;
  console.error(error instanceof Error ? (error.stack ?? error.message) : error);
} finally {
  if (validator && validator.exitCode === null) {
    validator.kill("SIGTERM");
    await Promise.race([onceExit(validator), sleep(5_000)]);
    if (validator.exitCode === null) {
      validator.kill("SIGKILL");
      await Promise.race([onceExit(validator), sleep(2_000)]);
    }
  }
  if (ledgerDirectory) rmSync(ledgerDirectory, { recursive: true, force: true });
}
// @solana/web3.js retains websocket retry timers after the managed validator
// disappears. The report is complete and the ledger is removed at this point.
process.exit(process.exitCode ?? 0);

async function startValidator(mintAddress) {
  const programSo = process.env.GLORY_DUMP_TEST_PROGRAM_SO;
  if (!programSo) {
    throw new Error(
      "Set GLORY_DUMP_TEST_PROGRAM_SO to the accelerated test-fast SBF binary. " +
      "Never use a production deployment binary for this time-compressed suite.",
    );
  }
  const validatorBinary = process.env.SOLANA_TEST_VALIDATOR ?? "solana-test-validator";
  const absoluteProgram = resolve(programSo);
  const ledger = mkdtempSync(`${tmpdir()}/glory-dump-validator-`);
  const child = spawn(
    validatorBinary,
    [
      "--reset",
      "--quiet",
      "--ledger",
      ledger,
      "--rpc-port",
      String(RPC_PORT),
      "--faucet-port",
      String(RPC_PORT + 2),
      "--dynamic-port-range",
      `${RPC_PORT + 11}-${RPC_PORT + 111}`,
      "--ticks-per-slot",
      "8",
      "--mint",
      mintAddress.toBase58(),
      "--upgradeable-program",
      PROGRAM_ID.toBase58(),
      absoluteProgram,
      "none",
      "--limit-ledger-size",
      "50000",
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
  const capture = (chunk) => {
    validatorLog.push(chunk.toString());
    if (validatorLog.length > 100) validatorLog.shift();
  };
  child.stdout.on("data", capture);
  child.stderr.on("data", capture);
  child.once("exit", (code) => {
    if (code && code !== 0) {
      process.stderr.write(`validator exited with ${code}\n${validatorLog.join("")}\n`);
    }
  });
  await waitForRpc(new Connection(RPC_URL, "confirmed"), child, PROGRAM_ID);
  return { child, ledger };
}

async function runLifecycle(payer, payerFundedAtGenesis) {
  const connection = new Connection(RPC_URL, "confirmed");
  const signatures = [];
  if (!payerFundedAtGenesis) await airdrop(connection, payer.publicKey, 10_000_000_000);
  const payerProgram = programFor(connection, payer);
  const protocol = protocolPda();
  const gloryMint = gloryMintPda();
  const epochOne = epochPda(1n);
  const leaderboardOne = leaderboardPda(1n);

  const initializeSignature = await payerProgram.methods
    .initializeProtocol()
    .accountsPartial({
      payer: payer.publicKey,
      protocol,
      gloryMint,
      epoch: epochOne,
      leaderboard: leaderboardOne,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  signatures.push(await observeTransaction(connection, initializeSignature, "initialize_protocol"));
  const protocolAccount = await payerProgram.account.protocol.fetch(protocol);
  assert.equal(
    protocolAccount.version,
    TEST_PROTOCOL_VERSION,
    "lifecycle tests must run against the visibly test-only accelerated binary",
  );
  await expectFailure(
    payerProgram.methods
      .initializeProtocol()
      .accountsPartial({
        payer: payer.publicKey,
        protocol,
        gloryMint,
        epoch: epochOne,
        leaderboard: leaderboardOne,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc(),
    "a second protocol initialization must fail",
  );

  const players = Array.from({ length: PLAYER_COUNT }, () => Keypair.generate());
  await fundWallets(connection, payer, players, 300_000_000);
  const secrets = players.map((player, index) => deterministicSecret(player.publicKey, 1n, index));
  const commitments = players.map((player, index) => commitmentFor(secrets[index], player.publicKey, 1n));
  const initialEpochLamports = await connection.getBalance(epochOne, "confirmed");

  await expectFailure(register(connection, players[0], 1n, Buffer.alloc(32)), "zero commitment");
  const registrationSignatures = await parallelMap(players, 7, (player, index) =>
    register(connection, player, 1n, commitments[index]));
  signatures.push(...await observeMany(connection, registrationSignatures, "register"));
  await expectFailure(register(connection, players[0], 1n, commitments[0]), "duplicate registration");
  const registeredEpoch = await payerProgram.account.epoch.fetch(epochOne);
  assert.equal(registeredEpoch.participantCount, PLAYER_COUNT);
  assert.equal(
    await connection.getBalance(epochOne, "confirmed") - initialEpochLamports,
    PLAYER_COUNT * BOND_LAMPORTS,
    "registration must transfer exactly one bond per player",
  );

  await waitForChainTimestamp(connection, toNumber(registeredEpoch.registrationEndsAt));
  const revealOpenSignature = await payerProgram.methods
    .beginReveal()
    .accountsPartial({ protocol, epoch: epochOne })
    .rpc();
  signatures.push(await observeTransaction(connection, revealOpenSignature, "begin_reveal"));
  await expectFailure(reveal(connection, players[0], 1n, Buffer.alloc(32, 0xff)), "wrong reveal secret");
  const revealSignatures = await parallelMap(players.slice(0, REVEALED_COUNT), 7, (player, index) =>
    reveal(connection, player, 1n, secrets[index]));
  signatures.push(...await observeMany(connection, revealSignatures, "reveal"));
  await expectFailure(reveal(connection, players[0], 1n, secrets[0]), "duplicate reveal");
  const revealingEpoch = await payerProgram.account.epoch.fetch(epochOne);
  assert.equal(revealingEpoch.revealedCount, REVEALED_COUNT);

  await waitForChainTimestamp(connection, toNumber(revealingEpoch.revealEndsAt));
  const sealSignature = await payerProgram.methods
    .sealRandomness()
    .accountsPartial({ protocol, epoch: epochOne })
    .rpc();
  signatures.push(await observeTransaction(connection, sealSignature, "seal_randomness"));
  const planningEpoch = await payerProgram.account.epoch.fetch(epochOne);
  assert(!Buffer.from(planningEpoch.seed).equals(Buffer.alloc(32)), "sealed epoch seed must be nonzero");
  await expectFailure(
    programFor(connection, players[0]).methods
      .armRedirect(0)
      .accountsPartial({
        signer: players[0].publicKey,
        epoch: epochOne,
        player: playerPda(1n, players[0].publicKey),
        lane: lanePda(1n, players[0].publicKey, 0),
      })
      .rpc(),
    "REDIRECT without Guard",
  );

  const allocationSignatures = await parallelMap(players, 7, (player) => claimAllocation(connection, player, 1n));
  signatures.push(...await observeMany(connection, allocationSignatures, "claim_allocation"));
  const bundles = await Promise.all(players.map((player) => fetchBundle(payerProgram, 1n, player.publicKey)));
  for (const [index, bundle] of bundles.entries()) {
    const allocation = toBigInt(bundle.player.startingAllocation);
    assert.equal(bundle.lanes.reduce((sum, lane) => sum + toBigInt(lane.balance), 0n), allocation);
    if (index < REVEALED_COUNT) {
      assert(allocation >= 1_000_000_000n && allocation <= 10_000_000_000n);
      assert.equal(allocation % 1_000_000_000n, 0n);
    } else {
      assert.equal(allocation, 10_000_000_000n, "non-revealer must receive maximum burden");
      assert.equal(bundle.player.revealed, false);
    }
  }
  const initialDump = bundles.reduce(
    (total, bundle) => total + bundle.lanes.reduce((sum, lane) => sum + toBigInt(lane.balance), 0n),
    0n,
  );

  await waitForChainTimestamp(connection, toNumber(planningEpoch.activeStartsAt));
  const activeSignature = await payerProgram.methods
    .beginActive()
    .accountsPartial({ protocol, epoch: epochOne })
    .rpc();
  signatures.push(await observeTransaction(connection, activeSignature, "begin_active"));
  await expectFailure(
    completeEpoch(payerProgram, payer.publicKey, protocol, epochOne, leaderboardOne),
    "completion before settlement",
  );

  const activeEpoch = await payerProgram.account.epoch.fetch(epochOne);
  const eligibilityActions = await parallelMap(players.slice(0, REVEALED_COUNT), 5, async (actor, index) => {
    const target = players[(index + 1) % REVEALED_COUNT];
    const actorAccount = await payerProgram.account.playerEpoch.fetch(playerPda(1n, actor.publicKey));
    const amount = divCeil(toBigInt(actorAccount.startingAllocation), 1_000n);
    return dump(connection, actor, actor.publicKey, target.publicKey, 1n, amount, 0);
  });
  signatures.push(...await observeMany(connection, eligibilityActions, "eligibility_dump"));

  const staleActor = players[4];
  const staleTarget = players[7];
  const staleAccount = await payerProgram.account.playerEpoch.fetch(playerPda(1n, staleActor.publicKey));
  const staleAmount = divCeil(toBigInt(staleAccount.startingAllocation), 1_000n);
  const correctLane = await expectedTargetLane(payerProgram, 1n, staleActor.publicKey, staleTarget.publicKey);
  const staleRivalry = rivalryPda(1n, staleActor.publicKey, staleTarget.publicKey);
  await expectFailure(
    dump(connection, staleActor, staleActor.publicKey, staleTarget.publicKey, 1n, staleAmount, 0, (correctLane + 1) % 4),
    "caller-chosen target lane",
  );
  assert.equal(await connection.getAccountInfo(staleRivalry, "confirmed"), null, "failed action must roll back rivalry creation");

  const unauthorizedActor = players[5];
  const stranger = players[PLAYER_COUNT - 1];
  const unauthorizedTarget = players[8];
  const unauthorizedAccount = await payerProgram.account.playerEpoch.fetch(playerPda(1n, unauthorizedActor.publicKey));
  await expectFailure(
    dump(
      connection,
      stranger,
      unauthorizedActor.publicKey,
      unauthorizedTarget.publicKey,
      1n,
      divCeil(toBigInt(unauthorizedAccount.startingAllocation), 1_000n),
      0,
    ),
    "stranger gameplay authority",
  );

  const sessionOwner = players[3];
  const sessionDelegate = players[PLAYER_COUNT - 1];
  const sessionProgram = programFor(connection, sessionOwner);
  const sessionSignature = await sessionProgram.methods
    .authorizeSession(sessionDelegate.publicKey, new BN(20), 1)
    .accountsPartial({
      owner: sessionOwner.publicKey,
      epoch: epochOne,
      player: playerPda(1n, sessionOwner.publicKey),
    })
    .rpc();
  signatures.push(await observeTransaction(connection, sessionSignature, "authorize_session"));
  const sessionTarget = players[9];
  const sessionOwnerAccount = await payerProgram.account.playerEpoch.fetch(playerPda(1n, sessionOwner.publicKey));
  const delegatedAmount = divCeil(toBigInt(sessionOwnerAccount.startingAllocation), 1_000n);
  const delegatedSignature = await dump(
    connection,
    sessionDelegate,
    sessionOwner.publicKey,
    sessionTarget.publicKey,
    1n,
    delegatedAmount,
    1,
  );
  signatures.push(await observeTransaction(connection, delegatedSignature, "delegated_dump"));
  await expectFailure(
    dump(
      connection,
      sessionDelegate,
      sessionOwner.publicKey,
      sessionTarget.publicKey,
      1n,
      delegatedAmount,
      1,
    ),
    "exhausted session delegate",
  );

  const guardOwner = players[0];
  const guardSource = players[1];
  const guardAttacker = players[2];
  const guardOwnerAccount = await payerProgram.account.playerEpoch.fetch(playerPda(1n, guardOwner.publicKey));
  const guardAttackerAccount = await payerProgram.account.playerEpoch.fetch(playerPda(1n, guardAttacker.publicKey));
  const incoming = divCeil(toBigInt(guardAttackerAccount.startingAllocation), 1_000n);
  const absorbAmount = max(
    incoming * 4n,
    divCeil(toBigInt(guardOwnerAccount.startingAllocation), 1_000n),
  );
  const guardedLaneIndex = await expectedTargetLane(payerProgram, 1n, guardAttacker.publicKey, guardOwner.publicKey);
  const guardSourceBundle = await fetchBundle(payerProgram, 1n, guardSource.publicKey);
  const guardSourceLane = indexOfLargestSpendable(guardSourceBundle.lanes);
  const absorbSignature = await absorb(
    connection,
    guardOwner,
    guardOwner.publicKey,
    guardSource.publicKey,
    1n,
    absorbAmount,
    guardedLaneIndex,
    guardSourceLane,
  );
  signatures.push(await observeTransaction(connection, absorbSignature, "absorb_guard"));
  const guardedLaneAddress = lanePda(1n, guardOwner.publicKey, guardedLaneIndex);
  const guardedAfterAbsorb = await payerProgram.account.balanceLane.fetch(guardedLaneAddress);
  const expectedGuard = min(
    absorbAmount / 2n,
    toBigInt(guardOwnerAccount.startingAllocation) / 16n,
  );
  assert.equal(toBigInt(guardedAfterAbsorb.guard), expectedGuard);
  assert(toBigInt(guardedAfterAbsorb.lockedAmount) >= absorbAmount);
  const armSignature = await programFor(connection, guardOwner).methods
    .armRedirect(guardedLaneIndex)
    .accountsPartial({
      signer: guardOwner.publicKey,
      epoch: epochOne,
      player: playerPda(1n, guardOwner.publicKey),
      lane: guardedLaneAddress,
    })
    .rpc();
  signatures.push(await observeTransaction(connection, armSignature, "arm_redirect"));
  await expectFailure(
    programFor(connection, guardOwner).methods
      .armRedirect(guardedLaneIndex)
      .accountsPartial({
        signer: guardOwner.publicKey,
        epoch: epochOne,
        player: playerPda(1n, guardOwner.publicKey),
        lane: guardedLaneAddress,
      })
      .rpc(),
    "double arm",
  );
  const attackerBundle = await fetchBundle(payerProgram, 1n, guardAttacker.publicKey);
  const attackerSourceLane = indexOfLargestSpendable(attackerBundle.lanes);
  const beforeRicochet = await payerProgram.account.balanceLane.fetch(guardedLaneAddress);
  const ricochetSignature = await dump(
    connection,
    guardAttacker,
    guardAttacker.publicKey,
    guardOwner.publicKey,
    1n,
    incoming,
    attackerSourceLane,
  );
  signatures.push(await observeTransaction(connection, ricochetSignature, "redirect_hit"));
  const afterRicochet = await payerProgram.account.balanceLane.fetch(guardedLaneAddress);
  assert.equal(
    toBigInt(afterRicochet.balance),
    toBigInt(beforeRicochet.balance),
    "Guard larger than the hit must prevent landed DUMP",
  );
  assert.equal(toBigInt(afterRicochet.guard), expectedGuard - incoming);
  assert.equal(afterRicochet.redirectArmed, false);
  await expectFailure(
    programFor(connection, guardOwner).methods
      .armRedirect(guardedLaneIndex)
      .accountsPartial({
        signer: guardOwner.publicKey,
        epoch: epochOne,
        player: playerPda(1n, guardOwner.publicKey),
        lane: guardedLaneAddress,
      })
      .rpc(),
    "REDIRECT rearm delay",
  );

  const lockedLane = await payerProgram.account.balanceLane.fetch(guardedLaneAddress);
  const lockedSpendable = toBigInt(lockedLane.balance) - toBigInt(lockedLane.lockedAmount);
  const lockedAttempt = lockedSpendable + 1n;
  const lockedTarget = players[10];
  await expectFailure(
    dump(
      connection,
      guardOwner,
      guardOwner.publicKey,
      lockedTarget.publicKey,
      1n,
      lockedAttempt,
      guardedLaneIndex,
    ),
    "spending locked absorbed DUMP",
  );

  const activeBundles = await Promise.all(players.map((player) => fetchBundle(payerProgram, 1n, player.publicKey)));
  const activeDump = activeBundles.reduce(
    (total, bundle) => total + bundle.lanes.reduce((sum, lane) => sum + toBigInt(lane.balance), 0n),
    0n,
  );
  assert.equal(activeDump, initialDump, "all gameplay paths must conserve total DUMP");

  await waitForChainTimestamp(connection, toNumber(activeEpoch.activeEndsAt));
  const settlementSignature = await payerProgram.methods
    .beginSettlement()
    .accountsPartial({ protocol, epoch: epochOne })
    .rpc();
  signatures.push(await observeTransaction(connection, settlementSignature, "begin_settlement"));
  await expectFailure(
    completeEpoch(payerProgram, payer.publicKey, protocol, epochOne, leaderboardOne),
    "completion before every player is settled",
  );

  const keeperCredit = keeperPda(1n, payer.publicKey);
  const keeperBalanceBefore = await connection.getBalance(payer.publicKey, "confirmed");
  for (const player of players) {
    const signature = await payerProgram.methods
      .settlePlayer()
      .accountsPartial({
        keeper: payer.publicKey,
        epoch: epochOne,
        leaderboard: leaderboardOne,
        player: playerPda(1n, player.publicKey),
        laneZero: lanePda(1n, player.publicKey, 0),
        laneOne: lanePda(1n, player.publicKey, 1),
        laneTwo: lanePda(1n, player.publicKey, 2),
        laneThree: lanePda(1n, player.publicKey, 3),
        keeperCredit,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    signatures.push(await observeTransaction(connection, signature, "settle_player"));
  }
  await expectFailure(
    payerProgram.methods
      .settlePlayer()
      .accountsPartial({
        keeper: payer.publicKey,
        epoch: epochOne,
        leaderboard: leaderboardOne,
        player: playerPda(1n, players[0].publicKey),
        laneZero: lanePda(1n, players[0].publicKey, 0),
        laneOne: lanePda(1n, players[0].publicKey, 1),
        laneTwo: lanePda(1n, players[0].publicKey, 2),
        laneThree: lanePda(1n, players[0].publicKey, 3),
        keeperCredit,
        systemProgram: SystemProgram.programId,
      })
      .rpc(),
    "double settlement",
  );
  const settledEpoch = await payerProgram.account.epoch.fetch(epochOne);
  assert.equal(settledEpoch.settledCount, PLAYER_COUNT);
  assert.equal(settledEpoch.eligibleCount, REVEALED_COUNT);
  assert.equal(
    toBigInt(settledEpoch.settlementBountiesPaid),
    BigInt(PLAYER_COUNT * SETTLEMENT_BOUNTY_LAMPORTS),
  );
  const keeperAfterSettlement = await connection.getBalance(payer.publicKey, "confirmed");
  assert(
    keeperAfterSettlement > keeperBalanceBefore - 20_000_000,
    "settlement bounties should offset keeper transaction/rent costs",
  );
  const completeSignature = await completeEpoch(
    payerProgram,
    payer.publicKey,
    protocol,
    epochOne,
    leaderboardOne,
  );
  signatures.push(await observeTransaction(connection, completeSignature, "complete_epoch"));
  const completedEpoch = await payerProgram.account.epoch.fetch(epochOne);
  const leaderboard = await payerProgram.account.leaderboard.fetch(leaderboardOne);
  assert.equal(completedEpoch.winnerCount, 1);
  assert.equal(leaderboard.entries.length, completedEpoch.winnerCount);
  assert.equal(
    toBigInt(completedEpoch.playerRewardPool) + toBigInt(completedEpoch.keeperRewardPool),
    toBigInt(completedEpoch.totalRewardPool),
  );
  assert(
    leaderboard.entries.every((entry, index, entries) =>
      index === 0 || toBigInt(entries[index - 1].score) <= toBigInt(entry.score)),
    "leaderboard must be sorted from lowest score upward",
  );

  const winner = players.find((player) => player.publicKey.equals(leaderboard.entries[0].player));
  assert(winner, "winner key must belong to a registered wallet");
  const nonWinner = players.slice(0, REVEALED_COUNT).find((player) => !player.publicKey.equals(winner.publicKey));
  assert(nonWinner, "test needs an eligible non-winner");
  await expectFailure(claimPlayerReward(connection, nonWinner, 1n, gloryMint), "non-winner GLORY claim");
  const winnerClaim = await claimPlayerReward(connection, winner, 1n, gloryMint);
  signatures.push(await observeTransaction(connection, winnerClaim, "claim_player_reward"));
  await expectFailure(claimPlayerReward(connection, winner, 1n, gloryMint), "double winner claim");
  const winnerAta = associatedTokenAddress(winner.publicKey, gloryMint);
  const winnerToken = await connection.getParsedAccountInfo(winnerAta, "confirmed");
  const winnerAmount = BigInt(winnerToken.value?.data?.parsed?.info?.tokenAmount?.amount ?? "0");
  assert(winnerAmount > 0n);

  const keeperClaim = await claimKeeperReward(connection, payer, 1n, gloryMint);
  signatures.push(await observeTransaction(connection, keeperClaim, "claim_keeper_reward"));
  await expectFailure(claimKeeperReward(connection, payer, 1n, gloryMint), "double keeper claim");
  const mintInfo = await connection.getParsedAccountInfo(gloryMint, "confirmed");
  const mintedSupply = BigInt(mintInfo.value?.data?.parsed?.info?.supply ?? "0");
  assert.equal(mintedSupply, toBigInt(completedEpoch.totalRewardPool));
  assert(mintedSupply <= 10_000_000n * GLORY_SCALE);

  const epochLamportsBeforeRefunds = await connection.getBalance(epochOne, "confirmed");
  for (const player of players.slice(0, REVEALED_COUNT)) {
    const signature = await programFor(connection, player).methods
      .claimBond()
      .accountsPartial({
        owner: player.publicKey,
        epoch: epochOne,
        player: playerPda(1n, player.publicKey),
      })
      .rpc();
    signatures.push(await observeTransaction(connection, signature, "claim_bond"));
  }
  assert.equal(
    epochLamportsBeforeRefunds - await connection.getBalance(epochOne, "confirmed"),
    REVEALED_COUNT * REFUND_LAMPORTS,
  );
  await expectFailure(
    programFor(connection, players[0]).methods
      .claimBond()
      .accountsPartial({
        owner: players[0].publicKey,
        epoch: epochOne,
        player: playerPda(1n, players[0].publicKey),
      })
      .rpc(),
    "double bond claim",
  );
  await expectFailure(
    programFor(connection, players[PLAYER_COUNT - 1]).methods
      .claimBond()
      .accountsPartial({
        owner: players[PLAYER_COUNT - 1].publicKey,
        epoch: epochOne,
        player: playerPda(1n, players[PLAYER_COUNT - 1].publicKey),
      })
      .rpc(),
    "non-revealer bond forfeiture",
  );

  const cleanupPlayer = nonWinner;
  const cleanupRival = players[(players.indexOf(cleanupPlayer) + 1) % REVEALED_COUNT];
  const cleanupRivalry = rivalryPda(1n, cleanupPlayer.publicKey, cleanupRival.publicKey);
  const closeRivalrySignature = await programFor(connection, cleanupPlayer).methods
    .closeRivalry()
    .accountsPartial({
      epoch: epochOne,
      rentRecipient: cleanupPlayer.publicKey,
      rivalry: cleanupRivalry,
    })
    .rpc();
  signatures.push(await observeTransaction(connection, closeRivalrySignature, "close_rivalry"));
  assert.equal(await connection.getAccountInfo(cleanupRivalry, "confirmed"), null);
  const closePlayerSignature = await closePlayer(connection, cleanupPlayer, 1n, leaderboardOne);
  signatures.push(await observeTransaction(connection, closePlayerSignature, "close_player_accounts"));
  assert.equal(await connection.getAccountInfo(playerPda(1n, cleanupPlayer.publicKey), "confirmed"), null);
  const closeKeeperSignature = await payerProgram.methods
    .closeKeeperCredit()
    .accountsPartial({
      keeper: payer.publicKey,
      epoch: epochOne,
      keeperCredit,
    })
    .rpc();
  signatures.push(await observeTransaction(connection, closeKeeperSignature, "close_keeper_credit"));
  assert.equal(await connection.getAccountInfo(keeperCredit, "confirmed"), null);

  const epochTwo = await openNextEpoch(connection, payer, 1n);
  signatures.push(await observeTransaction(connection, epochTwo.signature, "open_next_epoch"));
  const cancellationWallet = Keypair.generate();
  await fundWallets(connection, payer, [cancellationWallet], 100_000_000);
  const cancellationSecret = deterministicSecret(cancellationWallet.publicKey, 2n, 0);
  const cancellationRegistration = await register(
    connection,
    cancellationWallet,
    2n,
    commitmentFor(cancellationSecret, cancellationWallet.publicKey, 2n),
  );
  signatures.push(await observeTransaction(connection, cancellationRegistration, "cancel_registration"));
  const epochTwoAccount = await payerProgram.account.epoch.fetch(epochTwo.epoch);
  await waitForChainTimestamp(connection, toNumber(epochTwoAccount.registrationEndsAt));
  const cancelSignature = await payerProgram.methods
    .cancelEpoch()
    .accountsPartial({ protocol, epoch: epochTwo.epoch })
    .rpc();
  signatures.push(await observeTransaction(connection, cancelSignature, "cancel_epoch"));
  const cancelBalanceBefore = await connection.getBalance(epochTwo.epoch, "confirmed");
  const cancelRefundSignature = await programFor(connection, cancellationWallet).methods
    .claimBond()
    .accountsPartial({
      owner: cancellationWallet.publicKey,
      epoch: epochTwo.epoch,
      player: playerPda(2n, cancellationWallet.publicKey),
    })
    .rpc();
  signatures.push(await observeTransaction(connection, cancelRefundSignature, "cancelled_bond_refund"));
  assert.equal(
    cancelBalanceBefore - await connection.getBalance(epochTwo.epoch, "confirmed"),
    BOND_LAMPORTS,
    "cancelled epoch must return the complete bond",
  );
  const epochThree = await openNextEpoch(connection, payer, 2n);
  signatures.push(await observeTransaction(connection, epochThree.signature, "open_after_cancel"));
  const finalProtocol = await payerProgram.account.protocol.fetch(protocol);
  assert.equal(finalProtocol.currentEpoch.toString(), "3");

  const byLabel = new Map();
  for (const observation of signatures) {
    const row = byLabel.get(observation.label) ?? { count: 0, fees: 0, computeUnits: 0 };
    row.count += 1;
    row.fees += observation.fee;
    row.computeUnits += observation.computeUnits;
    byLabel.set(observation.label, row);
  }
  return {
    schema: "glory-dump-localnet-lifecycle-v1",
    testProtocolVersion: TEST_PROTOCOL_VERSION,
    programId: PROGRAM_ID.toBase58(),
    rpcUrl: RPC_URL,
    players: PLAYER_COUNT,
    revealed: REVEALED_COUNT,
    eligible: completedEpoch.eligibleCount,
    winners: completedEpoch.winnerCount,
    initialDump: initialDump.toString(),
    finalMintSupply: mintedSupply.toString(),
    totalRewardPool: completedEpoch.totalRewardPool.toString(),
    finalEpoch: finalProtocol.currentEpoch.toString(),
    transactionMetrics: Object.fromEntries(byLabel),
  };
}

function programFor(connection, signer) {
  const provider = new AnchorProvider(connection, new Wallet(signer), {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  return new Program({ ...idl, address: PROGRAM_ID.toBase58() }, provider);
}

async function register(connection, payer, epoch, commitment) {
  const lanes = Array.from({ length: 4 }, (_, index) => lanePda(epoch, payer.publicKey, index));
  return programFor(connection, payer).methods
    .register([...commitment])
    .accountsPartial({
      payer: payer.publicKey,
      protocol: protocolPda(),
      epoch: epochPda(epoch),
      player: playerPda(epoch, payer.publicKey),
      laneZero: lanes[0],
      laneOne: lanes[1],
      laneTwo: lanes[2],
      laneThree: lanes[3],
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

async function reveal(connection, owner, epoch, secret) {
  return programFor(connection, owner).methods
    .reveal([...secret])
    .accountsPartial({
      owner: owner.publicKey,
      epoch: epochPda(epoch),
      player: playerPda(epoch, owner.publicKey),
    })
    .rpc();
}

async function claimAllocation(connection, owner, epoch) {
  return programFor(connection, owner).methods
    .claimAllocation()
    .accountsPartial({
      owner: owner.publicKey,
      epoch: epochPda(epoch),
      player: playerPda(epoch, owner.publicKey),
      laneZero: lanePda(epoch, owner.publicKey, 0),
      laneOne: lanePda(epoch, owner.publicKey, 1),
      laneTwo: lanePda(epoch, owner.publicKey, 2),
      laneThree: lanePda(epoch, owner.publicKey, 3),
    })
    .rpc();
}

async function dump(
  connection,
  signer,
  actor,
  target,
  epoch,
  amount,
  sourceIndex,
  targetIndexOverride,
) {
  const program = programFor(connection, signer);
  const targetIndex = targetIndexOverride ?? await expectedTargetLane(program, epoch, actor, target);
  return program.methods
    .dump(new BN(amount.toString()), sourceIndex, targetIndex)
    .accountsPartial({
      signer: signer.publicKey,
      epoch: epochPda(epoch),
      actorPlayer: playerPda(epoch, actor),
      targetPlayer: playerPda(epoch, target),
      actorLane: lanePda(epoch, actor, sourceIndex),
      targetLane: lanePda(epoch, target, targetIndex),
      rivalry: rivalryPda(epoch, actor, target),
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

async function absorb(connection, signer, actor, target, epoch, amount, destinationIndex, sourceIndex) {
  return programFor(connection, signer).methods
    .absorb(new BN(amount.toString()), destinationIndex, sourceIndex)
    .accountsPartial({
      signer: signer.publicKey,
      epoch: epochPda(epoch),
      actorPlayer: playerPda(epoch, actor),
      targetPlayer: playerPda(epoch, target),
      actorLane: lanePda(epoch, actor, destinationIndex),
      targetLane: lanePda(epoch, target, sourceIndex),
      rivalry: rivalryPda(epoch, actor, target),
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

async function expectedTargetLane(program, epoch, actor, target) {
  const [epochAccount, rivalry] = await Promise.all([
    program.account.epoch.fetch(epochPda(epoch)),
    program.account.rivalry.fetchNullable(rivalryPda(epoch, actor, target)),
  ]);
  const actionCount = rivalry?.actionCount ?? 0;
  const digest = createHash("sha256")
    .update("glory-dump-target-lane-v3")
    .update(Buffer.from(epochAccount.seed))
    .update(actor.toBuffer())
    .update(target.toBuffer())
    .update(u64LittleEndian(epoch))
    .update(u32LittleEndian(actionCount))
    .digest();
  return digest[0] % 4;
}

async function completeEpoch(program, finalizer, protocol, epoch, leaderboard) {
  return program.methods
    .completeEpoch()
    .accountsPartial({ finalizer, protocol, epoch, leaderboard })
    .rpc();
}

async function claimPlayerReward(connection, owner, epoch, gloryMint) {
  return programFor(connection, owner).methods
    .claimPlayerReward()
    .accountsPartial({
      owner: owner.publicKey,
      protocol: protocolPda(),
      epoch: epochPda(epoch),
      leaderboard: leaderboardPda(epoch),
      player: playerPda(epoch, owner.publicKey),
      gloryMint,
      destination: associatedTokenAddress(owner.publicKey, gloryMint),
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

async function claimKeeperReward(connection, keeper, epoch, gloryMint) {
  return programFor(connection, keeper).methods
    .claimKeeperReward()
    .accountsPartial({
      keeper: keeper.publicKey,
      protocol: protocolPda(),
      epoch: epochPda(epoch),
      keeperCredit: keeperPda(epoch, keeper.publicKey),
      gloryMint,
      destination: associatedTokenAddress(keeper.publicKey, gloryMint),
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

async function closePlayer(connection, owner, epoch, leaderboard) {
  return programFor(connection, owner).methods
    .closePlayerAccounts()
    .accountsPartial({
      owner: owner.publicKey,
      epoch: epochPda(epoch),
      leaderboard,
      player: playerPda(epoch, owner.publicKey),
      laneZero: lanePda(epoch, owner.publicKey, 0),
      laneOne: lanePda(epoch, owner.publicKey, 1),
      laneTwo: lanePda(epoch, owner.publicKey, 2),
      laneThree: lanePda(epoch, owner.publicKey, 3),
    })
    .rpc();
}

async function openNextEpoch(connection, payer, previousEpochNumber) {
  const next = previousEpochNumber + 1n;
  const epoch = epochPda(next);
  const signature = await programFor(connection, payer).methods
    .openNextEpoch(new BN(next.toString()))
    .accountsPartial({
      payer: payer.publicKey,
      protocol: protocolPda(),
      previousEpoch: epochPda(previousEpochNumber),
      epoch,
      leaderboard: leaderboardPda(next),
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  return { epoch, signature };
}

async function fetchBundle(program, epoch, owner) {
  const [player, ...lanes] = await Promise.all([
    program.account.playerEpoch.fetch(playerPda(epoch, owner)),
    ...Array.from({ length: 4 }, (_, index) =>
      program.account.balanceLane.fetch(lanePda(epoch, owner, index))),
  ]);
  return { player, lanes };
}

function protocolPda() {
  return PublicKey.findProgramAddressSync([Buffer.from("protocol")], PROGRAM_ID)[0];
}

function gloryMintPda() {
  return PublicKey.findProgramAddressSync([Buffer.from("glory_mint")], PROGRAM_ID)[0];
}

function epochPda(epoch) {
  return PublicKey.findProgramAddressSync([Buffer.from("epoch"), u64LittleEndian(epoch)], PROGRAM_ID)[0];
}

function leaderboardPda(epoch) {
  return PublicKey.findProgramAddressSync([Buffer.from("leaderboard"), u64LittleEndian(epoch)], PROGRAM_ID)[0];
}

function playerPda(epoch, owner) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("player"), u64LittleEndian(epoch), owner.toBuffer()],
    PROGRAM_ID,
  )[0];
}

function lanePda(epoch, owner, index) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("lane"), u64LittleEndian(epoch), owner.toBuffer(), Buffer.from([index])],
    PROGRAM_ID,
  )[0];
}

function rivalryPda(epoch, actor, target) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("rivalry"), u64LittleEndian(epoch), actor.toBuffer(), target.toBuffer()],
    PROGRAM_ID,
  )[0];
}

function keeperPda(epoch, keeper) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("keeper"), u64LittleEndian(epoch), keeper.toBuffer()],
    PROGRAM_ID,
  )[0];
}

function associatedTokenAddress(owner, mint) {
  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  )[0];
}

function deterministicSecret(owner, epoch, index) {
  return createHash("sha256")
    .update("glory-dump-localnet-secret-v1")
    .update(owner.toBuffer())
    .update(u64LittleEndian(epoch))
    .update(u32LittleEndian(index))
    .digest();
}

function commitmentFor(secret, owner, epoch) {
  return createHash("sha256")
    .update("glory-dump-commitment-v3")
    .update(secret)
    .update(owner.toBuffer())
    .update(u64LittleEndian(epoch))
    .digest();
}

async function fundWallets(connection, payer, wallets, lamports) {
  for (let offset = 0; offset < wallets.length; offset += 8) {
    const transaction = new Transaction();
    for (const wallet of wallets.slice(offset, offset + 8)) {
      transaction.add(SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: wallet.publicKey,
        lamports,
      }));
    }
    await sendAndConfirmTransaction(connection, transaction, [payer], {
      commitment: "confirmed",
      preflightCommitment: "confirmed",
    });
  }
}

async function airdrop(connection, recipient, lamports) {
  const signature = await connection.requestAirdrop(recipient, lamports);
  const latest = await connection.getLatestBlockhash("confirmed");
  await connection.confirmTransaction({ signature, ...latest }, "confirmed");
}

async function waitForRpc(connection, child, expectedProgram) {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`validator exited before RPC became ready\n${validatorLog.join("")}`);
    }
    try {
      await connection.getLatestBlockhash("confirmed");
      const slot = await connection.getSlot("confirmed");
      const program = await connection.getAccountInfo(expectedProgram, "confirmed");
      if (slot > 0 && program?.executable && await hasDeployedProgramData(connection, program)) return;
    } catch {
      await sleep(200);
    }
  }
  throw new Error(`validator RPC did not become ready\n${validatorLog.join("")}`);
}

async function hasDeployedProgramData(connection, program) {
  if (!program.owner.equals(UPGRADEABLE_LOADER_ID)) return true;
  if (program.data.length !== 36 || program.data.readUInt32LE(0) !== 2) return false;
  const programDataAddress = new PublicKey(program.data.subarray(4));
  const programData = await connection.getAccountInfo(programDataAddress, "confirmed");
  return Boolean(
    programData
      && programData.owner.equals(UPGRADEABLE_LOADER_ID)
      && programData.data.length >= 45
      && programData.data.readUInt32LE(0) === 3,
  );
}

async function waitForChainTimestamp(connection, target) {
  const deadline = Date.now() + 300_000;
  let latestSlot = 0;
  let latestTimestamp = null;
  while (Date.now() < deadline) {
    latestSlot = await connection.getSlot("confirmed");
    latestTimestamp = await connection.getBlockTime(latestSlot);
    if (latestTimestamp !== null && latestTimestamp >= target) return;
    await sleep(250);
  }
  throw new Error(
    `chain clock did not reach ${target}; last observed ${latestTimestamp ?? "unknown"} at slot ${latestSlot}`,
  );
}

async function observeMany(connection, signatures, label) {
  return Promise.all(signatures.map((signature) => observeTransaction(connection, signature, label)));
}

async function observeTransaction(connection, signature, label) {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const transaction = await connection.getTransaction(signature, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });
    if (transaction?.meta) {
      assert.equal(transaction.meta.err, null, `${label} transaction must succeed`);
      return {
        label,
        signature,
        fee: transaction.meta.fee,
        computeUnits: transaction.meta.computeUnitsConsumed ?? 0,
      };
    }
    await sleep(100);
  }
  throw new Error(`confirmed transaction ${signature} was not retrievable`);
}

async function expectFailure(promise, label) {
  try {
    await promise;
  } catch {
    return;
  }
  throw new Error(`${label} unexpectedly succeeded`);
}

async function parallelMap(values, concurrency, operation) {
  const results = new Array(values.length);
  let cursor = 0;
  async function worker() {
    while (cursor < values.length) {
      const index = cursor;
      cursor += 1;
      results[index] = await operation(values[index], index);
    }
  }
  await Promise.all(Array.from({ length: Math.min(concurrency, values.length) }, worker));
  return results;
}

function indexOfLargestSpendable(lanes) {
  let best = 0;
  for (let index = 1; index < lanes.length; index += 1) {
    if (toBigInt(lanes[index].balance) > toBigInt(lanes[best].balance)) best = index;
  }
  return best;
}

function toBigInt(value) {
  return BigInt(value.toString());
}

function toNumber(value) {
  return Number(value.toString());
}

function divCeil(value, divisor) {
  return (value + divisor - 1n) / divisor;
}

function min(left, right) {
  return left < right ? left : right;
}

function max(left, right) {
  return left > right ? left : right;
}

function u64LittleEndian(value) {
  const bytes = Buffer.alloc(8);
  bytes.writeBigUInt64LE(BigInt(value));
  return bytes;
}

function u32LittleEndian(value) {
  const bytes = Buffer.alloc(4);
  bytes.writeUInt32LE(Number(value));
  return bytes;
}

function sleep(milliseconds) {
  return new Promise((resolvePromise) => setTimeout(resolvePromise, milliseconds));
}

function onceExit(child) {
  if (child.exitCode !== null) return Promise.resolve(child.exitCode);
  return new Promise((resolvePromise) => child.once("exit", resolvePromise));
}
