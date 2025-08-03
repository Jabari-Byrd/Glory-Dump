use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::constants::*;
use crate::errors::*;

// Stake for Participation
#[derive(Accounts)]
pub struct StakeForParticipation<'info> {
    #[account(
        init_if_needed,
        payer = player,
        space = PlayerState::LEN,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        associated_token::mint = dump_mint,
        associated_token::authority = player
    )]
    pub player_dump_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [DUMP_MINT_SEED],
        bump
    )]
    pub dump_mint: Account<'info, anchor_spl::token::Mint>,

    /// Vault to hold staked DUMP tokens
    #[account(
        init_if_needed,
        payer = player,
        token::mint = dump_mint,
        token::authority = game_state,
        seeds = [b"stake_vault", player.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn stake_handler(ctx: Context<StakeForParticipation>, amount: u64) -> Result<()> {
    let player_state = &mut ctx.accounts.player_state;
    let game_state = &ctx.accounts.game_state;
    let clock = Clock::get()?;

    // Check minimum stake requirement
    require!(amount >= MIN_STAKE_AMOUNT, GameError::InsufficientStake);

    // Check if already an active participant
    require!(!player_state.is_active_participant, GameError::AlreadyActiveParticipant);

    // Check if player has enough DUMP tokens
    require!(
        ctx.accounts.player_dump_account.amount >= amount,
        GameError::InsufficientBalance
    );

    // Transfer DUMP tokens to stake vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.player_dump_account.to_account_info(),
            to: ctx.accounts.stake_vault.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Initialize player state if needed
    if player_state.player == Pubkey::default() {
        player_state.player = ctx.accounts.player.key();
        player_state.staked_amount = 0;
        player_state.is_active_participant = false;
        player_state.last_transfer_time = 0;
        player_state.give_cooldown_end_time = 0;
        player_state.take_cooldown_end_time = 0;
        player_state.current_dump_balance = 0;
        player_state.epoch_start_balance = 0;
        player_state.time_weighted_sum = 0;
        player_state.last_update_time = clock.unix_timestamp;
        player_state.total_transfers_made = 0;
        player_state.total_thefts_made = 0;
        player_state.total_dump_received = 0;
        player_state.total_dump_given = 0;
        player_state.epochs_participated = 0;
        player_state.total_glory_earned = 0;
        player_state.current_epoch_signed_up = 0;
        player_state.join_fee_paid = 0;
    }

    // Update player state
    player_state.staked_amount += amount;
    player_state.is_active_participant = true;

    msg!("Player {} staked {} DUMP tokens and is now an active participant", 
         ctx.accounts.player.key(), 
         amount);

    Ok(())
}

// Withdraw Stake
#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        associated_token::mint = dump_mint,
        associated_token::authority = player
    )]
    pub player_dump_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [DUMP_MINT_SEED],
        bump
    )]
    pub dump_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        mut,
        token::mint = dump_mint,
        token::authority = game_state,
        seeds = [b"stake_vault", player.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn withdraw_stake_handler(ctx: Context<WithdrawStake>) -> Result<()> {
    let player_state = &mut ctx.accounts.player_state;
    let game_state = &ctx.accounts.game_state;

    // Check if player is active participant
    require!(player_state.is_active_participant, GameError::NotActiveParticipant);

    // Check if not in active epoch (can only withdraw during waiting period)
    require!(game_state.is_waiting_period, GameError::CannotWithdrawDuringEpoch);

    let stake_amount = player_state.staked_amount;
    require!(stake_amount > 0, GameError::InsufficientBalance);

    // Transfer staked DUMP back to player
    let game_state_key = game_state.key();
    let seeds = &[
        GAME_STATE_SEED,
        &[game_state.bump],
    ];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.player_dump_account.to_account_info(),
            authority: game_state.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, stake_amount)?;

    // Update player state
    player_state.staked_amount = 0;
    player_state.is_active_participant = false;

    msg!("Player {} withdrew {} DUMP tokens and is no longer an active participant", 
         ctx.accounts.player.key(), 
         stake_amount);

    Ok(())
}
