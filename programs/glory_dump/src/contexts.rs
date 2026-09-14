use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::state::{BalanceLane, Epoch, KeeperCredit, Leaderboard, PlayerEpoch, Protocol, Rivalry};

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = Protocol::SPACE,
        seeds = [b"protocol"],
        bump
    )]
    pub protocol: Account<'info, Protocol>,
    #[account(
        init,
        payer = payer,
        seeds = [b"glory_mint"],
        bump,
        mint::decimals = glory_dump_core::GLORY_DECIMALS,
        mint::authority = protocol
    )]
    pub glory_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        space = Epoch::SPACE,
        seeds = [b"epoch", 1u64.to_le_bytes().as_ref()],
        bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        init,
        payer = payer,
        space = Leaderboard::SPACE,
        seeds = [b"leaderboard", 1u64.to_le_bytes().as_ref()],
        bump
    )]
    pub leaderboard: Account<'info, Leaderboard>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(next_epoch: u64)]
pub struct OpenNextEpoch<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [b"protocol"], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,
    #[account(
        seeds = [b"epoch", protocol.current_epoch.to_le_bytes().as_ref()],
        bump = previous_epoch.bump,
        constraint = previous_epoch.number == protocol.current_epoch
    )]
    pub previous_epoch: Account<'info, Epoch>,
    #[account(
        init,
        payer = payer,
        space = Epoch::SPACE,
        seeds = [b"epoch", next_epoch.to_le_bytes().as_ref()],
        bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        init,
        payer = payer,
        space = Leaderboard::SPACE,
        seeds = [b"leaderboard", next_epoch.to_le_bytes().as_ref()],
        bump
    )]
    pub leaderboard: Account<'info, Leaderboard>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransitionEpoch<'info> {
    #[account(seeds = [b"protocol"], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,
    #[account(
        mut,
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump,
        constraint = epoch.number == protocol.current_epoch
    )]
    pub epoch: Account<'info, Epoch>,
}

#[derive(Accounts)]
#[instruction(commitment: [u8; 32])]
pub struct Register<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"protocol"], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,
    #[account(
        mut,
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump,
        constraint = epoch.number == protocol.current_epoch
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        init,
        payer = payer,
        space = PlayerEpoch::SPACE,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub player: Account<'info, PlayerEpoch>,
    #[account(
        init,
        payer = payer,
        space = BalanceLane::SPACE,
        seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), payer.key().as_ref(), &[0]],
        bump
    )]
    pub lane_zero: Account<'info, BalanceLane>,
    #[account(
        init,
        payer = payer,
        space = BalanceLane::SPACE,
        seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), payer.key().as_ref(), &[1]],
        bump
    )]
    pub lane_one: Account<'info, BalanceLane>,
    #[account(
        init,
        payer = payer,
        space = BalanceLane::SPACE,
        seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), payer.key().as_ref(), &[2]],
        bump
    )]
    pub lane_two: Account<'info, BalanceLane>,
    #[account(
        init,
        payer = payer,
        space = BalanceLane::SPACE,
        seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), payer.key().as_ref(), &[3]],
        bump
    )]
    pub lane_three: Account<'info, BalanceLane>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Reveal<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
}

#[derive(Accounts)]
pub struct ClaimAllocation<'info> {
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[0]], bump = lane_zero.bump)]
    pub lane_zero: Account<'info, BalanceLane>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[1]], bump = lane_one.bump)]
    pub lane_one: Account<'info, BalanceLane>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[2]], bump = lane_two.bump)]
    pub lane_two: Account<'info, BalanceLane>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[3]], bump = lane_three.bump)]
    pub lane_three: Account<'info, BalanceLane>,
}

#[derive(Accounts)]
pub struct ManageSession<'info> {
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
}

