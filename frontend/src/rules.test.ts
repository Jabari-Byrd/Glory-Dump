import { describe, expect, it } from "vitest";

import {
  DUMP_TIER_SIZE,
  bytesToHex,
  commitmentFor,
  formatDump,
  heatAt,
  heatCost,
  minimumAction,
  parseDumpInput,
  projectedWeightedScore,
} from "./rules";

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
});
