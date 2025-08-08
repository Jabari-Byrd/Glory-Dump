use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Token, TokenAccount};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;

#[derive(Accounts)]
pub struct SetRewardsRoot<'info> {
    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &epoch_state.epoch_number.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    /// Admin must sign
    #[account(mut, constraint = admin.key() == game_state.admin)]
    pub admin: Signer<'info>,
}

pub fn set_rewards_root_handler(ctx: Context<SetRewardsRoot>, merkle_root: [u8; 32]) -> Result<()> {
    let epoch_state = &mut ctx.accounts.epoch_state;
    require!(epoch_state.is_finalized, GameError::EpochNotFinalized);
    epoch_state.merkle_root = merkle_root;
    epoch_state.merkle_root_set = true;
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [EPOCH_STATE_SEED, &epoch_state.epoch_number.to_le_bytes()],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        init_if_needed,
        payer = claimer,
        space = ClaimStatus::LEN,
        seeds = [CLAIM_STATUS_SEED, &epoch_state.epoch_number.to_le_bytes(), claimer.key().as_ref()],
        bump
    )]
    pub claim_status: Account<'info, ClaimStatus>,

    #[account(
        mut,
        seeds = [GLORY_MINT_SEED],
        bump
    )]
    pub glory_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        init_if_needed,
        payer = claimer,
        mut,
        associated_token::mint = glory_mint,
        associated_token::authority = claimer
    )]
    pub claimer_glory_account: Account<'info, TokenAccount>,

    /// CHECK: claimer is a system account
    #[account(mut)]
    pub claimer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

// Lightweight on-chain Merkle verification for fixed 32-byte leaves using keccak256
fn keccak256(data: &[u8]) -> [u8; 32] {
    use solana_program::keccak::hash;
    hash(data).0
}

pub fn claim_rewards_handler(
    ctx: Context<ClaimRewards>,
    amount: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let epoch_state = &mut ctx.accounts.epoch_state;
    let game_state = &mut ctx.accounts.game_state;
    let claim_status = &mut ctx.accounts.claim_status;

    require!(epoch_state.is_finalized, GameError::EpochNotFinalized);
    require!(epoch_state.merkle_root_set, GameError::MerkleRootNotSet);
    require!(!claim_status.claimed, GameError::AlreadyClaimed);

    // Leaf: keccak(claimer || amount_u64_le)
    let mut leaf_data = Vec::with_capacity(32 + 8);
    leaf_data.extend_from_slice(ctx.accounts.claimer.key().as_ref());
    leaf_data.extend_from_slice(&amount.to_le_bytes());
    let mut computed = keccak256(&leaf_data);

    for p in proof.iter() {
        // Hash pair with lexical order to be consistent
        let (left, right) = if computed <= *p { (computed, *p) } else { (*p, computed) };
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(&left);
        buf[32..].copy_from_slice(&right);
        computed = keccak256(&buf);
    }

    require!(computed == epoch_state.merkle_root, GameError::InvalidMerkleProof);

    // Enforce GLORY cap
    require!(game_state.total_glory_minted.saturating_add(amount) <= GLORY_SUPPLY_CAP, GameError::InvalidAmount);

    // Mint to claimer
    let seeds = &[GAME_STATE_SEED, &[game_state.bump]];
    let signer = &[&seeds[..]];
    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.glory_mint.to_account_info(),
            to: ctx.accounts.claimer_glory_account.to_account_info(),
            authority: ctx.accounts.game_state.to_account_info(),
        },
        signer,
    );
    token::mint_to(mint_ctx, amount)?;

    // Mark claimed and update totals
    claim_status.epoch_number = epoch_state.epoch_number;
    claim_status.player = ctx.accounts.claimer.key();
    claim_status.claimed = true;
    epoch_state.total_glory_rewards = epoch_state.total_glory_rewards.saturating_add(amount);
    game_state.total_glory_minted = game_state.total_glory_minted.saturating_add(amount);

    Ok(())
}