#[derive(Accounts)]
#[instruction(amount: u64, source_index: u8, target_index: u8)]
pub struct GameplayAction<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), actor_player.owner.as_ref()],
        bump = actor_player.bump,
        constraint = actor_player.epoch == epoch.number
    )]
    pub actor_player: Account<'info, PlayerEpoch>,
    #[account(
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), target_player.owner.as_ref()],
        bump = target_player.bump,
        constraint = target_player.epoch == epoch.number
    )]
    pub target_player: Account<'info, PlayerEpoch>,
    #[account(
        mut,
        seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), actor_player.owner.as_ref(), &[source_index]],
        bump = actor_lane.bump,
        constraint = actor_lane.owner == actor_player.owner,
        constraint = actor_lane.epoch == epoch.number,
        constraint = actor_lane.index == source_index
    )]
    pub actor_lane: Account<'info, BalanceLane>,
    #[account(
        mut,
        seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), target_player.owner.as_ref(), &[target_index]],
        bump = target_lane.bump,
        constraint = target_lane.owner == target_player.owner,
        constraint = target_lane.epoch == epoch.number,
        constraint = target_lane.index == target_index
    )]
    pub target_lane: Account<'info, BalanceLane>,
    #[account(
        init_if_needed,
        payer = signer,
        space = Rivalry::SPACE,
        seeds = [b"rivalry", epoch.number.to_le_bytes().as_ref(), actor_player.owner.as_ref(), target_player.owner.as_ref()],
        bump
    )]
    pub rivalry: Account<'info, Rivalry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(lane_index: u8)]
pub struct ArmRedirect<'info> {
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), player.owner.as_ref()],
        bump = player.bump,
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
    #[account(
        mut,
        seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), player.owner.as_ref(), &[lane_index]],
        bump = lane.bump,
        constraint = lane.owner == player.owner,
        constraint = lane.epoch == epoch.number,
        constraint = lane.index == lane_index
    )]
    pub lane: Account<'info, BalanceLane>,
}

#[derive(Accounts)]
pub struct SettlePlayer<'info> {
    #[account(mut)]
    pub keeper: Signer<'info>,
    #[account(
        mut,
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"leaderboard", epoch.number.to_le_bytes().as_ref()],
        bump = leaderboard.bump,
        constraint = leaderboard.epoch == epoch.number
    )]
    pub leaderboard: Account<'info, Leaderboard>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), player.owner.as_ref()],
        bump = player.bump,
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), player.owner.as_ref(), &[0]], bump = lane_zero.bump)]
    pub lane_zero: Account<'info, BalanceLane>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), player.owner.as_ref(), &[1]], bump = lane_one.bump)]
    pub lane_one: Account<'info, BalanceLane>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), player.owner.as_ref(), &[2]], bump = lane_two.bump)]
    pub lane_two: Account<'info, BalanceLane>,
    #[account(mut, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), player.owner.as_ref(), &[3]], bump = lane_three.bump)]
    pub lane_three: Account<'info, BalanceLane>,
    #[account(
        init_if_needed,
        payer = keeper,
        space = KeeperCredit::SPACE,
        seeds = [b"keeper", epoch.number.to_le_bytes().as_ref(), keeper.key().as_ref()],
        bump
    )]
    pub keeper_credit: Account<'info, KeeperCredit>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CompleteEpoch<'info> {
    pub finalizer: Signer<'info>,
    #[account(mut, seeds = [b"protocol"], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,
    #[account(
        mut,
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump,
        constraint = epoch.number == protocol.current_epoch
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"leaderboard", epoch.number.to_le_bytes().as_ref()],
        bump = leaderboard.bump,
        constraint = leaderboard.epoch == epoch.number
    )]
    pub leaderboard: Account<'info, Leaderboard>,
}

