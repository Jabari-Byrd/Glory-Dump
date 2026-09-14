use anchor_lang::prelude::*;

pub mod contexts;
pub mod errors;
pub mod events;
pub mod handlers;
pub mod state;

pub use contexts::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod glory_dump {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        handlers::initialize_protocol(ctx)
    }

    pub fn open_next_epoch(ctx: Context<OpenNextEpoch>, next_epoch: u64) -> Result<()> {
        handlers::open_next_epoch(ctx, next_epoch)
    }

    pub fn register(ctx: Context<Register>, commitment: [u8; 32]) -> Result<()> {
        handlers::register(ctx, commitment)
    }

    pub fn begin_reveal(ctx: Context<TransitionEpoch>) -> Result<()> {
        handlers::begin_reveal(ctx)
    }

    pub fn reveal(ctx: Context<Reveal>, secret: [u8; 32]) -> Result<()> {
        handlers::reveal(ctx, secret)
    }

    pub fn seal_randomness(ctx: Context<TransitionEpoch>) -> Result<()> {
        handlers::seal_randomness(ctx)
    }

    pub fn cancel_epoch(ctx: Context<TransitionEpoch>) -> Result<()> {
        handlers::cancel_epoch(ctx)
    }

    pub fn claim_allocation(ctx: Context<ClaimAllocation>) -> Result<()> {
        handlers::claim_allocation(ctx)
    }

    pub fn begin_active(ctx: Context<TransitionEpoch>) -> Result<()> {
        handlers::begin_active(ctx)
    }

    pub fn authorize_session(
        ctx: Context<ManageSession>,
        delegate: Pubkey,
        duration_seconds: i64,
        max_actions: u16,
    ) -> Result<()> {
        handlers::authorize_session(ctx, delegate, duration_seconds, max_actions)
    }

    pub fn revoke_session(ctx: Context<ManageSession>) -> Result<()> {
        handlers::revoke_session(ctx)
    }

    pub fn dump(
        ctx: Context<GameplayAction>,
        amount: u64,
        source_index: u8,
        target_index: u8,
    ) -> Result<()> {
        handlers::dump(ctx, amount, source_index, target_index)
    }

    pub fn absorb(
        ctx: Context<GameplayAction>,
        amount: u64,
        destination_index: u8,
        source_index: u8,
    ) -> Result<()> {
        handlers::absorb(ctx, amount, destination_index, source_index)
    }

    pub fn arm_redirect(ctx: Context<ArmRedirect>, lane_index: u8) -> Result<()> {
        handlers::arm_redirect(ctx, lane_index)
    }

    pub fn begin_settlement(ctx: Context<TransitionEpoch>) -> Result<()> {
        handlers::begin_settlement(ctx)
    }

    pub fn settle_player(ctx: Context<SettlePlayer>) -> Result<()> {
        handlers::settle_player(ctx)
    }

    pub fn complete_epoch(ctx: Context<CompleteEpoch>) -> Result<()> {
        handlers::complete_epoch(ctx)
    }

    pub fn claim_player_reward(ctx: Context<ClaimPlayerReward>) -> Result<()> {
        handlers::claim_player_reward(ctx)
    }

    pub fn claim_keeper_reward(ctx: Context<ClaimKeeperReward>) -> Result<()> {
        handlers::claim_keeper_reward(ctx)
    }

    pub fn claim_bond(ctx: Context<ClaimBond>) -> Result<()> {
        handlers::claim_bond(ctx)
    }

    pub fn refresh_badges(ctx: Context<RefreshBadges>) -> Result<()> {
        handlers::refresh_badges(ctx)
    }

    pub fn sweep_expired_bonds(ctx: Context<SweepExpiredBonds>) -> Result<()> {
        handlers::sweep_expired_bonds(ctx)
    }

    pub fn close_rivalry(ctx: Context<CloseRivalry>) -> Result<()> {
        handlers::close_rivalry(ctx)
    }

    pub fn close_player_accounts(ctx: Context<ClosePlayerAccounts>) -> Result<()> {
        handlers::close_player_accounts(ctx)
    }

    pub fn close_keeper_credit(ctx: Context<CloseKeeperCredit>) -> Result<()> {
        handlers::close_keeper_credit(ctx)
    }
}
