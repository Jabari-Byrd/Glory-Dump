use anchor_lang::{
    prelude::*,
    system_program::{self, Transfer},
};
use anchor_spl::token::{self, MintTo};
use solana_sha256_hasher::hashv;

use crate::{
    contexts::*,
    errors::GloryDumpError,
    events::*,
    state::{
        BADGE_GARBAGE_EMPEROR, BalanceLane, Epoch, HeatState, Leaderboard, Phase, PlayerEpoch,
        Protocol, Rivalry, WinnerEntry,
    },
};
use glory_dump_core::{
    self as rules, ABSORB_LOCK_SECONDS, ACTIVE_SECONDS, BOND_REFUND_BPS, BPS_DENOMINATOR,
    LANE_COUNT, MAX_GLORY_SUPPLY, MAX_PARTICIPANTS, MAX_STARTING_DUMP, MIN_REVEALED_PLAYERS,
    MIN_REWARDED_PLAYERS, PLANNING_SECONDS, REDIRECT_REARM_SECONDS, REGISTRATION_BOND_LAMPORTS,
    REGISTRATION_SECONDS, REVEAL_SECONDS, SETTLEMENT_BOUNTY_LAMPORTS,
};

const BOND_CLAIM_SECONDS: i64 = 7 * 24 * 60 * 60;
const MAX_SESSION_SECONDS: i64 = 24 * 60 * 60;
const MAX_SESSION_ACTIONS: u16 = 100;

const COMMITMENT_DOMAIN: &[u8] = b"glory-dump-commitment-v3";
const CONTRIBUTION_DOMAIN: &[u8] = b"glory-dump-contribution-v3";
const SEED_DOMAIN: &[u8] = b"glory-dump-epoch-seed-v3";
const ALLOCATION_DOMAIN: &[u8] = b"glory-dump-allocation-v3";
const TARGET_LANE_DOMAIN: &[u8] = b"glory-dump-target-lane-v3";
const TIE_BREAK_DOMAIN: &[u8] = b"glory-dump-tie-break-v3";

pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let protocol = &mut ctx.accounts.protocol;
    protocol.version = Protocol::VERSION;
    protocol.current_epoch = 1;
    protocol.glory_mint = ctx.accounts.glory_mint.key();
    protocol.glory_committed = 0;
    protocol.bump = ctx.bumps.protocol;

    initialize_epoch(&mut ctx.accounts.epoch, 1, now, ctx.bumps.epoch)?;
    initialize_leaderboard(&mut ctx.accounts.leaderboard, 1, ctx.bumps.leaderboard);

    emit!(ProtocolInitialized {
        protocol: protocol.key(),
        glory_mint: ctx.accounts.glory_mint.key(),
        first_epoch: ctx.accounts.epoch.key(),
    });
    emit!(EpochOpened {
        epoch: 1,
        registration_ends_at: ctx.accounts.epoch.registration_ends_at,
    });
    Ok(())
}

pub fn open_next_epoch(ctx: Context<OpenNextEpoch>, next_epoch: u64) -> Result<()> {
    require!(
        matches!(
            ctx.accounts.previous_epoch.phase,
            Phase::Complete | Phase::Cancelled
        ),
        GloryDumpError::WrongPhase
    );
    let expected = ctx
        .accounts
        .protocol
        .current_epoch
        .checked_add(1)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    require_eq!(next_epoch, expected, GloryDumpError::InvalidNextEpoch);

    let now = Clock::get()?.unix_timestamp;
    initialize_epoch(&mut ctx.accounts.epoch, next_epoch, now, ctx.bumps.epoch)?;
    initialize_leaderboard(
        &mut ctx.accounts.leaderboard,
        next_epoch,
        ctx.bumps.leaderboard,
    );
    ctx.accounts.protocol.current_epoch = next_epoch;

    emit!(EpochOpened {
        epoch: next_epoch,
        registration_ends_at: ctx.accounts.epoch.registration_ends_at,
    });
    Ok(())
}

pub fn register(ctx: Context<Register>, commitment: [u8; 32]) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        ctx.accounts.epoch.registration_open(now),
        GloryDumpError::WindowClosed
    );
    require!(commitment != [0u8; 32], GloryDumpError::InvalidCommitment);
    require!(
        ctx.accounts.epoch.participant_count < MAX_PARTICIPANTS,
        GloryDumpError::ParticipantCapReached
    );

    system_program::transfer(
        CpiContext::new(
            system_program::ID,
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.epoch.to_account_info(),
            },
        ),
        REGISTRATION_BOND_LAMPORTS,
    )?;

    let epoch_number = ctx.accounts.epoch.number;
    let owner = ctx.accounts.payer.key();
    *ctx.accounts.player = PlayerEpoch {
        epoch: epoch_number,
        owner,
        commitment,
        revealed: false,
        allocation_claimed: false,
        settled: false,
        bond_claimed: false,
        starting_allocation: 0,
        final_score: 0,
        dump_heat: HeatState::default(),
        absorb_heat: HeatState::default(),
        impact_ppm: 0,
        late_impact_ppm: 0,
        meaningful_actions: 0,
        distinct_opponents: 0,
        session_delegate: Pubkey::default(),
        session_expires_at: 0,
        session_actions_remaining: 0,
        badges: 0,
        bump: ctx.bumps.player,
    };

    initialize_lane(
        &mut ctx.accounts.lane_zero,
        epoch_number,
        owner,
        0,
        ctx.bumps.lane_zero,
    );
    initialize_lane(
        &mut ctx.accounts.lane_one,
        epoch_number,
        owner,
        1,
        ctx.bumps.lane_one,
    );
    initialize_lane(
        &mut ctx.accounts.lane_two,
        epoch_number,
        owner,
        2,
        ctx.bumps.lane_two,
    );
    initialize_lane(
        &mut ctx.accounts.lane_three,
        epoch_number,
        owner,
        3,
        ctx.bumps.lane_three,
    );

    let epoch = &mut ctx.accounts.epoch;
    epoch.participant_count = epoch
        .participant_count
        .checked_add(1)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    epoch.bonds_collected = epoch
        .bonds_collected
        .checked_add(REGISTRATION_BOND_LAMPORTS)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(PlayerRegistered {
        epoch: epoch_number,
        player: owner,
    });
    Ok(())
}

