use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Game is currently paused")]
    GamePaused,
    
    #[msg("Not in waiting period")]
    NotInWaitingPeriod,
    
    #[msg("Epoch has not started yet")]
    EpochNotStarted,
    
    #[msg("Epoch has already ended")]
    EpochEnded,
    
    #[msg("Player is not an active participant")]
    NotActiveParticipant,
    
    #[msg("Player is already an active participant")]
    AlreadyActiveParticipant,
    
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    
    #[msg("Transfer amount exceeds balance")]
    InsufficientBalance,
    
    #[msg("Player is in cooldown period")]
    PlayerInCooldown,
    
    #[msg("Cannot transfer to yourself")]
    SelfTransfer,
    
    #[msg("Invalid transfer amount")]
    InvalidAmount,
    
    #[msg("Epoch is already finalized")]
    EpochAlreadyFinalized,
    
    #[msg("Epoch is not finalized")]
    EpochNotFinalized,
    
    #[msg("Player already signed up for this epoch")]
    AlreadySignedUp,
    
    #[msg("Join fee payment insufficient")]
    InsufficientJoinFee,
    
    #[msg("Cannot withdraw stake during active epoch")]
    CannotWithdrawDuringEpoch,
    
    #[msg("Bug report already exists")]
    BugReportExists,
    
    #[msg("Bug report not found")]
    BugReportNotFound,
    
    #[msg("Bug report already verified")]
    BugReportAlreadyVerified,
    
    #[msg("Bug report already paid")]
    BugReportAlreadyPaid,
    
    #[msg("Invalid bug severity level")]
    InvalidBugSeverity,
    
    #[msg("Reward tier mismatch")]
    RewardTierMismatch,
    
    #[msg("Player not eligible for rewards")]
    NotEligibleForRewards,
    
    #[msg("Rewards already distributed for this tier")]
    RewardsAlreadyDistributed,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Invalid epoch number")]
    InvalidEpoch,
    
    #[msg("Unauthorized operation")]
    Unauthorized,

    #[msg("Rewards already claimed")]
    AlreadyClaimed,

    #[msg("Merkle root not set for this epoch")]
    MerkleRootNotSet,

    #[msg("Invalid Merkle proof")]
    InvalidMerkleProof,
}
