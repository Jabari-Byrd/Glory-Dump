import {
  AnchorProvider,
  BN,
  Program,
  utils,
  type IdlAccounts,
  type Wallet,
} from "@anchor-lang/core";
import {
  Connection,
  PublicKey,
  SystemProgram,
  type Transaction,
  type VersionedTransaction,
} from "@solana/web3.js";

import { appConfig } from "./config";
import idl from "./idl/glory_dump.json";
import type { GloryDump } from "./idl/glory_dump";
import {
  bytesToHex,
  commitmentFor,
  projectedWeightedScore,
} from "./rules";
import type {
  ActionRequest,
  EpochView,
  GameGateway,
  LaneView,
  Phase,
  PlayerView,
  StrategySnapshot,
} from "./types";

interface InjectedSolanaWallet {
  publicKey: PublicKey | null;
  isConnected?: boolean;
  connect(): Promise<{ publicKey: PublicKey }>;
  signTransaction<T extends Transaction | VersionedTransaction>(transaction: T): Promise<T>;
  signAllTransactions<T extends Transaction | VersionedTransaction>(transactions: T[]): Promise<T[]>;
}

declare global {
  interface Window {
    solana?: InjectedSolanaWallet;
  }
}

const TOKEN_PROGRAM_ID = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
);

export class SolanaGateway implements GameGateway {
  private readonly connection = new Connection(appConfig.rpcUrl, "confirmed");
  private readonly programId: PublicKey;
  private program?: Program<GloryDump>;
  private wallet?: InjectedSolanaWallet;

  constructor() {
    if (!appConfig.liveEnabled) throw new Error("No Solana program ID is configured");
    this.programId = new PublicKey(appConfig.programId);
  }

  async connect(): Promise<StrategySnapshot> {
    const wallet = window.solana;
    if (!wallet) {
      throw new Error("No injected Solana wallet was found. Install or open a compatible wallet.");
    }
    const connected = await wallet.connect();
    if (!wallet.publicKey || !wallet.publicKey.equals(connected.publicKey)) {
      throw new Error("The wallet did not return a stable public key");
    }
    this.wallet = wallet;
    const provider = new AnchorProvider(
      this.connection,
      wallet as unknown as Wallet,
      { commitment: "confirmed", preflightCommitment: "confirmed" },
    );
    const configuredIdl = { ...idl, address: this.programId.toBase58() } as unknown as GloryDump;
    this.program = new Program(configuredIdl, provider);
    return this.refresh();
  }