pub fn begin_reveal(ctx: Context<TransitionEpoch>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    require!(
        epoch.phase == Phase::Registration,
        GloryDumpError::WrongPhase
    );
    require!(now >= epoch.registration_ends_at, GloryDumpError::TooEarly);
    require!(
        epoch.participant_count >= MIN_REWARDED_PLAYERS,
        GloryDumpError::UnderfilledEpoch
    );

    epoch.phase = Phase::Reveal;
    epoch.reveal_ends_at = now
        .checked_add(REVEAL_SECONDS)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    emit!(RevealPhaseOpened {
        epoch: epoch.number,
        reveal_ends_at: epoch.reveal_ends_at,
    });
    Ok(())
}

pub fn reveal(ctx: Context<Reveal>, secret: [u8; 32]) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    require!(epoch.phase == Phase::Reveal, GloryDumpError::WrongPhase);
    require!(now < epoch.reveal_ends_at, GloryDumpError::WindowClosed);
    require!(
        !ctx.accounts.player.revealed,
        GloryDumpError::AlreadyRevealed
    );

    let expected = commitment_for(secret, ctx.accounts.owner.key(), epoch.number);
    require!(
        ctx.accounts.player.commitment == expected,
        GloryDumpError::InvalidCommitment
    );
    let contribution = entropy_contribution(secret, ctx.accounts.owner.key(), epoch.number);
    for (aggregate, byte) in epoch.entropy.iter_mut().zip(contribution) {
        *aggregate ^= byte;
    }
    ctx.accounts.player.revealed = true;
    epoch.revealed_count = epoch
        .revealed_count
        .checked_add(1)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(SecretRevealed {
        epoch: epoch.number,
        player: ctx.accounts.owner.key(),
    });
    Ok(())
}

pub fn seal_randomness(ctx: Context<TransitionEpoch>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    require!(epoch.phase == Phase::Reveal, GloryDumpError::WrongPhase);
    require!(now >= epoch.reveal_ends_at, GloryDumpError::TooEarly);
    require!(
        epoch.revealed_count >= MIN_REVEALED_PLAYERS,
        GloryDumpError::InsufficientReveals
    );

    let epoch_bytes = epoch.number.to_le_bytes();
    epoch.seed = hashv(&[
        SEED_DOMAIN,
        &epoch.entropy,
        &epoch_bytes,
        crate::ID.as_ref(),
    ])
    .to_bytes();
    epoch.phase = Phase::Planning;
    epoch.active_starts_at = now
        .checked_add(PLANNING_SECONDS)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    epoch.active_ends_at = epoch
        .active_starts_at
        .checked_add(ACTIVE_SECONDS)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(RandomnessSealed {
        epoch: epoch.number,
        seed: epoch.seed,
        active_starts_at: epoch.active_starts_at,
        active_ends_at: epoch.active_ends_at,
    });
    Ok(())
}

pub fn cancel_epoch(ctx: Context<TransitionEpoch>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    let cancellable = match epoch.phase {
        Phase::Registration => {
            now >= epoch.registration_ends_at && epoch.participant_count < MIN_REWARDED_PLAYERS
        }
        Phase::Reveal => now >= epoch.reveal_ends_at && epoch.revealed_count < MIN_REVEALED_PLAYERS,
        _ => false,
    };
    require!(cancellable, GloryDumpError::WrongPhase);
    epoch.phase = Phase::Cancelled;
    epoch.bond_claim_ends_at = now
        .checked_add(BOND_CLAIM_SECONDS)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(EpochCancelled {
        epoch: epoch.number,
        participant_count: epoch.participant_count,
        revealed_count: epoch.revealed_count,
    });
    Ok(())
}

pub fn claim_allocation(ctx: Context<ClaimAllocation>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        matches!(ctx.accounts.epoch.phase, Phase::Planning | Phase::Active),
        GloryDumpError::WrongPhase
    );
    require!(
        !ctx.accounts.player.allocation_claimed,
        GloryDumpError::AllocationAlreadyClaimed
    );

    let amount = allocation_for(&ctx.accounts.epoch, &ctx.accounts.player);
    apply_allocation(
        &ctx.accounts.epoch,
        &mut ctx.accounts.player,
        [
            &mut ctx.accounts.lane_zero,
            &mut ctx.accounts.lane_one,
            &mut ctx.accounts.lane_two,
            &mut ctx.accounts.lane_three,
        ],
        amount,
        now,
    )?;

    emit!(AllocationClaimed {
        epoch: ctx.accounts.epoch.number,
        player: ctx.accounts.owner.key(),
        amount,
        revealed: ctx.accounts.player.revealed,
    });
    Ok(())
}

pub fn begin_active(ctx: Context<TransitionEpoch>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    require!(epoch.phase == Phase::Planning, GloryDumpError::WrongPhase);
    require!(now >= epoch.active_starts_at, GloryDumpError::TooEarly);
    require!(now < epoch.active_ends_at, GloryDumpError::WindowClosed);
    epoch.phase = Phase::Active;
    emit!(ActivePlayStarted {
        epoch: epoch.number,
        ends_at: epoch.active_ends_at,
    });
    Ok(())
}

