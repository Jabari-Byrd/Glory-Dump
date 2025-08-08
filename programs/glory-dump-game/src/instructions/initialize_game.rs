use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = admin,
        space = GameState::LEN,
        seeds = [GAME_STATE_SEED],
        bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        init,
        payer = admin,
        mint::decimals = DUMP_DECIMALS,
        mint::authority = game_state,
        seeds = [DUMP_MINT_SEED],
        bump
    )]
    pub dump_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        mint::decimals = GLORY_DECIMALS,
        mint::authority = game_state,
        seeds = [GLORY_MINT_SEED],
        bump
    )]
    pub glory_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        token::mint = dump_mint,
        token::authority = game_state,
        seeds = [FEE_VAULT_SEED],
        bump
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        token::mint = glory_mint,
        token::authority = game_state,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury: Account<'info, TokenAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeGame>, bump: u8) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    game_state.admin = ctx.accounts.admin.key();
    game_state.dump_mint = ctx.accounts.dump_mint.key();
    game_state.glory_mint = ctx.accounts.glory_mint.key();
    game_state.fee_vault = ctx.accounts.fee_vault.key();
    game_state.treasury = ctx.accounts.treasury.key();
    game_state.current_epoch = 1;
    game_state.epoch_start_time = clock.unix_timestamp;
    game_state.next_epoch_start_time = clock.unix_timestamp + WAITING_PERIOD;
    game_state.is_waiting_period = true;
    game_state.is_paused = false;
    game_state.total_participants = 0;
    game_state.total_fees_collected = 0;
    game_state.total_glory_distributed = 0;
    game_state.total_glory_minted = 0;
    game_state.bump = bump;

    msg!("Glory Dump Game initialized successfully!");
    msg!("DUMP Mint: {}", ctx.accounts.dump_mint.key());
    msg!("GLORY Mint: {}", ctx.accounts.glory_mint.key());
    msg!("First epoch starts at: {}", game_state.next_epoch_start_time);

    Ok(())
}
