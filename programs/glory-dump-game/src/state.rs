use anchor_lang::prelude::*;
use crate::constants::*;

/// Global game state
#[account]
pub struct GameState {
    pub admin: Pubkey,
    pub dump_mint: Pubkey,
    pub glory_mint: Pubkey,
    pub fee_vault: Pubkey,
    pub treasury: Pubkey,
    pub current_epoch: u64,
    pub epoch_start_time: i64,
    pub next_epoch_start_time: i64,
    pub is_waiting_period: bool,
    pub is_paused: bool,
    pub total_participants: u64,
    pub total_fees_collected: u64,
    pub total_glory_distributed: u64,
    pub total_glory_minted: u64, // supply tracker
    pub bump: u8,
}

impl GameState {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        32 + // dump_mint
        32 + // glory_mint
        32 + // fee_vault
        32 + // treasury
        8 + // current_epoch
        8 + // epoch_start_time
        8 + // next_epoch_start_time
        1 + // is_waiting_period
        1 + // is_paused
        8 + // total_participants
        8 + // total_fees_collected
        8 + // total_glory_distributed
    8 + // total_glory_minted
        1; // bump
}

/// Per-epoch state and participant tracking
#[account]
pub struct EpochState {
    pub epoch_number: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub participants: Vec<Pubkey>,
    pub is_finalized: bool,
    pub total_dump_distributed: u64,
    pub total_glory_rewards: u64,
    pub winner: Option<Pubkey>,
    pub rewards_distributed: [bool; 4], // [winner, top_tier, middle_tier, bottom_tier]
    pub total_transfers: u64,
    pub total_thefts: u64,
    pub highest_volume_player: Option<Pubkey>,
    pub is_bonus_epoch: bool,
    pub bonus_multiplier: u64,
    // Merkle-based rewards claim
    pub merkle_root: [u8; 32],
    pub merkle_root_set: bool,
}

impl EpochState {
    pub const MAX_LEN: usize = 8 + // discriminator
        8 + // epoch_number
        8 + // start_time
        8 + // end_time
        4 + (32 * MAX_PARTICIPANTS_PER_EPOCH as usize) + // participants vector
        1 + // is_finalized
        8 + // total_dump_distributed
        8 + // total_glory_rewards
        1 + 32 + // winner (option + pubkey)
        4 + // rewards_distributed array
        8 + // total_transfers
        8 + // total_thefts
        1 + 32 + // highest_volume_player (option + pubkey)
        1 + // is_bonus_epoch
    8 + // bonus_multiplier
    32 + // merkle_root
    1; // merkle_root_set
}

/// Individual player state and statistics
#[account]
pub struct PlayerState {
    pub player: Pubkey,
    pub staked_amount: u64,
    pub is_active_participant: bool,
    pub last_transfer_time: i64,
    pub give_cooldown_end_time: i64,
    pub take_cooldown_end_time: i64,
    pub current_dump_balance: u64,
    pub epoch_start_balance: u64,
    pub time_weighted_sum: u128,
    pub last_update_time: i64,
    pub total_transfers_made: u64,
    pub total_thefts_made: u64,
    pub total_dump_received: u64,
    pub total_dump_given: u64,
    pub epochs_participated: u64,
    pub total_glory_earned: u64,
    pub current_epoch_signed_up: u64,
    pub join_fee_paid: u64,
}

impl PlayerState {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // staked_amount
        1 + // is_active_participant
        8 + // last_transfer_time
        8 + // give_cooldown_end_time
        8 + // take_cooldown_end_time
        8 + // current_dump_balance
        8 + // epoch_start_balance
        16 + // time_weighted_sum
        8 + // last_update_time
        8 + // total_transfers_made
        8 + // total_thefts_made
        8 + // total_dump_received
        8 + // total_dump_given
        8 + // epochs_participated
        8 + // total_glory_earned
        8 + // current_epoch_signed_up
        8; // join_fee_paid

    /// Calculate the time-weighted average DUMP balance for the current epoch
    pub fn calculate_time_weighted_average(&self, current_time: i64, epoch_start: i64) -> Result<u64> {
        let time_in_epoch = current_time.saturating_sub(epoch_start.max(self.last_update_time));
        if time_in_epoch <= 0 {
            return Ok(0);
        }
        
        let total_time_in_epoch = current_time.saturating_sub(epoch_start);
        if total_time_in_epoch <= 0 {
            return Ok(0);
        }

        let average = self.time_weighted_sum
            .checked_div(total_time_in_epoch as u128)
            .ok_or(crate::errors::GameError::MathOverflow)?;
        
        Ok(average as u64)
    }

    /// Update the time-weighted average with current balance
    pub fn update_time_weighted_average(&mut self, current_time: i64) -> Result<()> {
        let time_delta = current_time.saturating_sub(self.last_update_time);
        if time_delta > 0 {
            let balance_time_product = (self.current_dump_balance as u128)
                .checked_mul(time_delta as u128)
                .ok_or(crate::errors::GameError::MathOverflow)?;
            
            self.time_weighted_sum = self.time_weighted_sum
                .checked_add(balance_time_product)
                .ok_or(crate::errors::GameError::MathOverflow)?;
        }
        
        self.last_update_time = current_time;
        Ok(())
    }
}

/// Tracks if a player has claimed rewards for an epoch
#[account]
pub struct ClaimStatus {
    pub epoch_number: u64,
    pub player: Pubkey,
    pub claimed: bool,
}

impl ClaimStatus {
    pub const LEN: usize = 8 + // discriminator
        8 + // epoch_number
        32 + // player
        1; // claimed
}

/// Bug report for bounty system
#[account]
pub struct BugReport {
    pub reporter: Pubkey,
    pub report_id: [u8; 32],
    pub severity: BugSeverity,
    pub description: String,
    pub proof_of_concept: String,
    pub timestamp: i64,
    pub is_verified: bool,
    pub is_paid: bool,
    pub bounty_amount: u64,
    pub verifier: Option<Pubkey>,
}

impl BugReport {
    pub const MAX_LEN: usize = 8 + // discriminator
        32 + // reporter
        32 + // report_id
        1 + // severity
        4 + 256 + // description (max 256 chars)
        4 + 512 + // proof_of_concept (max 512 chars)
        8 + // timestamp
        1 + // is_verified
        1 + // is_paid
        8 + // bounty_amount
        1 + 32; // verifier (option + pubkey)
}

/// Enums
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BugSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl BugSeverity {
    pub fn get_bounty_amount(&self) -> u64 {
        match self {
            BugSeverity::Low => LOW_BOUNTY,
            BugSeverity::Medium => MEDIUM_BOUNTY,
            BugSeverity::High => HIGH_BOUNTY,
            BugSeverity::Critical => CRITICAL_BOUNTY,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RewardTier {
    Winner,     // Top 1%
    TopTier,    // Top 2-10%
    MiddleTier, // Top 11-50%
    BottomTier, // Bottom 51-100%
}

impl RewardTier {
    pub fn get_percentage(&self) -> u64 {
        match self {
            RewardTier::Winner => WINNER_PERCENTAGE,
            RewardTier::TopTier => TOP_TIER_PERCENTAGE,
            RewardTier::MiddleTier => MIDDLE_TIER_PERCENTAGE,
            RewardTier::BottomTier => BOTTOM_TIER_PERCENTAGE,
        }
    }
}
