use anchor_lang::prelude::*;

#[error_code]
pub enum GloryDumpError {
    #[msg("This instruction is not available in the current phase")]
    WrongPhase,
    #[msg("The current phase has not reached its transition time")]
    TooEarly,
    #[msg("The current window has closed")]
    WindowClosed,
    #[msg("The epoch does not have enough registered players")]
    UnderfilledEpoch,
    #[msg("The epoch does not have enough valid randomness reveals")]
    InsufficientReveals,
    #[msg("The epoch participant cap has been reached")]
    ParticipantCapReached,
    #[msg("The supplied commitment is invalid")]
    InvalidCommitment,
    #[msg("The player has already revealed")]
    AlreadyRevealed,
    #[msg("The allocation has already been claimed")]
    AllocationAlreadyClaimed,
    #[msg("The player must claim an allocation first")]
    AllocationNotClaimed,
    #[msg("The signer is not authorized for this player")]
    Unauthorized,
    #[msg("Self-targeted actions are not allowed")]
    SelfAction,
    #[msg("The action amount is invalid")]
    InvalidAmount,
    #[msg("The target lane is not the deterministic lane for this action")]
    WrongTargetLane,
    #[msg("The source lane does not have enough unlocked DUMP")]
    InsufficientSpendableDump,
    #[msg("This action would exceed the lane's action heat capacity")]
    HeatCapacityExceeded,
    #[msg("REDIRECT cannot be armed yet")]
    RedirectNotReady,
    #[msg("REDIRECT requires non-zero Guard")]
    NoGuard,
    #[msg("The player has already been settled")]
    AlreadySettled,
    #[msg("Not every participant has been settled")]
    SettlementIncomplete,
    #[msg("The player is not an epoch winner")]
    NotWinner,
    #[msg("This reward has already been claimed")]
    RewardAlreadyClaimed,
    #[msg("This bond has already been claimed")]
    BondAlreadyClaimed,
    #[msg("The player did not qualify for a bond refund")]
    BondForfeited,
    #[msg("The epoch account cannot fund the expected deterministic payout")]
    InsufficientEpochFunds,
    #[msg("The supplied epoch number is not the next epoch")]
    InvalidNextEpoch,
    #[msg("The session duration or action allowance is invalid")]
    InvalidSession,
    #[msg("The rivalry account does not match this actor and target")]
    InvalidRivalry,
    #[msg("The account cannot be reclaimed until its bond and reward are resolved")]
    CleanupNotReady,
    #[msg("The player still has an unclaimed GLORY reward")]
    PendingReward,
    #[msg("An arithmetic operation overflowed")]
    ArithmeticOverflow,
}

pub fn map_rule_error(error: glory_dump_core::RuleError) -> Error {
    match error {
        glory_dump_core::RuleError::InvalidAmount => {
            error!(GloryDumpError::InvalidAmount)
        }
        glory_dump_core::RuleError::HeatCapacityExceeded => {
            error!(GloryDumpError::HeatCapacityExceeded)
        }
        glory_dump_core::RuleError::ArithmeticOverflow
        | glory_dump_core::RuleError::InvalidDuration => {
            error!(GloryDumpError::ArithmeticOverflow)
        }
    }
}
