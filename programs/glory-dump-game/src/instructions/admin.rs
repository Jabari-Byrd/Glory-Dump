use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct EmergencyAction<'info> {
    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        constraint = admin.key() == game_state.admin
    )]
    pub admin: Signer<'info>,
}

pub fn emergency_pause_handler(ctx: Context<EmergencyAction>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    game_state.is_paused = true;

    msg!("Game emergency paused by admin: {}", ctx.accounts.admin.key());

    Ok(())
}

pub fn emergency_unpause_handler(ctx: Context<EmergencyAction>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;

    require!(game_state.is_paused, GameError::GamePaused);

    game_state.is_paused = false;

    msg!("Game emergency unpaused by admin: {}", ctx.accounts.admin.key());

    Ok(())
}