pub fn authorize_session(
    ctx: Context<ManageSession>,
    delegate: Pubkey,
    duration_seconds: i64,
    max_actions: u16,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &ctx.accounts.epoch;
    require!(
        matches!(epoch.phase, Phase::Planning | Phase::Active) && now < epoch.active_ends_at,
        GloryDumpError::WrongPhase
    );
    require!(
        delegate != Pubkey::default()
            && delegate != ctx.accounts.owner.key()
            && duration_seconds > 0
            && duration_seconds <= MAX_SESSION_SECONDS
            && max_actions > 0
            && max_actions <= MAX_SESSION_ACTIONS,
        GloryDumpError::InvalidSession
    );
    let requested_expiry = now
        .checked_add(duration_seconds)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    let expires_at = requested_expiry.min(epoch.active_ends_at);
    require!(expires_at > now, GloryDumpError::InvalidSession);

    let player = &mut ctx.accounts.player;
    player.session_delegate = delegate;
    player.session_expires_at = expires_at;
    player.session_actions_remaining = max_actions;
    emit!(SessionAuthorized {
        epoch: epoch.number,
        player: player.owner,
        delegate,
        expires_at,
        actions: max_actions,
    });
    Ok(())
}

pub fn revoke_session(ctx: Context<ManageSession>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    player.session_delegate = Pubkey::default();
    player.session_expires_at = 0;
    player.session_actions_remaining = 0;
    emit!(SessionRevoked {
        epoch: ctx.accounts.epoch.number,
        player: player.owner,
    });
    Ok(())
}

pub fn dump(
    ctx: Context<GameplayAction>,
    amount: u64,
    source_index: u8,
    target_index: u8,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_action_accounts(
        &ctx.accounts.epoch,
        &ctx.accounts.actor_player,
        &ctx.accounts.target_player,
        now,
    )?;
    ctx.accounts
        .actor_player
        .authorize_action(ctx.accounts.signer.key(), now)?;
    initialize_or_validate_rivalry(
        &mut ctx.accounts.rivalry,
        ctx.accounts.epoch.number,
        ctx.accounts.actor_player.owner,
        ctx.accounts.target_player.owner,
        ctx.accounts.signer.key(),
        ctx.bumps.rivalry,
    )?;
    let expected_lane = deterministic_target_lane(
        &ctx.accounts.epoch,
        ctx.accounts.actor_player.owner,
        ctx.accounts.target_player.owner,
        ctx.accounts.rivalry.action_count,
    );
    require_eq!(target_index, expected_lane, GloryDumpError::WrongTargetLane);

    let next_heat = rules::charge_heat(
        ctx.accounts.actor_player.dump_heat.into(),
        now,
        amount,
        ctx.accounts.actor_player.starting_allocation,
    )
    .map_err(crate::errors::map_rule_error)?;
    ctx.accounts
        .actor_lane
        .checkpoint(now, &ctx.accounts.epoch)?;
    ctx.accounts
        .target_lane
        .checkpoint(now, &ctx.accounts.epoch)?;
    require!(
        ctx.accounts.actor_lane.spendable(now) >= amount,
        GloryDumpError::InsufficientSpendableDump
    );

    let (landed, redirected, guard_after) = rules::apply_redirect(
        amount,
        ctx.accounts.target_lane.guard,
        ctx.accounts.target_lane.redirect_armed,
    );
    ctx.accounts.actor_lane.balance = ctx
        .accounts
        .actor_lane
        .balance
        .checked_sub(amount)
        .and_then(|value| value.checked_add(redirected))
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    ctx.accounts.target_lane.balance = ctx
        .accounts
        .target_lane
        .balance
        .checked_add(landed)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    ctx.accounts.target_lane.guard = guard_after;
    if redirected > 0 {
        ctx.accounts.target_lane.redirect_armed = false;
        ctx.accounts.target_lane.redirect_ready_at = now
            .checked_add(REDIRECT_REARM_SECONDS)
            .ok_or(GloryDumpError::ArithmeticOverflow)?;
        ctx.accounts.target_lane.redirected_volume = ctx
            .accounts
            .target_lane
            .redirected_volume
            .checked_add(redirected)
            .ok_or(GloryDumpError::ArithmeticOverflow)?;
    }

    ctx.accounts.actor_player.dump_heat = next_heat.into();
    record_action(
        &ctx.accounts.epoch,
        &mut ctx.accounts.actor_player,
        &mut ctx.accounts.rivalry,
        amount,
        now,
    )?;

    emit!(Dumped {
        epoch: ctx.accounts.epoch.number,
        actor: ctx.accounts.actor_player.owner,
        target: ctx.accounts.target_player.owner,
        attempted: amount,
        landed,
        redirected,
        source_lane: source_index,
        target_lane: target_index,
        dump_heat: ctx.accounts.actor_player.dump_heat.units,
    });
    Ok(())
}