#[derive(Accounts)]
pub struct ClaimPlayerReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [b"protocol"], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"leaderboard", epoch.number.to_le_bytes().as_ref()],
        bump = leaderboard.bump,
        constraint = leaderboard.epoch == epoch.number
    )]
    pub leaderboard: Account<'info, Leaderboard>,
    #[account(
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
    #[account(mut, address = protocol.glory_mint)]
    pub glory_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = glory_mint,
        associated_token::authority = owner
    )]
    pub destination: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimKeeperReward<'info> {
    #[account(mut)]
    pub keeper: Signer<'info>,
    #[account(seeds = [b"protocol"], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"keeper", epoch.number.to_le_bytes().as_ref(), keeper.key().as_ref()],
        bump = keeper_credit.bump,
        constraint = keeper_credit.keeper == keeper.key(),
        constraint = keeper_credit.epoch == epoch.number
    )]
    pub keeper_credit: Account<'info, KeeperCredit>,
    #[account(mut, address = protocol.glory_mint)]
    pub glory_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = keeper,
        associated_token::mint = glory_mint,
        associated_token::authority = keeper
    )]
    pub destination: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimBond<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
}

#[derive(Accounts)]
pub struct RefreshBadges<'info> {
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
}

#[derive(Accounts)]
pub struct SweepExpiredBonds<'info> {
    pub sweeper: Signer<'info>,
    #[account(seeds = [b"protocol"], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,
    #[account(
        mut,
        seeds = [b"epoch", old_epoch.number.to_le_bytes().as_ref()],
        bump = old_epoch.bump
    )]
    pub old_epoch: Account<'info, Epoch>,
    #[account(
        mut,
        seeds = [b"epoch", current_epoch.number.to_le_bytes().as_ref()],
        bump = current_epoch.bump,
        constraint = current_epoch.number == protocol.current_epoch
    )]
    pub current_epoch: Account<'info, Epoch>,
}

#[derive(Accounts)]
pub struct CloseRivalry<'info> {
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    /// CHECK: Constrained to the account that funded this rivalry PDA.
    #[account(mut, address = rivalry.rent_payer)]
    pub rent_recipient: UncheckedAccount<'info>,
    #[account(
        mut,
        close = rent_recipient,
        seeds = [b"rivalry", epoch.number.to_le_bytes().as_ref(), rivalry.actor.as_ref(), rivalry.target.as_ref()],
        bump = rivalry.bump,
        constraint = rivalry.epoch == epoch.number
    )]
    pub rivalry: Account<'info, Rivalry>,
}

#[derive(Accounts)]
pub struct ClosePlayerAccounts<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        seeds = [b"leaderboard", epoch.number.to_le_bytes().as_ref()],
        bump = leaderboard.bump,
        constraint = leaderboard.epoch == epoch.number
    )]
    pub leaderboard: Account<'info, Leaderboard>,
    #[account(
        mut,
        close = owner,
        seeds = [b"player", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
        constraint = player.epoch == epoch.number
    )]
    pub player: Account<'info, PlayerEpoch>,
    #[account(mut, close = owner, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[0]], bump = lane_zero.bump)]
    pub lane_zero: Account<'info, BalanceLane>,
    #[account(mut, close = owner, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[1]], bump = lane_one.bump)]
    pub lane_one: Account<'info, BalanceLane>,
    #[account(mut, close = owner, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[2]], bump = lane_two.bump)]
    pub lane_two: Account<'info, BalanceLane>,
    #[account(mut, close = owner, seeds = [b"lane", epoch.number.to_le_bytes().as_ref(), owner.key().as_ref(), &[3]], bump = lane_three.bump)]
    pub lane_three: Account<'info, BalanceLane>,
}

#[derive(Accounts)]
pub struct CloseKeeperCredit<'info> {
    #[account(mut)]
    pub keeper: Signer<'info>,
    #[account(
        seeds = [b"epoch", epoch.number.to_le_bytes().as_ref()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    #[account(
        mut,
        close = keeper,
        seeds = [b"keeper", epoch.number.to_le_bytes().as_ref(), keeper.key().as_ref()],
        bump = keeper_credit.bump,
        constraint = keeper_credit.keeper == keeper.key(),
        constraint = keeper_credit.epoch == epoch.number
    )]
    pub keeper_credit: Account<'info, KeeperCredit>,
}
