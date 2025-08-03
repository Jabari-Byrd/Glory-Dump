use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("GDgame1111111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use constants::*;
use errors::*;
use instructions::*;
use state::*;

#[program]
pub mod glory_dump_game {
    use super::*;

    /// Initialize the game with DUMP and GLORY token mints
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        bump: u8,
    ) -> Result<()> {
        instructions::initialize_game::handler(ctx, bump)
    }

    /// Initialize a new epoch
    pub fn initialize_epoch(ctx: Context<InitializeEpoch>) -> Result<()> {
        instructions::epoch::initialize_epoch_handler(ctx)
    }

    /// Sign up for the next epoch during waiting period
    pub fn sign_up_for_epoch(
        ctx: Context<SignUpForEpoch>,
        payment_amount: u64,
    ) -> Result<()> {
        instructions::epoch::sign_up_handler(ctx, payment_amount)
    }

    /// Start a new epoch and distribute random DUMP amounts
    pub fn start_epoch(ctx: Context<StartEpoch>) -> Result<()> {
        instructions::epoch::start_epoch_handler(ctx)
    }

    /// Finalize an epoch and calculate rankings
    pub fn finalize_epoch(ctx: Context<FinalizeEpoch>) -> Result<()> {
        instructions::epoch::finalize_epoch_handler(ctx)
    }

    /// Distribute GLORY rewards to epoch winners
    pub fn distribute_rewards(
        ctx: Context<DistributeRewards>,
        tier: RewardTier,
    ) -> Result<()> {
        instructions::rewards::distribute_rewards_handler(ctx, tier)
    }

    /// Stake DUMP tokens to become an active participant
    pub fn stake_for_participation(
        ctx: Context<StakeForParticipation>,
        amount: u64,
    ) -> Result<()> {
        instructions::participation::stake_handler(ctx, amount)
    }

    /// Withdraw stake (only when not in active epoch)
    pub fn withdraw_stake(ctx: Context<WithdrawStake>) -> Result<()> {
        instructions::participation::withdraw_stake_handler(ctx)
    }

    /// Transfer DUMP to another player (with cooldown)
    pub fn transfer_dump(
        ctx: Context<TransferDump>,
        amount: u64,
    ) -> Result<()> {
        instructions::transfer::transfer_dump_handler(ctx, amount)
    }

    /// Steal DUMP from another player (with cooldown)
    pub fn steal_dump(
        ctx: Context<StealDump>,
        amount: u64,
    ) -> Result<()> {
        instructions::transfer::steal_dump_handler(ctx, amount)
    }

    /// Update a player's time-weighted average DUMP balance
    pub fn update_player_average(ctx: Context<UpdatePlayerAverage>) -> Result<()> {
        instructions::tracking::update_average_handler(ctx)
    }

    /// Submit a bug report for bounty consideration
    pub fn submit_bug_report(
        ctx: Context<SubmitBugReport>,
        description: String,
        proof_of_concept: String,
        severity: BugSeverity,
    ) -> Result<()> {
        instructions::bug_bounty::submit_report_handler(ctx, description, proof_of_concept, severity)
    }

    /// Verify and pay bug bounty (admin only)
    pub fn verify_bug_report(
        ctx: Context<VerifyBugReport>,
        report_id: [u8; 32],
        is_valid: bool,
    ) -> Result<()> {
        instructions::bug_bounty::verify_report_handler(ctx, report_id, is_valid)
    }

    /// Emergency pause (admin only)
    pub fn emergency_pause(ctx: Context<EmergencyAction>) -> Result<()> {
        instructions::admin::emergency_pause_handler(ctx)
    }

    /// Emergency unpause (admin only)
    pub fn emergency_unpause(ctx: Context<EmergencyAction>) -> Result<()> {
        instructions::admin::emergency_unpause_handler(ctx)
    }
}
