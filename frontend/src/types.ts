export type Phase =
  | "registration"
  | "reveal"
  | "planning"
  | "active"
  | "settling"
  | "complete"
  | "cancelled";

export type ActionKind = "dump" | "absorb" | "redirect";

export interface HeatView {
  units: number;
  updatedAt: number;
}

export interface LaneView {
  index: number;
  balance: bigint;
  guard: bigint;
  lockedAmount: bigint;
  lockedUntil: number;
  redirectArmed: boolean;
  redirectReadyAt: number;
  redirectedVolume: bigint;
  cumulativeWeighted: bigint;
  lastCheckpointAt: number;
}

export interface PlayerView {
  address: string;
  alias: string;
  startingAllocation: bigint;
  balance: bigint;
  projectedScore: bigint;
  impactPpm: bigint;
  lateImpactPpm: bigint;
  meaningfulActions: number;
  distinctOpponents: number;
  dumpHeat: HeatView;
  absorbHeat: HeatView;
  lanes: LaneView[];
  revealed: boolean;
  allocationClaimed: boolean;
  settled: boolean;
  eligible: boolean;
  bondClaimed: boolean;
  isWinner: boolean;
  rewardClaimed: boolean;
  badges: number;
  isSelf?: boolean;
}

export interface EpochView {
  number: bigint;
  phase: Phase;
  phaseEndsAt: number;
  activeStartsAt: number;
  activeEndsAt: number;
  bondClaimEndsAt: number;
  participantCount: number;
  revealedCount: number;
  settledCount: number;
  eligibleCount: number;
  winnerCount: number;
  playerRewardPool: bigint;
  keeperRewardPool: bigint;
  worstPlayer: string;
}

export interface EpochClaimView {
  epoch: bigint;
  phase: Phase;
  isCurrent: boolean;
  registered: boolean;
  revealed: boolean;
  eligible: boolean;
  settled: boolean;
  isWinner: boolean;
  rank?: number;
  rewardAmount: bigint;
  rewardClaimed: boolean;
  bondRefund: bigint;
  bondClaimed: boolean;
  bondClaimEndsAt: number;
  keeperPlayersSettled: number;
  keeperRewardAmount: bigint;
  keeperClaimed: boolean;
  badges: number;
  worstPlayer: string;
  claimWindowOpen: boolean;
}

export interface FeedItem {
  id: string;
  timestamp: number;
  kind: ActionKind | "system" | "reward";
  actor?: string;
  target?: string;
  amount?: bigint;
  message: string;
}

export interface ScoreForecastPoint {
  timestamp: number;
  score: bigint;
}

export interface ThreatView {
  address: string;
  incomingDump: bigint;
  outgoingDump: bigint;
  incomingActions: number;
  outgoingActions: number;
}

export interface StrategySnapshot {
  mode: "demo" | "live";
  epoch: EpochView;
  players: PlayerView[];
  claims: EpochClaimView[];
  feed: FeedItem[];
  walletAddress?: string;
  gloryBalance: bigint;
  clusterLabel: string;
  lastSyncedAt: number;
}

export interface ActionRequest {
  kind: ActionKind;
  target?: string;
  amount?: bigint;
  actorLane?: number;
  targetLane?: number;
}

export interface GameGateway {
  connect(): Promise<StrategySnapshot>;
  refresh(): Promise<StrategySnapshot>;
  act(request: ActionRequest): Promise<string>;
  register(): Promise<string>;
  reveal(): Promise<string>;
  claimAllocation(): Promise<string>;
  advanceEpoch(): Promise<string>;
  settleNextPlayer(): Promise<string>;
  claimPlayerReward(epoch: bigint): Promise<string>;
  claimBond(epoch: bigint): Promise<string>;
  refreshBadges(epoch: bigint): Promise<string>;
  claimKeeperReward(epoch: bigint): Promise<string>;
  closePlayerAccounts(epoch: bigint): Promise<string>;
  closeKeeperCredit(epoch: bigint): Promise<string>;
  exportRevealSecret(): Promise<void>;
  importRevealSecret(file: File): Promise<void>;
}