pub fn absorb(
    ctx: Context<GameplayAction>,
    amount: u64,
    destination_index: u8,
    source_index: u8,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    validate_action_accounts(
        &ctx.accounts.epoch,
        &ctx.accounts.actor_player,
        &ctx.accounts.target_player,
        now,
    )?;
    ctx.accounts
        .actor_player
        .authorize_action(ctx.accounts.signer.key(), now)?;
    initialize_or_validate_rivalry(
        &mut ctx.accounts.rivalry,
        ctx.accounts.epoch.number,
        ctx.accounts.actor_player.owner,
        ctx.accounts.target_player.owner,
        ctx.accounts.signer.key(),
        ctx.bumps.rivalry,
    )?;

    let next_heat = rules::charge_heat(
        ctx.accounts.actor_player.absorb_heat.into(),
        now,
        amount,
        ctx.accounts.actor_player.starting_allocation,
    )
    .map_err(crate::errors::map_rule_error)?;
    ctx.accounts
        .actor_lane
        .checkpoint(now, &ctx.accounts.epoch)?;
    ctx.accounts
        .target_lane
        .checkpoint(now, &ctx.accounts.epoch)?;
    require!(
        ctx.accounts.target_lane.spendable(now) >= amount,
        GloryDumpError::InsufficientSpendableDump
    );

    ctx.accounts.target_lane.balance = ctx
        .accounts
        .target_lane
        .balance
        .checked_sub(amount)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    ctx.accounts.actor_lane.balance = ctx
        .accounts
        .actor_lane
        .balance
        .checked_add(amount)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    ctx.accounts.actor_lane.locked_amount = ctx
        .accounts
        .actor_lane
        .locked_amount
        .checked_add(amount)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    ctx.accounts.actor_lane.locked_until = ctx.accounts.actor_lane.locked_until.max(
        now.checked_add(ABSORB_LOCK_SECONDS)
            .ok_or(GloryDumpError::ArithmeticOverflow)?,
    );
    ctx.accounts.actor_lane.guard = rules::grant_guard(
        ctx.accounts.actor_lane.guard,
        amount,
        ctx.accounts.actor_player.starting_allocation,
    )
    .map_err(crate::errors::map_rule_error)?;

    ctx.accounts.actor_player.absorb_heat = next_heat.into();
    record_action(
        &ctx.accounts.epoch,
        &mut ctx.accounts.actor_player,
        &mut ctx.accounts.rivalry,
        amount,
        now,
    )?;

    emit!(Absorbed {
        epoch: ctx.accounts.epoch.number,
        actor: ctx.accounts.actor_player.owner,
        target: ctx.accounts.target_player.owner,
        amount,
        destination_lane: destination_index,
        source_lane: source_index,
        guard_after: ctx.accounts.actor_lane.guard,
        locked_until: ctx.accounts.actor_lane.locked_until,
        absorb_heat: ctx.accounts.actor_player.absorb_heat.units,
    });
    Ok(())
}

pub fn arm_redirect(ctx: Context<ArmRedirect>, lane_index: u8) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        (ctx.accounts.epoch.phase == Phase::Planning || ctx.accounts.epoch.gameplay_open(now))
            && now < ctx.accounts.epoch.active_ends_at,
        GloryDumpError::WrongPhase
    );
    require!(
        ctx.accounts.player.allocation_claimed,
        GloryDumpError::AllocationNotClaimed
    );
    ctx.accounts
        .player
        .authorize_action(ctx.accounts.signer.key(), now)?;
    require!(ctx.accounts.lane.guard > 0, GloryDumpError::NoGuard);
    require!(
        !ctx.accounts.lane.redirect_armed && now >= ctx.accounts.lane.redirect_ready_at,
        GloryDumpError::RedirectNotReady
    );
    ctx.accounts.lane.redirect_armed = true;
    emit!(RedirectArmed {
        epoch: ctx.accounts.epoch.number,
        player: ctx.accounts.player.owner,
        lane: lane_index,
        guard: ctx.accounts.lane.guard,
    });
    Ok(())
}

pub fn begin_settlement(ctx: Context<TransitionEpoch>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    require!(
        matches!(epoch.phase, Phase::Planning | Phase::Active),
        GloryDumpError::WrongPhase
    );
    require!(now >= epoch.active_ends_at, GloryDumpError::TooEarly);
    epoch.phase = Phase::Settling;
    emit!(SettlementStarted {
        epoch: epoch.number,
        participant_count: epoch.participant_count,
    });
    Ok(())
}