  async refresh(): Promise<StrategySnapshot> {
    const program = this.requireProgram();
    const wallet = this.requireWallet();
    const owner = wallet.publicKey;
    if (!owner) throw new Error("Wallet disconnected");

    const [protocolAddress] = this.protocolPda();
    const protocol = await program.account.protocol.fetch(protocolAddress);
    const epochNumber = toBigInt(protocol.currentEpoch);
    const [epochAddress] = this.epochPda(epochNumber);
    const epochRaw = await program.account.epoch.fetch(epochAddress);
    const epoch = mapEpoch(epochRaw);
    const [leaderboardAddress] = this.leaderboardPda(epochNumber);
    const epochFilter = {
      memcmp: { offset: 8, bytes: utils.bytes.bs58.encode(u64LittleEndian(epochNumber)) },
    };

    const [currentPlayers, currentLanes, leaderboard, tokenAccounts] = await Promise.all([
      program.account.playerEpoch.all([epochFilter]),
      program.account.balanceLane.all([epochFilter]),
      program.account.leaderboard.fetch(leaderboardAddress),
      this.connection.getParsedTokenAccountsByOwner(owner, { mint: protocol.gloryMint }),
    ]);

    const lanesByOwner = new Map<string, typeof currentLanes>();
    for (const row of currentLanes) {
      const key = row.account.owner.toBase58();
      const lanes = lanesByOwner.get(key) ?? [];
      lanes.push(row);
      lanesByOwner.set(key, lanes);
    }
    const winners = new Map(
      leaderboard.entries.map((entry) => [entry.player.toBase58(), entry.claimed] as const),
    );
    const now = Math.floor(Date.now() / 1000);
    const players = currentPlayers.map(({ account }) => {
      const address = account.owner.toBase58();
      const laneRows = (lanesByOwner.get(address) ?? []).sort(
        (left, right) => left.account.index - right.account.index,
      );
      const lanes = laneRows.map((row) => mapLane(row.account));
      while (lanes.length < 4) lanes.push(emptyLane(lanes.length));
      const balance = lanes.reduce((total, lane) => total + lane.balance, 0n);
      const projectedScore = account.settled
        ? toBigInt(account.finalScore)
        : laneRows.reduce(
              (total, row) => total + projectedWeightedScore(
                toBigInt(row.account.cumulativeWeighted),
                toBigInt(row.account.balance),
                toNumber(row.account.lastCheckpointAt),
                now,
                epoch.activeStartsAt,
                epoch.activeEndsAt,
              ),
              0n,
            );
      return {
        address,
        alias: aliasFor(address),
        startingAllocation: toBigInt(account.startingAllocation),
        balance,
        projectedScore,
        impactPpm: toBigInt(account.impactPpm),
        lateImpactPpm: toBigInt(account.lateImpactPpm),
        meaningfulActions: account.meaningfulActions,
        distinctOpponents: account.distinctOpponents,
        dumpHeat: {
          units: account.dumpHeat.units,
          updatedAt: toNumber(account.dumpHeat.updatedAt),
        },
        absorbHeat: {
          units: account.absorbHeat.units,
          updatedAt: toNumber(account.absorbHeat.updatedAt),
        },
        lanes,
        revealed: account.revealed,
        allocationClaimed: account.allocationClaimed,
        settled: account.settled,
        eligible: account.revealed && account.meaningfulActions > 0,
        bondClaimed: account.bondClaimed,
        isWinner: winners.has(address),
        rewardClaimed: winners.get(address) ?? false,
        badges: Number(toBigInt(account.badges)),
        isSelf: account.owner.equals(owner),
      } satisfies PlayerView;
    });
    const gloryBalance = tokenAccounts.value.reduce((total, item) => {
      const info = item.account.data.parsed.info as { tokenAmount: { amount: string } };
      return total + BigInt(info.tokenAmount.amount);
    }, 0n);

    return {
      mode: "live",
      epoch,
      players,
      feed: [{
        id: `live-${epoch.number}`,
        timestamp: now,
        kind: "system",
        message: "Live accounts loaded. Historical battle feed requires the optional indexer.",
      }],
      walletAddress: owner.toBase58(),
      gloryBalance,
      clusterLabel: appConfig.clusterLabel,
    };
  }

