use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct UpdatePlayerAverage<'info> {
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

    pub player: Signer<'info>,
}

pub fn update_average_handler(ctx: Context<UpdatePlayerAverage>) -> Result<()> {
    let player_state = &mut ctx.accounts.player_state;
    let game_state = &ctx.accounts.game_state;
    let clock = Clock::get()?;

    // Check if player is active participant
    require!(player_state.is_active_participant, GameError::NotActiveParticipant);

    // Check if not in waiting period
    require!(!game_state.is_waiting_period, GameError::EpochNotStarted);

    // Update the time-weighted average
    player_state.update_time_weighted_average(clock.unix_timestamp)?;

    msg!("Updated time-weighted average for player: {}", ctx.accounts.player.key());

    Ok(())
}
