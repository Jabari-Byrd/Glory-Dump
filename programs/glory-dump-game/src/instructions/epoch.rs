use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, MintTo};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;

// Initialize Epoch
#[derive(Accounts)]
pub struct InitializeEpoch<'info> {
    #[account(
        init,
        payer = admin,
        space = EpochState::MAX_LEN,
        seeds = [EPOCH_STATE_SEED, &game_state.current_epoch.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(mut)]
    pub game_state: Account<'info, GameState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_epoch_handler(ctx: Context<InitializeEpoch>) -> Result<()> {
    let game_state = &ctx.accounts.game_state;
    let epoch_state = &mut ctx.accounts.epoch_state;
    let clock = Clock::get()?;

    epoch_state.epoch_number = game_state.current_epoch;
    epoch_state.start_time = game_state.next_epoch_start_time;
    epoch_state.end_time = game_state.next_epoch_start_time + EPOCH_DURATION;
    epoch_state.participants = Vec::new();
    epoch_state.is_finalized = false;
    epoch_state.total_dump_distributed = 0;
    epoch_state.total_glory_rewards = 0;
    epoch_state.winner = None;
    epoch_state.rewards_distributed = [false; 4];
    epoch_state.total_transfers = 0;
    epoch_state.total_thefts = 0;
    epoch_state.highest_volume_player = None;
    epoch_state.is_bonus_epoch = false;
    epoch_state.bonus_multiplier = 100; // 100% = 1x multiplier
    epoch_state.merkle_root = [0u8; 32];
    epoch_state.merkle_root_set = false;

    msg!("Epoch {} initialized. Waiting period active until {}", 
         game_state.current_epoch, 
         game_state.next_epoch_start_time);

    Ok(())
}

// Sign Up for Epoch
#[derive(Accounts)]
pub struct SignUpForEpoch<'info> {
    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &game_state.current_epoch.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(mut)]
    pub game_state: Account<'info, GameState>,

    #[account(mut)]
    pub player: Signer<'info>,

    /// Treasury account (PDA) to receive join fees (SOL)
    /// CHECK: PDA validated by seed on initialize; SOL transfers require only writable
    #[account(mut, address = game_state.treasury)]
    pub treasury_sol_receiver: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn sign_up_handler(ctx: Context<SignUpForEpoch>, payment_amount: u64) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let epoch_state = &mut ctx.accounts.epoch_state;
    let player_state = &mut ctx.accounts.player_state;
    let clock = Clock::get()?;

    // Check if in waiting period
    require!(game_state.is_waiting_period, GameError::NotInWaitingPeriod);
    
    // Check if player already signed up
    require!(
        player_state.current_epoch_signed_up != game_state.current_epoch,
        GameError::AlreadySignedUp
    );

    // Check if player has sufficient stake
    require!(
        player_state.is_active_participant,
        GameError::NotActiveParticipant
    );

    // Calculate required join fee based on time remaining
    let time_until_start = game_state.next_epoch_start_time - clock.unix_timestamp;
    let time_into_waiting = WAITING_PERIOD - time_until_start;
    let fee_percentage = (time_into_waiting * 100) / WAITING_PERIOD;
    let required_fee = BASE_JOIN_FEE + ((MAX_JOIN_FEE - BASE_JOIN_FEE) * fee_percentage as u64) / 100;

    require!(payment_amount >= required_fee, GameError::InsufficientJoinFee);

    // Transfer SOL payment to treasury
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.player.key(),
        &ctx.accounts.treasury_sol_receiver.key(),
        payment_amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.player.to_account_info(),
            ctx.accounts.treasury_sol_receiver.to_account_info(),
        ],
    )?;

    // Add player to epoch participants with cap
    require!(
        epoch_state.participants.len() < MAX_PARTICIPANTS_PER_EPOCH as usize,
        GameError::InvalidAmount
    );
    epoch_state.participants.push(ctx.accounts.player.key());
    
    // Update player state
    player_state.current_epoch_signed_up = game_state.current_epoch;
    player_state.join_fee_paid = payment_amount;

    // Update game state
    game_state.total_participants += 1;

    msg!("Player {} signed up for epoch {} with fee: {} lamports", 
         ctx.accounts.player.key(), 
         game_state.current_epoch,
         payment_amount);

    Ok(())
}

// Start Epoch
#[derive(Accounts)]
pub struct StartEpoch<'info> {
    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &game_state.current_epoch.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(mut)]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        seeds = [DUMP_MINT_SEED],
        bump
    )]
    pub dump_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn start_epoch_handler(ctx: Context<StartEpoch>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let epoch_state = &mut ctx.accounts.epoch_state;
    let clock = Clock::get()?;

    // Check if waiting period is over
    require!(
        clock.unix_timestamp >= game_state.next_epoch_start_time,
        GameError::EpochNotStarted
    );

    // End waiting period
    game_state.is_waiting_period = false;
    game_state.epoch_start_time = clock.unix_timestamp;

    // Calculate total DUMP to distribute
    let participants_count = epoch_state.participants.len() as u64;
    if participants_count > 0 {
        // Mint DUMP for distribution (simplified - in practice you'd use a more sophisticated random distribution)
        let total_dump_for_epoch = participants_count * MAX_DUMP_ASSIGNMENT / 2; // Average assignment
        epoch_state.total_dump_distributed = total_dump_for_epoch;

        // Note: In a real implementation, you'd need to:
        // 1. Use verifiable randomness (e.g., Switchboard/Chainlink VRF)
        // 2. Distribute DUMP to individual player token accounts
        // 3. Initialize time-weighted tracking for all players
    }

    msg!("Epoch {} started with {} participants!", 
         game_state.current_epoch,
         participants_count);

    Ok(())
}

// Finalize Epoch
#[derive(Accounts)]
pub struct FinalizeEpoch<'info> {
    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &game_state.current_epoch.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(mut)]
    pub game_state: Account<'info, GameState>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

pub fn finalize_epoch_handler(ctx: Context<FinalizeEpoch>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let epoch_state = &mut ctx.accounts.epoch_state;
    let clock = Clock::get()?;

    // Check if epoch has ended
    require!(clock.unix_timestamp >= epoch_state.end_time, GameError::EpochEnded);

    // Check if not already finalized
    require!(!epoch_state.is_finalized, GameError::EpochAlreadyFinalized);

    // Mark as finalized
    epoch_state.is_finalized = true;

    // Prepare for next epoch
    game_state.current_epoch += 1;
    game_state.next_epoch_start_time = clock.unix_timestamp + WAITING_PERIOD;
    game_state.is_waiting_period = true;

    // Note: In a real implementation, you'd need to:
    // 1. Calculate final rankings based on time-weighted averages
    // 2. Determine winners and reward tiers
    // 3. Check for bonus epoch conditions
    // 4. Set up the next epoch state

    msg!("Epoch {} finalized. Next epoch {} waiting period starts now.",
         epoch_state.epoch_number,
         game_state.current_epoch);

    Ok(())
}
