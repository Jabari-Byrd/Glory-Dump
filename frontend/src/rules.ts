import type { HeatView, PlayerView } from "./types";

export const DUMP_TIER_SIZE = 1_000_000_000n;
export const LANE_COUNT = 4;
export const HEAT_CAP = 10_000;
export const HEAT_RECOVERY_SECONDS = 6 * 60 * 60;
export const MIN_ACTION_BPS = 10n;
export const BPS_DENOMINATOR = 10_000n;
export const GLORY_SCALE = 1_000_000n;

export const BADGES = [
  { bit: 1 << 0, label: "Escape Artist", icon: "↘" },
  { bit: 1 << 1, label: "Human Shield", icon: "⬡" },
  { bit: 1 << 2, label: "Ricochet", icon: "↩" },
  { bit: 1 << 3, label: "Last Laugh", icon: "◴" },
  { bit: 1 << 4, label: "Garbage Emperor", icon: "♛" },
] as const;

export function minimumAction(startingAllocation: bigint): bigint {
  return divCeil(startingAllocation * MIN_ACTION_BPS, BPS_DENOMINATOR);
}

export function heatAt(heat: HeatView, now: number): number {
  if (now <= heat.updatedAt || heat.units === 0) return heat.units;
  const elapsed = Math.min(now - heat.updatedAt, HEAT_RECOVERY_SECONDS);
  const decay = Math.floor((elapsed * HEAT_CAP) / HEAT_RECOVERY_SECONDS);
  return Math.max(0, heat.units - decay);
}

export function heatCost(amount: bigint, startingAllocation: bigint): number {
  if (startingAllocation <= 0n) return HEAT_CAP + 1;
  return Number(divCeil(amount * BigInt(HEAT_CAP), startingAllocation));
}

export function secondsUntilHeatAvailable(
  heat: HeatView,
  now: number,
  amount: bigint,
  startingAllocation: bigint,
): number {
  const cost = heatCost(amount, startingAllocation);
  if (cost > HEAT_CAP) return Number.POSITIVE_INFINITY;
  const needed = Math.max(0, heatAt(heat, now) + cost - HEAT_CAP);
  return Math.ceil((needed * HEAT_RECOVERY_SECONDS) / HEAT_CAP);
}

export function projectedWeightedScore(
  cumulativeWeighted: bigint,
  balance: bigint,
  lastCheckpointAt: number,
  now: number,
  epochStart: number,
  epochEnd: number,
): bigint {
  const duration = BigInt(Math.max(0, epochEnd - epochStart));
  if (duration === 0n) return 0n;
  const from = BigInt(clamp(lastCheckpointAt, epochStart, epochEnd) - epochStart);
  const to = BigInt(clamp(now, epochStart, epochEnd) - epochStart);
  const area =
    (to - from) * duration * duration + to * to * to - from * from * from;
  return (cumulativeWeighted + balance * area) / (2n * duration * duration * duration);
}

export function rankPlayers(players: PlayerView[]): PlayerView[] {
  return [...players].sort((left, right) => {
    if (left.projectedScore !== right.projectedScore) {
      return left.projectedScore < right.projectedScore ? -1 : 1;
    }
    if (left.impactPpm !== right.impactPpm) {
      return left.impactPpm > right.impactPpm ? -1 : 1;
    }
    if (left.distinctOpponents !== right.distinctOpponents) {
      return right.distinctOpponents - left.distinctOpponents;
    }
    return left.address.localeCompare(right.address);
  });
}

export function formatDump(value: bigint, compact = false): string {
  const sign = value < 0n ? "−" : "";
  const absolute = value < 0n ? -value : value;
  if (absolute >= DUMP_TIER_SIZE) {
    return `${sign}${formatScaled(absolute, DUMP_TIER_SIZE, compact ? 1 : 2)}B`;
  }
  if (absolute >= 1_000_000n) {
    return `${sign}${formatScaled(absolute, 1_000_000n, compact ? 0 : 1)}M`;
  }
  return `${sign}${absolute.toLocaleString("en-US")}`;
}

export function formatGlory(value: bigint): string {
  return formatScaled(value, GLORY_SCALE, 2);
}

export function parseDumpInput(raw: string): bigint | null {
  const normalized = raw.trim().replaceAll(",", "").toLowerCase();
  const match = /^(\d+(?:\.\d{0,6})?)\s*([bmk])?$/.exec(normalized);
  if (!match?.[1]) return null;
  const multiplier = match[2] === "b"
    ? 1_000_000_000n
    : match[2] === "m"
      ? 1_000_000n
      : match[2] === "k"
        ? 1_000n
        : 1n;
  const [whole = "0", fraction = ""] = match[1].split(".");
  const scale = 10n ** BigInt(fraction.length);
  return (BigInt(whole) * scale + BigInt(fraction || "0")) * multiplier / scale;
}

export function formatCountdown(target: number, now: number): string {
  const remaining = Math.max(0, target - now);
  const days = Math.floor(remaining / 86_400);
  const hours = Math.floor((remaining % 86_400) / 3_600);
  const minutes = Math.floor((remaining % 3_600) / 60);
  if (days > 0) return `${days}d ${hours}h ${minutes}m`;
  return `${hours}h ${minutes}m`;
}

export function shortAddress(address: string, width = 4): string {
  return address.length <= width * 2 + 3
    ? address
    : `${address.slice(0, width)}…${address.slice(-width)}`;
}

export function badgeList(mask: number): (typeof BADGES)[number][] {
  return BADGES.filter((badge) => (mask & badge.bit) !== 0);
}

export async function commitmentFor(
  secret: Uint8Array,
  owner: Uint8Array,
  epoch: bigint,
): Promise<Uint8Array> {
  if (secret.length !== 32 || owner.length !== 32) {
    throw new Error("Commitment secrets and wallet keys must be 32 bytes");
  }
  const input = concatenate(
    new TextEncoder().encode("glory-dump-commitment-v3"),
    secret,
    owner,
    u64LittleEndian(epoch),
  );
  return new Uint8Array(
    await crypto.subtle.digest("SHA-256", input.buffer as ArrayBuffer),
  );
}

export function bytesToHex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

export function territoryLevel(player: PlayerView): number {
  if (player.startingAllocation === 0n) return 0;
  const ratio = Number(
    (player.projectedScore * 100n) / player.startingAllocation,
  );
  return Math.max(0, Math.min(4, 4 - Math.floor(ratio / 25)));
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

function u64LittleEndian(value: bigint): Uint8Array {
  if (value < 0n || value > 0xffff_ffff_ffff_ffffn) {
    throw new Error("Epoch must fit in an unsigned 64-bit integer");
  }
  const bytes = new Uint8Array(8);
  let remaining = value;
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Number(remaining & 0xffn);
    remaining >>= 8n;
  }
  return bytes;
}

function formatScaled(value: bigint, scale: bigint, decimals: number): string {
  const whole = value / scale;
  if (decimals === 0) return whole.toLocaleString("en-US");
  const fraction = ((value % scale) * 10n ** BigInt(decimals) / scale)
    .toString()
    .padStart(decimals, "0")
    .replace(/0+$/, "");
  return fraction
    ? `${whole.toLocaleString("en-US")}.${fraction}`
    : whole.toLocaleString("en-US");
}

function divCeil(value: bigint, divisor: bigint): bigint {
  return (value + divisor - 1n) / divisor;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}