  async register(): Promise<string> {
    const program = this.requireProgram();
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const storageKey = secretKey(this.programId, owner, epoch);
    const saved = localStorage.getItem(storageKey);
    const secret = saved
      ? base64ToBytes(saved)
      : crypto.getRandomValues(new Uint8Array(32));
    if (secret.length !== 32) {
      throw new Error("The saved reveal secret is corrupt. Restore its backup before joining.");
    }
    const commitment = await commitmentFor(secret, owner.toBytes(), epoch);
    const [protocol] = this.protocolPda();
    const [epochAddress] = this.epochPda(epoch);
    const [player] = this.playerPda(epoch, owner);
    const lanes = Array.from({ length: 4 }, (_, index) => this.lanePda(epoch, owner, index)[0]);

    // Persist before requesting a signature. A network timeout after a
    // confirmed transaction must not strand the player without the reveal.
    localStorage.setItem(storageKey, bytesToBase64(secret));
    const signature = await program.methods
      .register([...commitment])
      .accountsPartial({
        payer: owner,
        protocol,
        epoch: epochAddress,
        player,
        laneZero: lanes[0]!,
        laneOne: lanes[1]!,
        laneTwo: lanes[2]!,
        laneThree: lanes[3]!,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    downloadRevealBackup(this.programId, owner, epoch, secret, commitment);
    return signature;
  }

  async reveal(): Promise<string> {
    const program = this.requireProgram();
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const encoded = localStorage.getItem(secretKey(this.programId, owner, epoch));
    if (!encoded) {
      throw new Error("This browser does not have the reveal secret. Restore the saved secret first.");
    }
    const secret = base64ToBytes(encoded);
    const [epochAddress] = this.epochPda(epoch);
    const [player] = this.playerPda(epoch, owner);
    return program.methods
      .reveal([...secret])
      .accountsPartial({ owner, epoch: epochAddress, player })
      .rpc();
  }

  async claimAllocation(): Promise<string> {
    const program = this.requireProgram();
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const [epochAddress] = this.epochPda(epoch);
    const [player] = this.playerPda(epoch, owner);
    const lanes = Array.from({ length: 4 }, (_, index) => this.lanePda(epoch, owner, index)[0]);
    return program.methods
      .claimAllocation()
      .accountsPartial({
        owner,
        epoch: epochAddress,
        player,
        laneZero: lanes[0]!,
        laneOne: lanes[1]!,
        laneTwo: lanes[2]!,
        laneThree: lanes[3]!,
      })
      .rpc();
  }

  async advanceEpoch(): Promise<string> {
    const program = this.requireProgram();
    const payer = this.requirePublicKey();
    const [protocolAddress] = this.protocolPda();
    const protocol = await program.account.protocol.fetch(protocolAddress);
    const number = toBigInt(protocol.currentEpoch);
    const [epochAddress] = this.epochPda(number);
    const epoch = await program.account.epoch.fetch(epochAddress);
    const phase = phaseOf(epoch.phase);
    const now = Math.floor(Date.now() / 1_000);
    const transitionAccounts = { protocol: protocolAddress, epoch: epochAddress };

    if (phase === "registration") {
      if (now < toNumber(epoch.registrationEndsAt)) throw new Error("Registration is still open");
      const method = epoch.participantCount >= 20
        ? program.methods.beginReveal()
        : program.methods.cancelEpoch();
      return method.accountsPartial(transitionAccounts).rpc();
    }
    if (phase === "reveal") {
      if (now < toNumber(epoch.revealEndsAt)) throw new Error("The reveal window is still open");
      const method = epoch.revealedCount >= 2
        ? program.methods.sealRandomness()
        : program.methods.cancelEpoch();
      return method.accountsPartial(transitionAccounts).rpc();
    }
    if (phase === "planning") {
      if (now >= toNumber(epoch.activeEndsAt)) {
        return program.methods.beginSettlement().accountsPartial(transitionAccounts).rpc();
      }
      if (now < toNumber(epoch.activeStartsAt)) throw new Error("The planning hour is still active");
      return program.methods.beginActive().accountsPartial(transitionAccounts).rpc();
    }
    if (phase === "active") {
      if (now < toNumber(epoch.activeEndsAt)) throw new Error("The active epoch has not ended");
      return program.methods.beginSettlement().accountsPartial(transitionAccounts).rpc();
    }
    if (phase === "settling") {
      if (epoch.settledCount !== epoch.participantCount) {
        throw new Error("Settle every player before completing the epoch");
      }
      const [leaderboard] = this.leaderboardPda(number);
      return program.methods
        .completeEpoch()
        .accountsPartial({
          finalizer: payer,
          protocol: protocolAddress,
          epoch: epochAddress,
          leaderboard,
        })
        .rpc();
    }
    const next = number + 1n;
    const [nextEpoch] = this.epochPda(next);
    const [nextLeaderboard] = this.leaderboardPda(next);
    return program.methods
      .openNextEpoch(new BN(next.toString()))
      .accountsPartial({
        payer,
        protocol: protocolAddress,
        previousEpoch: epochAddress,
        epoch: nextEpoch,
        leaderboard: nextLeaderboard,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async settleNextPlayer(): Promise<string> {
    const program = this.requireProgram();
    const keeper = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const filter = {
      memcmp: { offset: 8, bytes: utils.bytes.bs58.encode(u64LittleEndian(epoch)) },
    };
    const players = await program.account.playerEpoch.all([filter]);
    const next = players.find(({ account }) => !account.settled);
    if (!next) throw new Error("Every registered player is already settled");

    const owner = next.account.owner;
    const [epochAddress] = this.epochPda(epoch);
    const [leaderboard] = this.leaderboardPda(epoch);
    const [player] = this.playerPda(epoch, owner);
    const [keeperCredit] = this.keeperPda(epoch, keeper);
    const lanes = Array.from({ length: 4 }, (_, index) => this.lanePda(epoch, owner, index)[0]);
    return program.methods
      .settlePlayer()
      .accountsPartial({
        keeper,
        epoch: epochAddress,
        leaderboard,
        player,
        laneZero: lanes[0]!,
        laneOne: lanes[1]!,
        laneTwo: lanes[2]!,
        laneThree: lanes[3]!,
        keeperCredit,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async claimPlayerReward(): Promise<string> {
    const program = this.requireProgram();
    const owner = this.requirePublicKey();
    const [protocolAddress] = this.protocolPda();
    const protocol = await program.account.protocol.fetch(protocolAddress);
    const epoch = toBigInt(protocol.currentEpoch);
    const [epochAddress] = this.epochPda(epoch);
    const [leaderboard] = this.leaderboardPda(epoch);
    const [player] = this.playerPda(epoch, owner);
    const destination = associatedTokenAddress(owner, protocol.gloryMint);
    return program.methods
      .claimPlayerReward()
      .accountsPartial({
        owner,
        protocol: protocolAddress,
        epoch: epochAddress,
        leaderboard,
        player,
        gloryMint: protocol.gloryMint,
        destination,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async claimBond(): Promise<string> {
    const program = this.requireProgram();
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const [epochAddress] = this.epochPda(epoch);
    const [player] = this.playerPda(epoch, owner);
    return program.methods
      .claimBond()
      .accountsPartial({ owner, epoch: epochAddress, player })
      .rpc();
  }

  async refreshBadges(): Promise<string> {
    const program = this.requireProgram();
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const [epochAddress] = this.epochPda(epoch);
    const [player] = this.playerPda(epoch, owner);
    return program.methods
      .refreshBadges()
      .accountsPartial({ owner, epoch: epochAddress, player })
      .rpc();
  }

  async exportRevealSecret(): Promise<void> {
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const encoded = localStorage.getItem(secretKey(this.programId, owner, epoch));
    if (!encoded) throw new Error("No reveal secret is saved in this browser for the current epoch");
    const secret = base64ToBytes(encoded);
    if (secret.length !== 32) throw new Error("The saved reveal secret is corrupt");
    const commitment = await commitmentFor(secret, owner.toBytes(), epoch);
    downloadRevealBackup(this.programId, owner, epoch, secret, commitment);
  }

  async importRevealSecret(file: File): Promise<void> {
    if (file.size > 16_384) throw new Error("Reveal backup is unexpectedly large");
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const backup = parseRevealBackup(await file.text());
    if (
      backup.programId !== this.programId.toBase58()
      || backup.owner !== owner.toBase58()
      || backup.epoch !== epoch.toString()
    ) {
      throw new Error("That reveal backup belongs to a different program, wallet, or epoch");
    }
    const secret = base64ToBytes(backup.secretBase64);
    if (secret.length !== 32) throw new Error("Reveal backup secret must contain exactly 32 bytes");
    const commitment = await commitmentFor(secret, owner.toBytes(), epoch);
    if (bytesToHex(commitment) !== backup.commitmentHex) {
      throw new Error("Reveal backup failed its commitment integrity check");
    }
    const [player] = this.playerPda(epoch, owner);
    const onChain = await this.requireProgram().account.playerEpoch.fetch(player);
    if (bytesToHex(new Uint8Array(onChain.commitment)) !== backup.commitmentHex) {
      throw new Error("Reveal backup does not match this player's on-chain commitment");
    }
    localStorage.setItem(secretKey(this.programId, owner, epoch), backup.secretBase64);
  }

  async act(request: ActionRequest): Promise<string> {
    const program = this.requireProgram();
    const owner = this.requirePublicKey();
    const epoch = await this.currentEpochNumber();
    const [epochAddress] = this.epochPda(epoch);
    const [actorPlayer] = this.playerPda(epoch, owner);

    if (request.kind === "redirect") {
      const laneIndex = request.actorLane ?? 0;
      const [player] = this.playerPda(epoch, owner);
      const [lane] = this.lanePda(epoch, owner, laneIndex);
      return program.methods
        .armRedirect(laneIndex)
        .accountsPartial({ signer: owner, epoch: epochAddress, player, lane })
        .rpc();
    }

    if (!request.target || request.amount === undefined) {
      throw new Error("A target and amount are required");
    }
    const target = new PublicKey(request.target);
    const [targetPlayer] = this.playerPda(epoch, target);
    const [rivalry] = this.rivalryPda(epoch, owner, target);
    const amount = new BN(request.amount.toString());
    const actorLaneIndex = request.actorLane ?? 0;
    let targetLaneIndex = request.targetLane ?? 0;

    if (request.kind === "dump") {
      const [epochAccount, rivalryAccount] = await Promise.all([
        program.account.epoch.fetch(epochAddress),
        program.account.rivalry.fetchNullable(rivalry),
      ]);
      targetLaneIndex = await deterministicTargetLane(
        new Uint8Array(epochAccount.seed),
        owner,
        target,
        epoch,
        rivalryAccount?.actionCount ?? 0,
      );
    }
    const [actorLane] = this.lanePda(epoch, owner, actorLaneIndex);
    const [targetLane] = this.lanePda(epoch, target, targetLaneIndex);
    const accounts = {
      signer: owner,
      epoch: epochAddress,
      actorPlayer,
      targetPlayer,
      actorLane,
      targetLane,
      rivalry,
      systemProgram: SystemProgram.programId,
    };
    if (request.kind === "dump") {
      return program.methods
        .dump(amount, actorLaneIndex, targetLaneIndex)
        .accountsPartial(accounts)
        .rpc();
    }
    return program.methods
      .absorb(amount, actorLaneIndex, targetLaneIndex)
      .accountsPartial(accounts)
      .rpc();
  }

  private requireProgram(): Program<GloryDump> {
    if (!this.program) throw new Error("Connect a wallet first");
    return this.program;
  }

  private requireWallet(): InjectedSolanaWallet {
    if (!this.wallet) throw new Error("Connect a wallet first");
    return this.wallet;
  }

  private requirePublicKey(): PublicKey {
    const publicKey = this.requireWallet().publicKey;
    if (!publicKey) throw new Error("Wallet disconnected");
    return publicKey;
  }

  private async currentEpochNumber(): Promise<bigint> {
    const [protocol] = this.protocolPda();
    const account = await this.requireProgram().account.protocol.fetch(protocol);
    return toBigInt(account.currentEpoch);
  }

  private protocolPda(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync([new TextEncoder().encode("protocol")], this.programId);
  }

  private epochPda(epoch: bigint): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("epoch"), u64LittleEndian(epoch)],
      this.programId,
    );
  }

  private leaderboardPda(epoch: bigint): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("leaderboard"), u64LittleEndian(epoch)],
      this.programId,
    );
  }

  private playerPda(epoch: bigint, owner: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("player"), u64LittleEndian(epoch), owner.toBytes()],
      this.programId,
    );
  }

  private lanePda(epoch: bigint, owner: PublicKey, index: number): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        new TextEncoder().encode("lane"),
        u64LittleEndian(epoch),
        owner.toBytes(),
        Uint8Array.of(index),
      ],
      this.programId,
    );
  }

