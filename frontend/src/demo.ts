import {
  DUMP_TIER_SIZE,
  HEAT_CAP,
  heatAt,
  heatCost,
  minimumAction,
  projectedPlayerScore,
  rankPlayers,
} from "./rules";
import type {
  ActionRequest,
  FeedItem,
  GameGateway,
  LaneView,
  PlayerView,
  StrategySnapshot,
} from "./types";

const ALIASES = [
  "Paper Crown",
  "Zero Baron",
  "Trash Oracle",
  "Debt Wizard",
  "Small Fry",
  "Bag Magnet",
  "Exit Liquidity",
  "Quiet Peasant",
  "Dump Knight",
  "Reverse Whale",
  "Hot Potato",
  "Poor Richard",
  "Mud Lobbyist",
  "Last Wallet",
  "Compost King",
  "Negative Alpha",
  "Broken Piggy",
  "The Unbanked",
] as const;

const SELF_ADDRESS = "7GLorYp4YERwALLeT1111111111111111111111111";

export class DemoGateway implements GameGateway {
  private snapshot: StrategySnapshot;

  constructor(now = Math.floor(Date.now() / 1000)) {
    const activeStartsAt = now - 22 * 86_400;
    const activeEndsAt = activeStartsAt + 30 * 86_400;
    const players = ALIASES.map((alias, index) => makePlayer(alias, index, now));
    players[7] = makePlayer(ALIASES[7]!, 7, now, true);
    for (const winner of rankPlayers(players).slice(0, 5)) winner.isWinner = true;
    this.snapshot = {
      mode: "demo",
      epoch: {
        number: 19n,
        phase: "active",
        phaseEndsAt: activeEndsAt,
        activeStartsAt,
        activeEndsAt,
        participantCount: 84,
        revealedCount: 79,
        settledCount: 0,
        eligibleCount: 0,
        winnerCount: 5,
        playerRewardPool: 90_125n * 1_000_000n,
        keeperRewardPool: 875n * 1_000_000n,
      },
      players,
      feed: initialFeed(now, players),
      walletAddress: SELF_ADDRESS,
      gloryBalance: 1_240_500_000n,
      clusterLabel: "Strategy Room demo",
    };
  }

  async connect(): Promise<StrategySnapshot> {
    return this.refresh();
  }

  async refresh(): Promise<StrategySnapshot> {
    const copy = structuredClone(this.snapshot);
    const now = Math.floor(Date.now() / 1000);
    for (const player of copy.players) {
      if (!player.settled) {
        player.projectedScore = projectedPlayerScore(
          player.lanes,
          now,
          copy.epoch.activeStartsAt,
          copy.epoch.activeEndsAt,
        );
      }
    }
    return copy;
  }