pub fn settle_player(ctx: Context<SettlePlayer>) -> Result<()> {
    require!(
        ctx.accounts.epoch.phase == Phase::Settling,
        GloryDumpError::WrongPhase
    );
    require!(!ctx.accounts.player.settled, GloryDumpError::AlreadySettled);

    let settlement_time = ctx.accounts.epoch.active_ends_at;
    if !ctx.accounts.player.allocation_claimed {
        let amount = allocation_for(&ctx.accounts.epoch, &ctx.accounts.player);
        apply_allocation(
            &ctx.accounts.epoch,
            &mut ctx.accounts.player,
            [
                &mut ctx.accounts.lane_zero,
                &mut ctx.accounts.lane_one,
                &mut ctx.accounts.lane_two,
                &mut ctx.accounts.lane_three,
            ],
            amount,
            settlement_time,
        )?;
    }

    ctx.accounts
        .lane_zero
        .checkpoint(settlement_time, &ctx.accounts.epoch)?;
    ctx.accounts
        .lane_one
        .checkpoint(settlement_time, &ctx.accounts.epoch)?;
    ctx.accounts
        .lane_two
        .checkpoint(settlement_time, &ctx.accounts.epoch)?;
    ctx.accounts
        .lane_three
        .checkpoint(settlement_time, &ctx.accounts.epoch)?;

    let cumulative = ctx
        .accounts
        .lane_zero
        .cumulative_weighted
        .checked_add(ctx.accounts.lane_one.cumulative_weighted)
        .and_then(|value| value.checked_add(ctx.accounts.lane_two.cumulative_weighted))
        .and_then(|value| value.checked_add(ctx.accounts.lane_three.cumulative_weighted))
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    let score = rules::weighted_score(
        cumulative,
        ctx.accounts.epoch.active_ends_at - ctx.accounts.epoch.active_starts_at,
    )
    .map_err(crate::errors::map_rule_error)?;
    let final_balance = ctx
        .accounts
        .lane_zero
        .balance
        .checked_add(ctx.accounts.lane_one.balance)
        .and_then(|value| value.checked_add(ctx.accounts.lane_two.balance))
        .and_then(|value| value.checked_add(ctx.accounts.lane_three.balance))
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    let redirected_volume = ctx
        .accounts
        .lane_zero
        .redirected_volume
        .checked_add(ctx.accounts.lane_one.redirected_volume)
        .and_then(|value| value.checked_add(ctx.accounts.lane_two.redirected_volume))
        .and_then(|value| value.checked_add(ctx.accounts.lane_three.redirected_volume))
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    let player = &mut ctx.accounts.player;
    player.final_score = score;
    player.badges |= rules::earned_badges(
        player.starting_allocation,
        final_balance,
        score,
        redirected_volume,
        player.late_impact_ppm,
    );
    player.settled = true;
    let eligible = player.eligible();
    if eligible {
        ctx.accounts.epoch.eligible_count = ctx
            .accounts
            .epoch
            .eligible_count
            .checked_add(1)
            .ok_or(GloryDumpError::ArithmeticOverflow)?;
        ctx.accounts.leaderboard.insert(WinnerEntry {
            player: player.owner,
            score,
            impact_ppm: player.impact_ppm,
            distinct_opponents: player.distinct_opponents,
            tie_breaker: tie_breaker(&ctx.accounts.epoch, player.owner),
            claimed: false,
        });
    }

    let epoch = &mut ctx.accounts.epoch;
    if epoch.settled_count == 0
        || score > epoch.worst_score
        || (score == epoch.worst_score && player.owner.to_bytes() > epoch.worst_player.to_bytes())
    {
        epoch.worst_score = score;
        epoch.worst_player = player.owner;
    }
    epoch.settled_count = epoch
        .settled_count
        .checked_add(1)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    let keeper_credit = &mut ctx.accounts.keeper_credit;
    if keeper_credit.epoch == 0 {
        keeper_credit.epoch = epoch.number;
        keeper_credit.keeper = ctx.accounts.keeper.key();
        keeper_credit.players_settled = 0;
        keeper_credit.claimed = false;
        keeper_credit.bump = ctx.bumps.keeper_credit;
    }
    keeper_credit.players_settled = keeper_credit
        .players_settled
        .checked_add(1)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    transfer_owned_lamports(
        &epoch.to_account_info(),
        &ctx.accounts.keeper.to_account_info(),
        SETTLEMENT_BOUNTY_LAMPORTS,
    )?;
    epoch.settlement_bounties_paid = epoch
        .settlement_bounties_paid
        .checked_add(SETTLEMENT_BOUNTY_LAMPORTS)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(PlayerSettled {
        epoch: epoch.number,
        player: player.owner,
        score,
        eligible,
        keeper: ctx.accounts.keeper.key(),
    });
    Ok(())
}

pub fn complete_epoch(ctx: Context<CompleteEpoch>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    require!(epoch.phase == Phase::Settling, GloryDumpError::WrongPhase);
    require_eq!(
        epoch.settled_count,
        epoch.participant_count,
        GloryDumpError::SettlementIncomplete
    );

    let winners = rules::winner_count(epoch.eligible_count);
    ctx.accounts.leaderboard.finalize(usize::from(winners));
    let remaining_supply = MAX_GLORY_SUPPLY.saturating_sub(ctx.accounts.protocol.glory_committed);
    let total_pool = rules::epoch_emission(epoch.number, epoch.eligible_count, remaining_supply)
        .map_err(crate::errors::map_rule_error)?;
    let raw_keeper_pool = rules::keeper_pool(total_pool).map_err(crate::errors::map_rule_error)?;
    let keeper_per_settlement = if epoch.participant_count == 0 {
        0
    } else {
        raw_keeper_pool / u64::from(epoch.participant_count)
    };
    let keeper_pool = keeper_per_settlement
        .checked_mul(u64::from(epoch.participant_count))
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    let player_pool = total_pool
        .checked_sub(keeper_pool)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    epoch.winner_count = winners;
    epoch.total_reward_pool = total_pool;
    epoch.player_reward_pool = player_pool;
    epoch.keeper_reward_pool = keeper_pool;
    epoch.phase = Phase::Complete;
    epoch.bond_claim_ends_at = now
        .checked_add(BOND_CLAIM_SECONDS)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    ctx.accounts.protocol.glory_committed = ctx
        .accounts
        .protocol
        .glory_committed
        .checked_add(total_pool)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(EpochCompleted {
        epoch: epoch.number,
        eligible_count: epoch.eligible_count,
        winner_count: winners,
        total_reward_pool: total_pool,
        player_reward_pool: player_pool,
        keeper_reward_pool: keeper_pool,
    });
    Ok(())
}

pub fn claim_player_reward(ctx: Context<ClaimPlayerReward>) -> Result<()> {
    require!(
        ctx.accounts.epoch.phase == Phase::Complete,
        GloryDumpError::WrongPhase
    );
    let owner = ctx.accounts.owner.key();
    let position = ctx
        .accounts
        .leaderboard
        .entries
        .iter()
        .position(|entry| entry.player == owner)
        .ok_or(GloryDumpError::NotWinner)?;
    let entry = &mut ctx.accounts.leaderboard.entries[position];
    require!(!entry.claimed, GloryDumpError::RewardAlreadyClaimed);
    let rank = u16::try_from(position).map_err(|_| GloryDumpError::ArithmeticOverflow)?;
    let amount = rules::rank_reward(
        ctx.accounts.epoch.player_reward_pool,
        ctx.accounts.epoch.winner_count,
        rank,
    )
    .map_err(crate::errors::map_rule_error)?;
    require!(
        ctx.accounts
            .glory_mint
            .supply
            .checked_add(amount)
            .is_some_and(|supply| supply <= MAX_GLORY_SUPPLY),
        GloryDumpError::ArithmeticOverflow
    );
    entry.claimed = true;
    mint_glory(
        &ctx.accounts.protocol,
        &ctx.accounts.glory_mint,
        &ctx.accounts.destination,
        amount,
    )?;

    emit!(PlayerRewardClaimed {
        epoch: ctx.accounts.epoch.number,
        player: owner,
        rank,
        amount,
    });
    Ok(())
}