  private rivalryPda(epoch: bigint, actor: PublicKey, target: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        new TextEncoder().encode("rivalry"),
        u64LittleEndian(epoch),
        actor.toBytes(),
        target.toBytes(),
      ],
      this.programId,
    );
  }

  private keeperPda(epoch: bigint, keeper: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("keeper"), u64LittleEndian(epoch), keeper.toBytes()],
      this.programId,
    );
  }
}

type GameAccounts = IdlAccounts<GloryDump>;
type EpochAccount = GameAccounts["epoch"];
type LaneAccount = GameAccounts["balanceLane"];

interface RevealBackup {
  version: 1;
  programId: string;
  owner: string;
  epoch: string;
  secretBase64: string;
  commitmentHex: string;
}

function mapEpoch(account: EpochAccount): EpochView {
  const phase = phaseOf(account.phase);
  const registrationEndsAt = toNumber(account.registrationEndsAt);
  const revealEndsAt = toNumber(account.revealEndsAt);
  const activeStartsAt = toNumber(account.activeStartsAt);
  const activeEndsAt = toNumber(account.activeEndsAt);
  const phaseEndsAt = phase === "registration"
    ? registrationEndsAt
    : phase === "reveal"
      ? revealEndsAt
      : phase === "planning"
        ? activeStartsAt
        : activeEndsAt;
  return {
    number: toBigInt(account.number),
    phase,
    phaseEndsAt,
    activeStartsAt,
    activeEndsAt,
    participantCount: account.participantCount,
    revealedCount: account.revealedCount,
    settledCount: account.settledCount,
    eligibleCount: account.eligibleCount,
    winnerCount: account.winnerCount,
    playerRewardPool: toBigInt(account.playerRewardPool),
    keeperRewardPool: toBigInt(account.keeperRewardPool),
  };
}