  async act(request: ActionRequest): Promise<string> {
    const actor = this.snapshot.players.find((player) => player.isSelf);
    if (!actor) throw new Error("Demo player is missing");
    const now = Math.floor(Date.now() / 1000);
    if (request.kind === "redirect") {
      const lane = actor.lanes[request.actorLane ?? 0];
      if (!lane || lane.guard === 0n) throw new Error("That lane has no Guard to arm");
      if (lane.redirectArmed || now < lane.redirectReadyAt) {
        throw new Error("That REDIRECT lane is not ready");
      }
      lane.redirectArmed = true;
      this.unshiftFeed({
        kind: "redirect",
        actor: actor.address,
        message: `${actor.alias} armed REDIRECT on lane ${lane.index + 1}.`,
      });
      return "demo-redirect";
    }

    const target = this.snapshot.players.find((player) => player.address === request.target);
    const amount = request.amount ?? 0n;
    if (!target || target.isSelf) throw new Error("Choose another territory first");
    if (amount < minimumAction(actor.startingAllocation)) {
      throw new Error("Amount is below the meaningful-action minimum");
    }
    const actorLane = actor.lanes[request.actorLane ?? 0];
    const targetLane = target.lanes[request.targetLane ?? 0];
    if (!actorLane || !targetLane) throw new Error("Choose valid lanes");

    if (request.kind === "dump") {
      const cost = heatCost(amount, actor.startingAllocation);
      const currentHeat = heatAt(actor.dumpHeat, now);
      if (currentHeat + cost > HEAT_CAP) throw new Error("DUMP heat is full");
      if (spendable(actorLane, now) < amount) {
        throw new Error("Not enough unlocked DUMP in that lane");
      }
      checkpointLane(actorLane, now, this.snapshot);
      checkpointLane(targetLane, now, this.snapshot);
      const redirected = targetLane.redirectArmed ? amount < targetLane.guard ? amount : targetLane.guard : 0n;
      const landed = amount - redirected;
      actorLane.balance = actorLane.balance - amount + redirected;
      targetLane.balance += landed;
      if (redirected > 0n) {
        targetLane.guard -= redirected;
        targetLane.redirectArmed = false;
        targetLane.redirectReadyAt = now + 15 * 60;
        targetLane.redirectedVolume += redirected;
      }
      actor.dumpHeat = { units: currentHeat + cost, updatedAt: now };
      actor.balance = sumLanes(actor.lanes);
      target.balance = sumLanes(target.lanes);
      refreshProjectedScore(actor, now, this.snapshot);
      refreshProjectedScore(target, now, this.snapshot);
      actor.meaningfulActions += 1;
      actor.impactPpm += amount * 1_000_000n / actor.startingAllocation;
      this.unshiftFeed({
        kind: "dump",
        actor: actor.address,
        target: target.address,
        amount,
        message: redirected > 0n
          ? `${target.alias} ricocheted part of ${actor.alias}'s DUMP.`
          : `${actor.alias} force-fed ${target.alias}.`,
      });
      return "demo-dump";
    }

    const cost = heatCost(amount, actor.startingAllocation);
    const currentHeat = heatAt(actor.absorbHeat, now);
    if (currentHeat + cost > HEAT_CAP) throw new Error("ABSORB heat is full");
    if (spendable(targetLane, now) < amount) {
      throw new Error("The target lane does not have enough unlocked DUMP");
    }
    checkpointLane(actorLane, now, this.snapshot);
    checkpointLane(targetLane, now, this.snapshot);
    targetLane.balance -= amount;
    actorLane.balance += amount;
    actorLane.lockedAmount += amount;
    actorLane.lockedUntil = now + 6 * 60 * 60;
    const laneGuardCap = actor.startingAllocation / 16n;
    actorLane.guard = min(laneGuardCap, actorLane.guard + amount / 2n);
    actor.absorbHeat = { units: currentHeat + cost, updatedAt: now };
    actor.balance = sumLanes(actor.lanes);
    target.balance = sumLanes(target.lanes);
    refreshProjectedScore(actor, now, this.snapshot);
    refreshProjectedScore(target, now, this.snapshot);
    actor.meaningfulActions += 1;
    actor.impactPpm += amount * 1_000_000n / actor.startingAllocation;
    this.unshiftFeed({
      kind: "absorb",
      actor: actor.address,
      target: target.address,
      amount,
      message: `${actor.alias} absorbed DUMP from ${target.alias} and forged Guard.`,
    });
    return "demo-absorb";
  }

  async register(): Promise<string> {
    this.unshiftFeed({ kind: "system", message: "Demo registration commitment prepared." });
    return "demo-register";
  }

  async reveal(): Promise<string> {
    this.unshiftFeed({ kind: "system", message: "Demo secret revealed." });
    return "demo-reveal";
  }

  async claimAllocation(): Promise<string> {
    this.unshiftFeed({ kind: "system", message: "Demo allocation already claimed." });
    return "demo-allocation";
  }

