import { describe, expect, it } from "vitest";

import {
  DUMP_TIER_SIZE,
  bondRefundLamports,
  bytesToHex,
  canClosePlayerAccounts,
  commitmentFor,
  formatDump,
  heatAt,
  heatClearAt,
  heatCost,
  isGarbageEmperor,
  keeperReward,
  minimumAction,
  noActionScoreForecast,
  parseDumpInput,
  projectedPlayerScore,
  projectedWeightedScore,
  rankReward,
  scoreDeltaForBalanceChange,
  threatViews,
} from "./rules";
import type { EpochClaimView, EpochView, PlayerView } from "./types";

describe("client rule parity", () => {
  it("matches the Rust commitment test vector", async () => {
    const commitment = await commitmentFor(
      new Uint8Array(32).fill(7),
      new Uint8Array(32).fill(9),
      42n,
    );
    expect(bytesToHex(commitment)).toBe(
      "0e12da7971e942fb5777ca6d16dbcef78d9cfccf08f5a0fd4d4c201549cd7aa9",
    );
  });

  it("keeps a constant balance's weighted score exact", () => {
    const start = 1_000;
    const duration = 30 * 24 * 60 * 60;
    expect(
      projectedWeightedScore(0n, DUMP_TIER_SIZE, start, start + duration, start, start + duration),
    ).toBe(DUMP_TIER_SIZE);
  });

  it("does not let a final-one-percent dump erase the epoch", () => {
    const start = 1_000;
    const duration = 30 * 24 * 60 * 60;
    const heldUntil = start + Math.floor(duration * 0.99);
    const score = projectedWeightedScore(
      0n,
      DUMP_TIER_SIZE,
      start,
      heldUntil,
      start,
      start + duration,
    );
    expect(score).toBeGreaterThanOrEqual(980_000_000n);
    expect(score).toBeLessThanOrEqual(981_000_000n);
  });

  it("keeps heat proportional and split-invariant", () => {
    const start = DUMP_TIER_SIZE;
    expect(heatCost(start / 10n, start)).toBe(
      10 * heatCost(start / 100n, start),
    );
    expect(heatAt({ units: 5_000, updatedAt: 0 }, 3 * 60 * 60)).toBe(0);
  });

  it("parses human DUMP notation without floating point", () => {
    expect(parseDumpInput("1.25B")).toBe(1_250_000_000n);
    expect(parseDumpInput("250m")).toBe(250_000_000n);
    expect(parseDumpInput("garbage")).toBeNull();
    expect(formatDump(1_250_000_000n)).toBe("1.25B");
    expect(minimumAction(DUMP_TIER_SIZE)).toBe(1_000_000n);
  });

  it("forecasts the no-action score from exact lane accumulators", () => {
    const start = 1_000;
    const end = start + 30 * 24 * 60 * 60;
    const player = forecastPlayer(start);
    const epoch: EpochView = {
      number: 1n,
      phase: "active",
      phaseEndsAt: end,
      activeStartsAt: start,
      activeEndsAt: end,
      bondClaimEndsAt: 0,
      participantCount: 1,
      revealedCount: 1,
      settledCount: 0,
      eligibleCount: 1,
      winnerCount: 1,
      playerRewardPool: 0n,
      keeperRewardPool: 0n,
      worstPlayer: "",
    };
    const forecast = noActionScoreForecast(player, epoch, start, 5);
    expect(forecast).toHaveLength(5);
    expect(forecast.map((point) => point.score)).toEqual(
      [...forecast].map((point) => point.score).sort((left, right) => left < right ? -1 : 1),
    );
    expect(forecast.at(-1)?.score).toBe(DUMP_TIER_SIZE);
    expect(projectedPlayerScore(player.lanes, end, start, end)).toBe(DUMP_TIER_SIZE);
  });

  it("does not project a lane backward when the chain clock is ahead", () => {
    const start = 1_000;
    const end = start + 30 * 24 * 60 * 60;
    const checkpoint = start + 10;
    const cumulative = 123_456_789n;
    expect(projectedWeightedScore(
      cumulative,
      DUMP_TIER_SIZE,
      checkpoint,
      start,
      start,
      end,
    )).toBe(projectedWeightedScore(
      cumulative,
      DUMP_TIER_SIZE,
      checkpoint,
      checkpoint,
      start,
      end,
    ));
  });

  it("shows the score consequence of burden changes at the action time", () => {
    const start = 1_000;
    const end = start + 30 * 24 * 60 * 60;
    const now = start + 15 * 24 * 60 * 60;
    const added = scoreDeltaForBalanceChange(100_000_000n, now, start, end);
    const removed = scoreDeltaForBalanceChange(-100_000_000n, now, start, end);
    expect(added).toBeGreaterThan(0n);
    expect(removed).toBe(-added);
    expect(scoreDeltaForBalanceChange(100_000_000n, end, start, end)).toBe(0n);
  });

  it("aggregates observed DUMP rivalries without inventing live history", () => {
    const threats = threatViews([
      { id: "1", timestamp: 1, kind: "dump", actor: "rival", target: "self", amount: 10n, message: "" },
      { id: "2", timestamp: 2, kind: "dump", actor: "self", target: "rival", amount: 4n, message: "" },
      { id: "3", timestamp: 3, kind: "absorb", actor: "rival", target: "self", amount: 99n, message: "" },
    ], "self");
    expect(threats).toEqual([{
      address: "rival",
      incomingDump: 10n,
      outgoingDump: 4n,
      incomingActions: 1,
      outgoingActions: 1,
    }]);
  });

  it("computes when existing Heat is fully clear", () => {
    expect(heatClearAt({ units: 5_000, updatedAt: 1_000 }, 1_000)).toBe(
      1_000 + 3 * 60 * 60,
    );
    expect(heatClearAt({ units: 5_000, updatedAt: 1_000 }, 1_001)).toBe(
      1_000 + 3 * 60 * 60,
    );
    expect(heatClearAt({ units: 5_000, updatedAt: 0 }, 3 * 60 * 60)).toBe(3 * 60 * 60);
  });

  it("mirrors the on-chain rank reward split", () => {
    const pool = 99_000_003n;
    for (const winners of [1, 2, 5, 20, 128]) {
      let distributed = 0n;
      for (let rank = 0; rank < winners; rank += 1) {
        distributed += rankReward(pool, winners, rank);
      }
      expect(distributed).toBe(pool);
    }
  });

  it("derives Garbage Emperor from the finalized worst player", () => {
    const epoch: EpochView = {
      number: 1n,
      phase: "complete",
      phaseEndsAt: 0,
      activeStartsAt: 0,
      activeEndsAt: 0,
      bondClaimEndsAt: 0,
      participantCount: 2,
      revealedCount: 2,
      settledCount: 2,
      eligibleCount: 2,
      winnerCount: 1,
      playerRewardPool: 0n,
      keeperRewardPool: 0n,
      worstPlayer: "worst",
    };
    expect(isGarbageEmperor("worst", epoch)).toBe(true);
    expect(isGarbageEmperor("other", epoch)).toBe(false);
    expect(isGarbageEmperor("worst", { ...epoch, phase: "active" })).toBe(false);
  });

  it("matches bond refund and keeper reward accounting", () => {
    expect(bondRefundLamports("cancelled", false)).toBe(2_000_000n);
    expect(bondRefundLamports("complete", true)).toBe(1_600_000n);
    expect(bondRefundLamports("complete", false)).toBe(0n);
    expect(bondRefundLamports("active", true)).toBe(0n);
    expect(keeperReward(1_000_000n, 3, 20)).toBe(150_000n);
    expect(keeperReward(1_000_000n, 3, 0)).toBe(0n);
  });

  it("only exposes rent recovery after claims and windows are safe", () => {
    const base: EpochClaimView = {
      epoch: 1n,
      phase: "complete",
      isCurrent: false,
      registered: true,
      revealed: true,
      eligible: true,
      settled: true,
      isWinner: true,
      rank: 1,
      rewardAmount: 1_000n,
      rewardClaimed: false,
      bondRefund: 1_600_000n,
      bondClaimed: false,
      bondClaimEndsAt: 100,
      keeperPlayersSettled: 0,
      keeperRewardAmount: 0n,
      keeperClaimed: false,
      badges: 0,
      worstPlayer: "someone",
      claimWindowOpen: true,
    };
    expect(canClosePlayerAccounts(base)).toBe(false);
    expect(canClosePlayerAccounts({ ...base, rewardClaimed: true })).toBe(false);
    expect(canClosePlayerAccounts({
      ...base,
      rewardClaimed: true,
      bondClaimed: true,
    })).toBe(true);
    expect(canClosePlayerAccounts({
      ...base,
      rewardClaimed: true,
      bondClaimed: true,
      claimWindowOpen: false,
    })).toBe(true);
    expect(canClosePlayerAccounts({
      ...base,
      phase: "cancelled",
      eligible: false,
      bondRefund: 2_000_000n,
      bondClaimed: true,
      isWinner: false,
    })).toBe(true);
  });
});

function forecastPlayer(start: number): PlayerView {
  return {
    address: "self",
    alias: "Self",
    startingAllocation: DUMP_TIER_SIZE,
    balance: DUMP_TIER_SIZE,
    projectedScore: 0n,
    impactPpm: 0n,
    lateImpactPpm: 0n,
    meaningfulActions: 1,
    distinctOpponents: 1,
    dumpHeat: { units: 0, updatedAt: start },
    absorbHeat: { units: 0, updatedAt: start },
    lanes: Array.from({ length: 4 }, (_, index) => ({
      index,
      balance: DUMP_TIER_SIZE / 4n,
      guard: 0n,
      lockedAmount: 0n,
      lockedUntil: 0,
      redirectArmed: false,
      redirectReadyAt: 0,
      redirectedVolume: 0n,
      cumulativeWeighted: 0n,
      lastCheckpointAt: start,
    })),
    revealed: true,
    allocationClaimed: true,
    settled: false,
    eligible: true,
    bondClaimed: false,
    isWinner: false,
    rewardClaimed: false,
    badges: 0,
    isSelf: true,
  };
}