function mapLane(account: LaneAccount): LaneView {
  return {
    index: account.index,
    balance: toBigInt(account.balance),
    guard: toBigInt(account.guard),
    lockedAmount: toBigInt(account.lockedAmount),
    lockedUntil: toNumber(account.lockedUntil),
    redirectArmed: account.redirectArmed,
    redirectReadyAt: toNumber(account.redirectReadyAt),
    redirectedVolume: toBigInt(account.redirectedVolume),
  };
}

function emptyLane(index: number): LaneView {
  return {
    index,
    balance: 0n,
    guard: 0n,
    lockedAmount: 0n,
    lockedUntil: 0,
    redirectArmed: false,
    redirectReadyAt: 0,
    redirectedVolume: 0n,
  };
}

function toBigInt(value: BN | number): bigint {
  return BigInt(value.toString());
}

function toNumber(value: BN | number): number {
  return Number(value.toString());
}

function u64LittleEndian(value: bigint): Uint8Array {
  const bytes = new Uint8Array(8);
  let remaining = value;
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Number(remaining & 0xffn);
    remaining >>= 8n;
  }
  return bytes;
}

async function deterministicTargetLane(
  seed: Uint8Array,
  actor: PublicKey,
  target: PublicKey,
  epoch: bigint,
  actionCount: number,
): Promise<number> {
  const count = new Uint8Array(4);
  new DataView(count.buffer).setUint32(0, actionCount, true);
  const digest = await crypto.subtle.digest(
    "SHA-256",
    concatenate(
      new TextEncoder().encode("glory-dump-target-lane-v3"),
      seed,
      actor.toBytes(),
      target.toBytes(),
      u64LittleEndian(epoch),
      count,
    ).buffer as ArrayBuffer,
  );
  return new Uint8Array(digest)[0]! % 4;
}