  async advanceEpoch(): Promise<string> {
    this.unshiftFeed({ kind: "system", message: "A public epoch transition was simulated." });
    return "demo-advance";
  }

  async settleNextPlayer(): Promise<string> {
    const player = this.snapshot.players.find((candidate) => !candidate.settled);
    if (!player) throw new Error("Every demo player is already settled");
    player.settled = true;
    this.snapshot.epoch.settledCount += 1;
    this.unshiftFeed({ kind: "system", message: `${player.alias}'s final score was settled.` });
    return "demo-settle";
  }

  async claimPlayerReward(): Promise<string> {
    const player = this.snapshot.players.find((candidate) => candidate.isSelf);
    if (!player?.isWinner) throw new Error("The demo wallet is not currently in the winner set");
    if (player.rewardClaimed) throw new Error("The demo reward was already claimed");
    player.rewardClaimed = true;
    this.unshiftFeed({ kind: "reward", message: `${player.alias} claimed demo GLORY.` });
    return "demo-glory";
  }

  async claimBond(): Promise<string> {
    const player = this.snapshot.players.find((candidate) => candidate.isSelf);
    if (!player) throw new Error("Demo player is missing");
    if (player.bondClaimed) throw new Error("The demo bond was already claimed");
    player.bondClaimed = true;
    this.unshiftFeed({ kind: "reward", message: `${player.alias} reclaimed the demo entry bond.` });
    return "demo-bond";
  }

  async refreshBadges(): Promise<string> {
    this.unshiftFeed({ kind: "system", message: "Demo badges synchronized." });
    return "demo-badges";
  }

  async exportRevealSecret(): Promise<void> {
    this.unshiftFeed({ kind: "system", message: "Reveal backups are enabled in live-wallet mode." });
  }

  async importRevealSecret(file: File): Promise<void> {
    await file.text();
    this.unshiftFeed({ kind: "system", message: "Reveal backup restore was simulated." });
  }

  private unshiftFeed(item: Omit<FeedItem, "id" | "timestamp">): void {
    const timestamp = Math.floor(Date.now() / 1000);
    this.snapshot.feed.unshift({
      ...item,
      id: `demo-${timestamp}-${this.snapshot.feed.length}`,
      timestamp,
    });
    this.snapshot.feed = this.snapshot.feed.slice(0, 20);
  }
}

function makePlayer(alias: string, index: number, now: number, isSelf = false): PlayerView {
  const tier = BigInt((index * 7 + 3) % 10 + 1);
  const start = tier * DUMP_TIER_SIZE;
  const balance = start * BigInt(18 + (index * 13) % 95) / 100n;
  const projected = balance * BigInt(68 + (index * 5) % 29) / 100n;
  const laneBalance = balance / 4n;
  const remainder = balance % 4n;
  const guardedLane = index % 4;
  const lanes: LaneView[] = Array.from({ length: 4 }, (_, laneIndex) => ({
    index: laneIndex,
    balance: laneBalance + (BigInt(laneIndex) < remainder ? 1n : 0n),
    guard: guardedLane === laneIndex ? start / 40n : 0n,
    lockedAmount: 0n,
    lockedUntil: 0,
    redirectArmed: guardedLane === laneIndex && index % 3 === 0,
    redirectReadyAt: 0,
    redirectedVolume: index % 3 === 0 ? start / 20n : 0n,
    cumulativeWeighted: projected * 2n * BigInt(30 * 86_400) ** 3n / 4n,
    lastCheckpointAt: now,
  }));
  return {
    address: isSelf ? SELF_ADDRESS : demoAddress(index),
    alias,
    startingAllocation: start,
    balance,
    projectedScore: projected,
    impactPpm: BigInt(70_000 + index * 41_337),
    lateImpactPpm: BigInt(index % 5 === 0 ? 120_000 : 20_000),
    meaningfulActions: 4 + (index * 11) % 57,
    distinctOpponents: 1 + (index * 3) % 12,
    dumpHeat: { units: (index * 1_031) % 8_800, updatedAt: now - index * 137 },
    absorbHeat: { units: (index * 677) % 7_200, updatedAt: now - index * 89 },
    lanes,
    revealed: true,
    allocationClaimed: true,
    settled: false,
    eligible: true,
    bondClaimed: false,
    isWinner: false,
    rewardClaimed: false,
    badges: index % 7 === 0 ? 1 << (index % 4) : 0,
    isSelf,
  };
}

