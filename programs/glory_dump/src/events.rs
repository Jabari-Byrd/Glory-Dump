use anchor_lang::prelude::*;

#[event]
pub struct ProtocolInitialized {
    pub protocol: Pubkey,
    pub glory_mint: Pubkey,
    pub first_epoch: Pubkey,
}

#[event]
pub struct EpochOpened {
    pub epoch: u64,
    pub registration_ends_at: i64,
}

#[event]
pub struct PlayerRegistered {
    pub epoch: u64,
    pub player: Pubkey,
}

#[event]
pub struct RevealPhaseOpened {
    pub epoch: u64,
    pub reveal_ends_at: i64,
}

#[event]
pub struct SecretRevealed {
    pub epoch: u64,
    pub player: Pubkey,
}

#[event]
pub struct RandomnessSealed {
    pub epoch: u64,
    pub seed: [u8; 32],
    pub active_starts_at: i64,
    pub active_ends_at: i64,
}

#[event]
pub struct EpochCancelled {
    pub epoch: u64,
    pub participant_count: u32,
    pub revealed_count: u32,
}

#[event]
pub struct AllocationClaimed {
    pub epoch: u64,
    pub player: Pubkey,
    pub amount: u64,
    pub revealed: bool,
}

#[event]
pub struct ActivePlayStarted {
    pub epoch: u64,
    pub ends_at: i64,
}

#[event]
pub struct SessionAuthorized {
    pub epoch: u64,
    pub player: Pubkey,
    pub delegate: Pubkey,
    pub expires_at: i64,
    pub actions: u16,
}

#[event]
pub struct SessionRevoked {
    pub epoch: u64,
    pub player: Pubkey,
}

#[event]
pub struct Dumped {
    pub epoch: u64,
    pub actor: Pubkey,
    pub target: Pubkey,
    pub attempted: u64,
    pub landed: u64,
    pub redirected: u64,
    pub source_lane: u8,
    pub target_lane: u8,
    pub dump_heat: u32,
}

#[event]
pub struct Absorbed {
    pub epoch: u64,
    pub actor: Pubkey,
    pub target: Pubkey,
    pub amount: u64,
    pub destination_lane: u8,
    pub source_lane: u8,
    pub guard_after: u64,
    pub locked_until: i64,
    pub absorb_heat: u32,
}

#[event]
pub struct RedirectArmed {
    pub epoch: u64,
    pub player: Pubkey,
    pub lane: u8,
    pub guard: u64,
}

#[event]
pub struct SettlementStarted {
    pub epoch: u64,
    pub participant_count: u32,
}

#[event]
pub struct PlayerSettled {
    pub epoch: u64,
    pub player: Pubkey,
    pub score: u64,
    pub eligible: bool,
    pub keeper: Pubkey,
}

#[event]
pub struct EpochCompleted {
    pub epoch: u64,
    pub eligible_count: u32,
    pub winner_count: u16,
    pub total_reward_pool: u64,
    pub player_reward_pool: u64,
    pub keeper_reward_pool: u64,
}

#[event]
pub struct PlayerRewardClaimed {
    pub epoch: u64,
    pub player: Pubkey,
    pub rank: u16,
    pub amount: u64,
}

#[event]
pub struct KeeperRewardClaimed {
    pub epoch: u64,
    pub keeper: Pubkey,
    pub players_settled: u32,
    pub amount: u64,
}

#[event]
pub struct BondClaimed {
    pub epoch: u64,
    pub player: Pubkey,
    pub amount: u64,
}

#[event]
pub struct BadgesRefreshed {
    pub epoch: u64,
    pub player: Pubkey,
    pub badges: u64,
}

#[event]
pub struct ExcessBondsSwept {
    pub from_epoch: u64,
    pub to_epoch: u64,
    pub amount: u64,
}
