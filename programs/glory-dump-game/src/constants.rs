use anchor_lang::prelude::*;

// Game timing constants
pub const EPOCH_DURATION: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
pub const WAITING_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days in seconds

// Fee constants (in basis points)
pub const TRANSFER_FEE_BASIS_POINTS: u64 = 30; // 0.3%
pub const THEFT_FEE_BASIS_POINTS: u64 = 30; // 0.3%

// Staking constants
pub const SYBIL_STAKE_PERCENTAGE: u64 = 5; // 0.05% of total supply required to stake
pub const MIN_STAKE_AMOUNT: u64 = 1_000_000; // 1 DUMP minimum stake

// Cooldown constants (in seconds)
pub const THEFT_COOLDOWN_MIN: i64 = 30; // 30 seconds minimum
pub const THEFT_COOLDOWN_MAX: i64 = 3600; // 1 hour maximum
pub const TRANSFER_COOLDOWN_MIN: i64 = 15; // 15 seconds minimum
pub const TRANSFER_COOLDOWN_MAX: i64 = 1800; // 30 minutes maximum

// Random DUMP distribution constants
pub const MIN_DUMP_ASSIGNMENT: u64 = 1_000_000; // 1 DUMP minimum
pub const MAX_DUMP_ASSIGNMENT: u64 = 10_000_000_000; // 10 billion DUMP maximum

// Join fee constants (in lamports)
pub const BASE_JOIN_FEE: u64 = 10_000_000; // 0.01 SOL
pub const MAX_JOIN_FEE: u64 = 1_000_000_000; // 1 SOL

// Reward distribution percentages (in basis points)
pub const WINNER_PERCENTAGE: u64 = 4000; // 40%
pub const TOP_TIER_PERCENTAGE: u64 = 4000; // 40%
pub const MIDDLE_TIER_PERCENTAGE: u64 = 2000; // 20%
pub const BOTTOM_TIER_PERCENTAGE: u64 = 500; // 5%

// Bug bounty amounts (in GLORY tokens with 9 decimals)
pub const CRITICAL_BOUNTY: u64 = 100_000_000_000_000; // 100K GLORY
pub const HIGH_BOUNTY: u64 = 50_000_000_000_000; // 50K GLORY
pub const MEDIUM_BOUNTY: u64 = 25_000_000_000_000; // 25K GLORY
pub const LOW_BOUNTY: u64 = 10_000_000_000_000; // 10K GLORY

// PDA seeds
pub const GAME_STATE_SEED: &[u8] = b"game_state";
pub const EPOCH_STATE_SEED: &[u8] = b"epoch_state";
pub const PLAYER_STATE_SEED: &[u8] = b"player_state";
pub const BUG_REPORT_SEED: &[u8] = b"bug_report";
pub const DUMP_MINT_SEED: &[u8] = b"dump_mint";
pub const GLORY_MINT_SEED: &[u8] = b"glory_mint";
pub const FEE_VAULT_SEED: &[u8] = b"fee_vault";
pub const TREASURY_SEED: &[u8] = b"treasury";

// Token decimals
pub const DUMP_DECIMALS: u8 = 6;
pub const GLORY_DECIMALS: u8 = 9;

// Maximum values to prevent overflow
pub const MAX_VIRTUAL_SIZE: u64 = 1_000_000;
pub const MAX_PARTICIPANTS_PER_EPOCH: u16 = 10_000;