function initialFeed(now: number, players: PlayerView[]): FeedItem[] {
  return [
    feed("dump", now - 18, players[3], players[7], 125_000_000n, "Debt Wizard pressure-tested Quiet Peasant's western lane."),
    feed("dump", now - 29, players[7], players[0], 210_000_000n, "Quiet Peasant sent a burden parcel to Paper Crown."),
    feed("dump", now - 41, players[9], players[2], 340_000_000n, "Reverse Whale force-fed Trash Oracle."),
    feed("redirect", now - 73, players[4], players[6], 0n, "Small Fry ricocheted a raid into Exit Liquidity."),
    feed("absorb", now - 118, players[1], players[5], 90_000_000n, "Zero Baron absorbed a burden and forged Guard."),
    feed("dump", now - 182, players[14], players[0], 510_000_000n, "Compost King launched a half-billion DUMP salvo."),
    {
      id: "demo-system",
      timestamp: now - 300,
      kind: "system",
      message: "Final-week score weighting is now 2.61× opening-week weight.",
    },
  ];
}

function feed(
  kind: "dump" | "absorb" | "redirect",
  timestamp: number,
  actor: PlayerView | undefined,
  target: PlayerView | undefined,
  amount: bigint,
  message: string,
): FeedItem {
  return {
    id: `demo-${kind}-${timestamp}`,
    timestamp,
    kind,
    actor: actor?.address,
    target: target?.address,
    amount,
    message,
  };
}

function demoAddress(index: number): string {
  const alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
  const symbol = alphabet[(index * 17 + 11) % alphabet.length] ?? "G";
  return `${symbol}Dmp${String(index + 1).padStart(2, "0")}x${symbol.repeat(35)}`;
}

function sumLanes(lanes: LaneView[]): bigint {
  return lanes.reduce((total, lane) => total + lane.balance, 0n);
}

function spendable(lane: LaneView, now: number): bigint {
  return now >= lane.lockedUntil ? lane.balance : lane.balance - lane.lockedAmount;
}

function checkpointLane(lane: LaneView, now: number, snapshot: StrategySnapshot): void {
  const { activeStartsAt, activeEndsAt } = snapshot.epoch;
  const duration = BigInt(Math.max(0, activeEndsAt - activeStartsAt));
  if (duration > 0n) {
    const checkpoint = clamp(lane.lastCheckpointAt, activeStartsAt, activeEndsAt);
    const current = Math.max(checkpoint, clamp(now, activeStartsAt, activeEndsAt));
    const from = BigInt(checkpoint - activeStartsAt);
    const to = BigInt(current - activeStartsAt);
    const area = (to - from) * duration * duration + to * to * to - from * from * from;
    lane.cumulativeWeighted += lane.balance * area;
    lane.lastCheckpointAt = current;
  } else {
    lane.lastCheckpointAt = clamp(now, activeStartsAt, activeEndsAt);
  }
  if (now >= lane.lockedUntil) lane.lockedAmount = 0n;
}

function refreshProjectedScore(
  player: PlayerView,
  now: number,
  snapshot: StrategySnapshot,
): void {
  player.projectedScore = projectedPlayerScore(
    player.lanes,
    now,
    snapshot.epoch.activeStartsAt,
    snapshot.epoch.activeEndsAt,
  );
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

function min(left: bigint, right: bigint): bigint {
  return left < right ? left : right;
}
