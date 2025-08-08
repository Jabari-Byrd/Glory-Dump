use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, MintTo};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &epoch_state.epoch_number.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, winner.key().as_ref()],
        bump
    )]
    pub winner_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        seeds = [GLORY_MINT_SEED],
        bump
    )]
    pub glory_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        mut,
        associated_token::mint = glory_mint,
        associated_token::authority = winner
    )]
    pub winner_glory_account: Account<'info, TokenAccount>,

    /// CHECK: This is the winner's pubkey
    pub winner: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn distribute_rewards_handler(ctx: Context<DistributeRewards>, tier: RewardTier) -> Result<()> {
    let epoch_state = &mut ctx.accounts.epoch_state;
    let game_state = &mut ctx.accounts.game_state;

    // Check if epoch is finalized
    require!(epoch_state.is_finalized, GameError::EpochNotFinalized);

    // Only Winner tier is distributed directly; others must use Merkle claims path
    require!(matches!(tier, RewardTier::Winner), GameError::RewardTierMismatch);

    // Check if rewards for this tier haven't been distributed yet
    let tier_index = match tier {
        RewardTier::Winner => 0,
        RewardTier::TopTier => 1,
        RewardTier::MiddleTier => 2,
        RewardTier::BottomTier => 3,
    };
    require!(!epoch_state.rewards_distributed[tier_index], GameError::RewardsAlreadyDistributed);

    // Calculate reward amount based on tier
    let base_reward = 1_000_000_000; // 1000 GLORY tokens (with 9 decimals)
    let tier_percentage = tier.get_percentage();
    let reward_amount = (base_reward * tier_percentage) / 10000;

    // Apply bonus multiplier if it's a bonus epoch
    // Basic eligibility checks (non-winner tiers should use Merkle-based claim via claims.rs)
    match tier {
        RewardTier::Winner => {
            // Ensure provided winner matches recorded winner
            require!(epoch_state.winner.is_some(), GameError::NotEligibleForRewards);
            require!(epoch_state.winner.unwrap() == ctx.accounts.winner.key(), GameError::NotEligibleForRewards);
        }
        // Future: Provide Merkle/proof-based eligibility for tiers with multiple winners
        _ => {}
    }

    let final_reward = if epoch_state.is_bonus_epoch {
        (reward_amount * epoch_state.bonus_multiplier) / 100
    } else {
        reward_amount
    };

    // Enforce GLORY supply cap
    require!(
        game_state.total_glory_minted.saturating_add(final_reward) <= GLORY_SUPPLY_CAP,
        GameError::InvalidAmount
    );

    // Mint GLORY tokens to winner
    let game_state_key = game_state.key();
    let seeds = &[
        GAME_STATE_SEED,
        &[game_state.bump],
    ];
    let signer = &[&seeds[..]];

    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.glory_mint.to_account_info(),
            to: ctx.accounts.winner_glory_account.to_account_info(),
            authority: game_state.to_account_info(),
        },
        signer,
    );
    token::mint_to(mint_ctx, final_reward)?;

    // Update states
    epoch_state.rewards_distributed[tier_index] = true;
    epoch_state.total_glory_rewards += final_reward;
    ctx.accounts.winner_state.total_glory_earned += final_reward;
    game_state.total_glory_minted = game_state.total_glory_minted.saturating_add(final_reward);

    msg!("Distributed {} GLORY tokens to {:?} tier winner: {}",
         final_reward,
         tier,
         ctx.accounts.winner.key());

    Ok(())
}