pub fn claim_keeper_reward(ctx: Context<ClaimKeeperReward>) -> Result<()> {
    require!(
        ctx.accounts.epoch.phase == Phase::Complete,
        GloryDumpError::WrongPhase
    );
    require!(
        !ctx.accounts.keeper_credit.claimed,
        GloryDumpError::RewardAlreadyClaimed
    );
    let amount = rules::keeper_reward(
        ctx.accounts.epoch.keeper_reward_pool,
        ctx.accounts.keeper_credit.players_settled,
        ctx.accounts.epoch.participant_count,
    )
    .map_err(crate::errors::map_rule_error)?;
    require!(
        ctx.accounts
            .glory_mint
            .supply
            .checked_add(amount)
            .is_some_and(|supply| supply <= MAX_GLORY_SUPPLY),
        GloryDumpError::ArithmeticOverflow
    );
    ctx.accounts.keeper_credit.claimed = true;
    mint_glory(
        &ctx.accounts.protocol,
        &ctx.accounts.glory_mint,
        &ctx.accounts.destination,
        amount,
    )?;

    emit!(KeeperRewardClaimed {
        epoch: ctx.accounts.epoch.number,
        keeper: ctx.accounts.keeper.key(),
        players_settled: ctx.accounts.keeper_credit.players_settled,
        amount,
    });
    Ok(())
}

pub fn claim_bond(ctx: Context<ClaimBond>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &mut ctx.accounts.epoch;
    require!(
        matches!(epoch.phase, Phase::Complete | Phase::Cancelled),
        GloryDumpError::WrongPhase
    );
    require!(
        now <= epoch.bond_claim_ends_at,
        GloryDumpError::WindowClosed
    );
    require!(
        !ctx.accounts.player.bond_claimed,
        GloryDumpError::BondAlreadyClaimed
    );

    let amount = if epoch.phase == Phase::Cancelled {
        REGISTRATION_BOND_LAMPORTS
    } else {
        require!(
            ctx.accounts.player.eligible(),
            GloryDumpError::BondForfeited
        );
        REGISTRATION_BOND_LAMPORTS
            .checked_mul(BOND_REFUND_BPS)
            .and_then(|value| value.checked_div(BPS_DENOMINATOR))
            .ok_or(GloryDumpError::ArithmeticOverflow)?
    };
    ctx.accounts.player.bond_claimed = true;
    transfer_owned_lamports(
        &epoch.to_account_info(),
        &ctx.accounts.owner.to_account_info(),
        amount,
    )?;
    epoch.bond_refunds_paid = epoch
        .bond_refunds_paid
        .checked_add(amount)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(BondClaimed {
        epoch: epoch.number,
        player: ctx.accounts.owner.key(),
        amount,
    });
    Ok(())
}

pub fn refresh_badges(ctx: Context<RefreshBadges>) -> Result<()> {
    require!(
        ctx.accounts.epoch.phase == Phase::Complete && ctx.accounts.player.settled,
        GloryDumpError::WrongPhase
    );
    if ctx.accounts.epoch.worst_player == ctx.accounts.player.owner {
        ctx.accounts.player.badges |= BADGE_GARBAGE_EMPEROR;
    } else {
        ctx.accounts.player.badges &= !BADGE_GARBAGE_EMPEROR;
    }
    emit!(BadgesRefreshed {
        epoch: ctx.accounts.epoch.number,
        player: ctx.accounts.player.owner,
        badges: ctx.accounts.player.badges,
    });
    Ok(())
}

pub fn sweep_expired_bonds(ctx: Context<SweepExpiredBonds>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        matches!(
            ctx.accounts.old_epoch.phase,
            Phase::Complete | Phase::Cancelled
        ),
        GloryDumpError::WrongPhase
    );
    require!(
        ctx.accounts.old_epoch.number < ctx.accounts.current_epoch.number,
        GloryDumpError::InvalidNextEpoch
    );
    require!(
        ctx.accounts.old_epoch.bond_claim_ends_at > 0
            && now > ctx.accounts.old_epoch.bond_claim_ends_at,
        GloryDumpError::TooEarly
    );

    let amount = spendable_lamports(&ctx.accounts.old_epoch.to_account_info())?;
    transfer_owned_lamports(
        &ctx.accounts.old_epoch.to_account_info(),
        &ctx.accounts.current_epoch.to_account_info(),
        amount,
    )?;
    ctx.accounts.old_epoch.swept_excess_bonds = ctx
        .accounts
        .old_epoch
        .swept_excess_bonds
        .checked_add(amount)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;

    emit!(ExcessBondsSwept {
        from_epoch: ctx.accounts.old_epoch.number,
        to_epoch: ctx.accounts.current_epoch.number,
        amount,
    });
    Ok(())
}

pub fn close_rivalry(ctx: Context<CloseRivalry>) -> Result<()> {
    require!(
        matches!(ctx.accounts.epoch.phase, Phase::Complete | Phase::Cancelled),
        GloryDumpError::CleanupNotReady
    );
    Ok(())
}

