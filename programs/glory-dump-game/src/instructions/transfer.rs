use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;

// Transfer DUMP (Give/Dump)
#[derive(Accounts)]
pub struct TransferDump<'info> {
    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, sender.key().as_ref()],
        bump
    )]
    pub sender_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, recipient_key.key().as_ref()],
        bump
    )]
    pub recipient_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &game_state.current_epoch.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(
        mut,
        associated_token::mint = dump_mint,
        associated_token::authority = sender
    )]
    pub sender_dump_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = dump_mint,
        associated_token::authority = recipient_key
    )]
    pub recipient_dump_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [FEE_VAULT_SEED],
        bump
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [DUMP_MINT_SEED],
        bump
    )]
    pub dump_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(mut)]
    pub sender: Signer<'info>,

    /// CHECK: This is the recipient's pubkey, verified by the recipient_dump_account constraint
    pub recipient_key: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn transfer_dump_handler(ctx: Context<TransferDump>, amount: u64) -> Result<()> {
    let sender_state = &mut ctx.accounts.sender_state;
    let recipient_state = &mut ctx.accounts.recipient_state;
    let game_state = &mut ctx.accounts.game_state;
    let epoch_state = &mut ctx.accounts.epoch_state;
    let clock = Clock::get()?;

    // Basic validations
    require!(!game_state.is_paused, GameError::GamePaused);
    require!(!game_state.is_waiting_period, GameError::EpochNotStarted);
    require!(amount > 0, GameError::InvalidAmount);
    require!(
        ctx.accounts.sender.key() != ctx.accounts.recipient_key.key(),
        GameError::SelfTransfer
    );

    // Check if both players are active participants
    require!(sender_state.is_active_participant, GameError::NotActiveParticipant);
    require!(recipient_state.is_active_participant, GameError::NotActiveParticipant);

    // Check cooldown
    require!(
        clock.unix_timestamp >= sender_state.give_cooldown_end_time,
        GameError::PlayerInCooldown
    );

    // Check if sender has enough DUMP
    require!(
        ctx.accounts.sender_dump_account.amount >= amount,
        GameError::InsufficientBalance
    );

    // Calculate fee
    let fee_amount = (amount * TRANSFER_FEE_BASIS_POINTS) / 10000;
    let transfer_amount = amount - fee_amount;

    // Update time-weighted averages before balance changes
    sender_state.update_time_weighted_average(clock.unix_timestamp)?;
    recipient_state.update_time_weighted_average(clock.unix_timestamp)?;

    // Transfer DUMP from sender to recipient
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.sender_dump_account.to_account_info(),
            to: ctx.accounts.recipient_dump_account.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, transfer_amount)?;

    // Transfer fee to fee vault
    if fee_amount > 0 {
        let fee_transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.sender_dump_account.to_account_info(),
                to: ctx.accounts.fee_vault.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            },
        );
        token::transfer(fee_transfer_ctx, fee_amount)?;
        game_state.total_fees_collected += fee_amount;
    }

    // Calculate cooldown based on amount transferred
    let cooldown_duration = calculate_transfer_cooldown(amount);

    // Update sender state (use token account balance for consistency)
    sender_state.current_dump_balance = ctx.accounts.sender_dump_account.amount.saturating_sub(0); // reflects post-fee transfer
    sender_state.total_dump_given += amount;
    sender_state.total_transfers_made += 1;
    sender_state.last_transfer_time = clock.unix_timestamp;
    sender_state.give_cooldown_end_time = clock.unix_timestamp + cooldown_duration;

    // Update recipient state (use token account balance)
    recipient_state.current_dump_balance = ctx.accounts.recipient_dump_account.amount;
    recipient_state.total_dump_received += transfer_amount;

    // Update epoch statistics
    epoch_state.total_transfers += 1;

    msg!("Transfer: {} DUMP from {} to {} (fee: {} DUMP, cooldown: {}s)",
         transfer_amount,
         ctx.accounts.sender.key(),
         ctx.accounts.recipient_key.key(),
         fee_amount,
         cooldown_duration);

    Ok(())
}

