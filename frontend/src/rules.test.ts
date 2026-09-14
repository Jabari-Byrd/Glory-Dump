import { describe, expect, it } from "vitest";

import {
  DUMP_TIER_SIZE,
  bytesToHex,
  commitmentFor,
  formatDump,
  heatAt,
  heatClearAt,
  heatCost,
  minimumAction,
  noActionScoreForecast,
  parseDumpInput,
  projectedPlayerScore,
  projectedWeightedScore,
  scoreDeltaForBalanceChange,
  threatViews,
} from "./rules";
import type { EpochView, PlayerView } from "./types";

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
      participantCount: 1,
      revealedCount: 1,
      settledCount: 0,
      eligibleCount: 1,
      winnerCount: 1,
      playerRewardPool: 0n,
      keeperRewardPool: 0n,
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