pub fn close_player_accounts(ctx: Context<ClosePlayerAccounts>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let epoch = &ctx.accounts.epoch;
    let player = &ctx.accounts.player;
    require!(
        matches!(epoch.phase, Phase::Complete | Phase::Cancelled),
        GloryDumpError::CleanupNotReady
    );

    let claim_window_expired = epoch.bond_claim_ends_at > 0 && now > epoch.bond_claim_ends_at;
    if epoch.phase == Phase::Cancelled {
        require!(
            player.bond_claimed || claim_window_expired,
            GloryDumpError::CleanupNotReady
        );
        return Ok(());
    }

    require!(player.settled, GloryDumpError::CleanupNotReady);
    if player.eligible() && !player.bond_claimed && !claim_window_expired {
        return err!(GloryDumpError::CleanupNotReady);
    }
    if let Some(entry) = ctx
        .accounts
        .leaderboard
        .entries
        .iter()
        .find(|entry| entry.player == player.owner)
    {
        require!(entry.claimed, GloryDumpError::PendingReward);
    }
    Ok(())
}

pub fn close_keeper_credit(ctx: Context<CloseKeeperCredit>) -> Result<()> {
    require!(
        ctx.accounts.epoch.phase == Phase::Complete
            && (ctx.accounts.keeper_credit.claimed || ctx.accounts.epoch.keeper_reward_pool == 0),
        GloryDumpError::CleanupNotReady
    );
    Ok(())
}

fn initialize_epoch(epoch: &mut Account<Epoch>, number: u64, now: i64, bump: u8) -> Result<()> {
    **epoch = Epoch {
        number,
        phase: Phase::Registration,
        registration_started_at: now,
        registration_ends_at: now
            .checked_add(REGISTRATION_SECONDS)
            .ok_or(GloryDumpError::ArithmeticOverflow)?,
        reveal_ends_at: 0,
        active_starts_at: 0,
        active_ends_at: 0,
        entropy: [0u8; 32],
        seed: [0u8; 32],
        participant_count: 0,
        revealed_count: 0,
        settled_count: 0,
        eligible_count: 0,
        winner_count: 0,
        total_reward_pool: 0,
        player_reward_pool: 0,
        keeper_reward_pool: 0,
        worst_player: Pubkey::default(),
        worst_score: 0,
        bonds_collected: 0,
        bond_refunds_paid: 0,
        settlement_bounties_paid: 0,
        bond_claim_ends_at: 0,
        swept_excess_bonds: 0,
        bump,
    };
    Ok(())
}

fn initialize_leaderboard(leaderboard: &mut Account<Leaderboard>, epoch: u64, bump: u8) {
    **leaderboard = Leaderboard {
        epoch,
        entries: Vec::new(),
        bump,
    };
}

fn initialize_lane(
    lane: &mut Account<BalanceLane>,
    epoch: u64,
    owner: Pubkey,
    index: u8,
    bump: u8,
) {
    **lane = BalanceLane {
        epoch,
        owner,
        index,
        balance: 0,
        guard: 0,
        locked_amount: 0,
        locked_until: 0,
        redirect_armed: false,
        redirect_ready_at: 0,
        redirected_volume: 0,
        cumulative_weighted: 0,
        last_checkpoint_at: 0,
        bump,
    };
}

fn apply_allocation(
    epoch: &Epoch,
    player: &mut PlayerEpoch,
    mut lanes: [&mut BalanceLane; LANE_COUNT],
    amount: u64,
    checkpoint_at: i64,
) -> Result<()> {
    require!(
        !player.allocation_claimed,
        GloryDumpError::AllocationAlreadyClaimed
    );
    let split = rules::split_allocation(amount);
    // The burden exists for scoring from the active-phase boundary even when a
    // player claims late. During planning, clamp the checkpoint to that future
    // boundary so claiming cannot attempt a backwards timestamp update.
    let effective_checkpoint = allocation_checkpoint(epoch.active_starts_at, checkpoint_at);
    for (lane, balance) in lanes.iter_mut().zip(split) {
        lane.balance = balance;
        lane.last_checkpoint_at = epoch.active_starts_at;
        lane.checkpoint(effective_checkpoint, epoch)?;
    }

    player.starting_allocation = amount;
    player.allocation_claimed = true;
    player.dump_heat.updated_at = epoch.active_starts_at;
    player.absorb_heat.updated_at = epoch.active_starts_at;
    Ok(())
}

fn validate_action_accounts(
    epoch: &Epoch,
    actor: &PlayerEpoch,
    target: &PlayerEpoch,
    now: i64,
) -> Result<()> {
    require!(epoch.gameplay_open(now), GloryDumpError::WrongPhase);
    require!(actor.owner != target.owner, GloryDumpError::SelfAction);
    require!(
        actor.allocation_claimed && target.allocation_claimed,
        GloryDumpError::AllocationNotClaimed
    );
    require!(
        !actor.settled && !target.settled,
        GloryDumpError::WrongPhase
    );
    Ok(())
}

fn allocation_checkpoint(active_starts_at: i64, requested_at: i64) -> i64 {
    requested_at.max(active_starts_at)
}

fn initialize_or_validate_rivalry(
    rivalry: &mut Account<Rivalry>,
    epoch: u64,
    actor: Pubkey,
    target: Pubkey,
    rent_payer: Pubkey,
    bump: u8,
) -> Result<()> {
    if rivalry.epoch == 0 {
        **rivalry = Rivalry {
            epoch,
            actor,
            target,
            rent_payer,
            impact_credited_ppm: 0,
            action_count: 0,
            bump,
        };
        return Ok(());
    }
    require!(
        rivalry.epoch == epoch && rivalry.actor == actor && rivalry.target == target,
        GloryDumpError::InvalidRivalry
    );
    Ok(())
}