// Steal DUMP (Theft)
#[derive(Accounts)]
pub struct StealDump<'info> {
    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, thief.key().as_ref()],
        bump
    )]
    pub thief_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, victim_key.key().as_ref()],
        bump
    )]
    pub victim_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &game_state.current_epoch.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(
        mut,
        associated_token::mint = dump_mint,
        associated_token::authority = thief
    )]
    pub thief_dump_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = dump_mint,
        associated_token::authority = victim_key
    )]
    pub victim_dump_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [FEE_VAULT_SEED],
        bump
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [DUMP_MINT_SEED],
        bump
    )]
    pub dump_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(mut)]
    pub thief: Signer<'info>,

    /// CHECK: This is the victim's pubkey, verified by the victim_dump_account constraint
    pub victim_key: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn steal_dump_handler(ctx: Context<StealDump>, amount: u64) -> Result<()> {
    let thief_state = &mut ctx.accounts.thief_state;
    let victim_state = &mut ctx.accounts.victim_state;
    let game_state = &mut ctx.accounts.game_state;
    let epoch_state = &mut ctx.accounts.epoch_state;
    let clock = Clock::get()?;

    require!(!game_state.is_paused, GameError::GamePaused);
    require!(!game_state.is_waiting_period, GameError::EpochNotStarted);
    require!(amount > 0, GameError::InvalidAmount);
    require!(
        ctx.accounts.thief.key() != ctx.accounts.victim_key.key(),
        GameError::SelfTransfer
    );
    require!(thief_state.is_active_participant, GameError::NotActiveParticipant);
    require!(victim_state.is_active_participant, GameError::NotActiveParticipant);
    require!(
        clock.unix_timestamp >= thief_state.take_cooldown_end_time,
        GameError::PlayerInCooldown
    );

    // Victim must have sufficient balance
    require!(ctx.accounts.victim_dump_account.amount >= amount, GameError::InsufficientBalance);

    // Calculate fee and steal amounts
    let fee_amount = (amount * THEFT_FEE_BASIS_POINTS) / 10000;
    let steal_amount = amount - fee_amount;

    // Update time-weighted averages
    thief_state.update_time_weighted_average(clock.unix_timestamp)?;
    victim_state.update_time_weighted_average(clock.unix_timestamp)?;

    // Use program signer (game_state PDA) as delegate to transfer from victim to thief
    let seeds = &[GAME_STATE_SEED, &[game_state.bump]];
    let signer = &[&seeds[..]];

    // Transfer net amount to thief
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.victim_dump_account.to_account_info(),
            to: ctx.accounts.thief_dump_account.to_account_info(),
            authority: game_state.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, steal_amount)?;

    // Transfer fee to fee vault
    if fee_amount > 0 {
        let fee_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.victim_dump_account.to_account_info(),
                to: ctx.accounts.fee_vault.to_account_info(),
                authority: game_state.to_account_info(),
            },
            signer,
        );
        token::transfer(fee_ctx, fee_amount)?;
        game_state.total_fees_collected = game_state.total_fees_collected.saturating_add(fee_amount);
    }

    // Cooldown
    let cooldown_duration = calculate_theft_cooldown(amount);
    thief_state.total_dump_received = thief_state.total_dump_received.saturating_add(steal_amount);
    thief_state.total_thefts_made = thief_state.total_thefts_made.saturating_add(1);
    thief_state.last_transfer_time = clock.unix_timestamp;
    thief_state.take_cooldown_end_time = clock.unix_timestamp + cooldown_duration;

    // Sync balances from token accounts
    thief_state.current_dump_balance = ctx.accounts.thief_dump_account.amount;
    victim_state.current_dump_balance = ctx.accounts.victim_dump_account.amount;
    victim_state.total_dump_given = victim_state.total_dump_given.saturating_add(amount);

    epoch_state.total_thefts = epoch_state.total_thefts.saturating_add(1);

    msg!(
        "Theft: {} DUMP from {} to {} (fee: {} DUMP, cooldown: {}s)",
        steal_amount,
        ctx.accounts.victim_key.key(),
        ctx.accounts.thief.key(),
        fee_amount,
        cooldown_duration
    );

    Ok(())
}

// Helper functions
fn calculate_transfer_cooldown(amount: u64) -> i64 {
    // Scale cooldown based on amount transferred
    let base_cooldown = TRANSFER_COOLDOWN_MIN;
    let max_cooldown = TRANSFER_COOLDOWN_MAX;
    
    // For larger transfers, longer cooldowns
    let scaled_cooldown = base_cooldown + (amount / 1_000_000) as i64; // 1 second per million DUMP
    std::cmp::min(scaled_cooldown, max_cooldown)
}

fn calculate_theft_cooldown(amount: u64) -> i64 {
    // Theft has longer cooldowns than transfers
    let base_cooldown = THEFT_COOLDOWN_MIN;
    let max_cooldown = THEFT_COOLDOWN_MAX;
    
    // For larger thefts, much longer cooldowns
    let scaled_cooldown = base_cooldown + (amount / 500_000) as i64; // 1 second per 500k DUMP
    std::cmp::min(scaled_cooldown, max_cooldown)
}