function concatenate(...arrays: Uint8Array[]): Uint8Array {
  const output = new Uint8Array(arrays.reduce((total, item) => total + item.length, 0));
  let offset = 0;
  for (const item of arrays) {
    output.set(item, offset);
    offset += item.length;
  }
  return output;
}

function secretKey(program: PublicKey, owner: PublicKey, epoch: bigint): string {
  return `glory-dump:reveal:${program.toBase58()}:${owner.toBase58()}:${epoch}`;
}

function associatedTokenAddress(owner: PublicKey, mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [owner.toBytes(), TOKEN_PROGRAM_ID.toBytes(), mint.toBytes()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  )[0];
}

function downloadRevealBackup(
  program: PublicKey,
  owner: PublicKey,
  epoch: bigint,
  secret: Uint8Array,
  commitment: Uint8Array,
): void {
  const backup: RevealBackup = {
    version: 1,
    programId: program.toBase58(),
    owner: owner.toBase58(),
    epoch: epoch.toString(),
    secretBase64: bytesToBase64(secret),
    commitmentHex: bytesToHex(commitment),
  };
  const blob = new Blob([`${JSON.stringify(backup, null, 2)}\n`], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `glory-dump-reveal-epoch-${epoch}-${shortKey(owner)}.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}

function parseRevealBackup(raw: string): RevealBackup {
  let candidate: unknown;
  try {
    candidate = JSON.parse(raw);
  } catch {
    throw new Error("Reveal backup is not valid JSON");
  }
  if (!isRecord(candidate)) throw new Error("Reveal backup must be a JSON object");
  if (
    candidate.version !== 1
    || typeof candidate.programId !== "string"
    || typeof candidate.owner !== "string"
    || typeof candidate.epoch !== "string"
    || typeof candidate.secretBase64 !== "string"
    || typeof candidate.commitmentHex !== "string"
    || !/^[0-9a-f]{64}$/.test(candidate.commitmentHex)
  ) {
    throw new Error("Reveal backup has an invalid or unsupported format");
  }
  return candidate as unknown as RevealBackup;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function shortKey(value: PublicKey): string {
  return value.toBase58().slice(0, 8);
}

function phaseOf(value: EpochAccount["phase"]): Phase {
  return (Object.keys(value)[0] ?? "registration") as Phase;
}

function bytesToBase64(bytes: Uint8Array): string {
  return btoa(String.fromCharCode(...bytes));
}

function base64ToBytes(value: string): Uint8Array {
  return Uint8Array.from(atob(value), (character) => character.charCodeAt(0));
}

function aliasFor(address: string): string {
  const adjectives = ["Quiet", "Broken", "Reverse", "Paper", "Hollow", "Tiny", "Unpaid", "Final"];
  const titles = ["Baron", "Whale", "Peasant", "Oracle", "Knight", "Wallet", "Magnet", "Crown"];
  const first = address.charCodeAt(0) % adjectives.length;
  const last = address.charCodeAt(address.length - 1) % titles.length;
  return `${adjectives[first]} ${titles[last]}`;
}