fn record_action(
    epoch: &Epoch,
    actor: &mut PlayerEpoch,
    rivalry: &mut Rivalry,
    amount: u64,
    now: i64,
) -> Result<()> {
    let was_new_opponent = rivalry.action_count == 0;
    let credit = rules::impact_credit(
        amount,
        actor.starting_allocation,
        rivalry.impact_credited_ppm,
    )
    .map_err(crate::errors::map_rule_error)?;
    rivalry.impact_credited_ppm = rivalry
        .impact_credited_ppm
        .checked_add(credit)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    rivalry.action_count = rivalry
        .action_count
        .checked_add(1)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    actor.impact_ppm = actor
        .impact_ppm
        .checked_add(credit)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    actor.meaningful_actions = actor
        .meaningful_actions
        .checked_add(1)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    if was_new_opponent && credit > 0 {
        actor.distinct_opponents = actor
            .distinct_opponents
            .checked_add(1)
            .ok_or(GloryDumpError::ArithmeticOverflow)?;
    }

    let duration = epoch
        .active_ends_at
        .checked_sub(epoch.active_starts_at)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    let late_window_starts = epoch
        .active_starts_at
        .checked_add(duration.saturating_mul(9) / 10)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    if now >= late_window_starts {
        actor.late_impact_ppm = actor
            .late_impact_ppm
            .checked_add(credit)
            .ok_or(GloryDumpError::ArithmeticOverflow)?;
    }
    Ok(())
}

fn allocation_for(epoch: &Epoch, player: &PlayerEpoch) -> u64 {
    if !player.revealed {
        return MAX_STARTING_DUMP;
    }
    let epoch_bytes = epoch.number.to_le_bytes();
    let digest = hashv(&[
        ALLOCATION_DOMAIN,
        &epoch.seed,
        player.owner.as_ref(),
        &epoch_bytes,
    ]);
    rules::allocation_from_entropy(u64_from_hash(digest.to_bytes()))
}

fn deterministic_target_lane(
    epoch: &Epoch,
    actor: Pubkey,
    target: Pubkey,
    action_count: u32,
) -> u8 {
    let epoch_bytes = epoch.number.to_le_bytes();
    let action_bytes = action_count.to_le_bytes();
    let digest = hashv(&[
        TARGET_LANE_DOMAIN,
        &epoch.seed,
        actor.as_ref(),
        target.as_ref(),
        &epoch_bytes,
        &action_bytes,
    ]);
    digest.to_bytes()[0] % LANE_COUNT as u8
}

fn tie_breaker(epoch: &Epoch, player: Pubkey) -> u64 {
    let epoch_bytes = epoch.number.to_le_bytes();
    u64_from_hash(hashv(&[TIE_BREAK_DOMAIN, &epoch.seed, player.as_ref(), &epoch_bytes]).to_bytes())
}

pub fn commitment_for(secret: [u8; 32], owner: Pubkey, epoch: u64) -> [u8; 32] {
    let epoch_bytes = epoch.to_le_bytes();
    hashv(&[COMMITMENT_DOMAIN, &secret, owner.as_ref(), &epoch_bytes]).to_bytes()
}

fn entropy_contribution(secret: [u8; 32], owner: Pubkey, epoch: u64) -> [u8; 32] {
    let epoch_bytes = epoch.to_le_bytes();
    hashv(&[CONTRIBUTION_DOMAIN, &secret, owner.as_ref(), &epoch_bytes]).to_bytes()
}

fn u64_from_hash(bytes: [u8; 32]) -> u64 {
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(prefix)
}

fn spendable_lamports(account: &AccountInfo<'_>) -> Result<u64> {
    let rent_floor = Rent::get()?.minimum_balance(account.data_len());
    Ok(account.lamports().saturating_sub(rent_floor))
}

fn transfer_owned_lamports(
    source: &AccountInfo<'_>,
    destination: &AccountInfo<'_>,
    amount: u64,
) -> Result<()> {
    require!(
        spendable_lamports(source)? >= amount,
        GloryDumpError::InsufficientEpochFunds
    );
    let source_after = source
        .lamports()
        .checked_sub(amount)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    let destination_after = destination
        .lamports()
        .checked_add(amount)
        .ok_or(GloryDumpError::ArithmeticOverflow)?;
    **source.try_borrow_mut_lamports()? = source_after;
    **destination.try_borrow_mut_lamports()? = destination_after;
    Ok(())
}

fn mint_glory<'info>(
    protocol: &Account<'info, Protocol>,
    mint: &Account<'info, anchor_spl::token::Mint>,
    destination: &Account<'info, anchor_spl::token::TokenAccount>,
    amount: u64,
) -> Result<()> {
    let bump = [protocol.bump];
    let signer_seeds: &[&[u8]] = &[b"protocol", &bump];
    token::mint_to(
        CpiContext::new_with_signer(
            token::ID,
            MintTo {
                mint: mint.to_account_info(),
                to: destination.to_account_info(),
                authority: protocol.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commitment_vector_is_stable_for_clients() {
        assert_eq!(
            commitment_for([7u8; 32], Pubkey::new_from_array([9u8; 32]), 42),
            [
                0x0e, 0x12, 0xda, 0x79, 0x71, 0xe9, 0x42, 0xfb, 0x57, 0x77, 0xca, 0x6d, 0x16, 0xdb,
                0xce, 0xf7, 0x8d, 0x9c, 0xfc, 0xcf, 0x08, 0xf5, 0xa0, 0xfd, 0x4d, 0x4c, 0x20, 0x15,
                0x49, 0xcd, 0x7a, 0xa9,
            ]
        );
    }

    #[test]
    fn planning_claims_checkpoint_at_the_active_boundary() {
        assert_eq!(allocation_checkpoint(10_000, 9_000), 10_000);
        assert_eq!(allocation_checkpoint(10_000, 12_000), 12_000);
    }
}
