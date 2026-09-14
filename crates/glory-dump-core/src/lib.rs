#![no_std]

use core::cmp::Ordering;

pub const BPS_DENOMINATOR: u64 = 10_000;
pub const PPM_DENOMINATOR: u64 = 1_000_000;

pub const DUMP_TIER_SIZE: u64 = 1_000_000_000;
pub const DUMP_TIER_COUNT: u64 = 10;
pub const MAX_STARTING_DUMP: u64 = DUMP_TIER_SIZE * DUMP_TIER_COUNT;

pub const LANE_COUNT: usize = 4;
// The v3 five-percent winner set fits in one sub-10 KiB Anchor account. A
// future one-world arena needs a different root/proof settlement architecture;
// silently raising this bound would put every settlement behind one oversized
// account and is not a scaling plan.
pub const MAX_PARTICIPANTS: u32 = 2_560;
pub const MAX_WINNERS: usize = 128;

#[cfg(not(feature = "test-fast"))]
pub const REGISTRATION_SECONDS: i64 = 7 * 24 * 60 * 60;
#[cfg(feature = "test-fast")]
pub const REGISTRATION_SECONDS: i64 = 30;

#[cfg(not(feature = "test-fast"))]
pub const REVEAL_SECONDS: i64 = 12 * 60 * 60;
#[cfg(feature = "test-fast")]
pub const REVEAL_SECONDS: i64 = 12;

#[cfg(not(feature = "test-fast"))]
pub const PLANNING_SECONDS: i64 = 60 * 60;
#[cfg(feature = "test-fast")]
pub const PLANNING_SECONDS: i64 = 5;

#[cfg(not(feature = "test-fast"))]
pub const ACTIVE_SECONDS: i64 = 30 * 24 * 60 * 60;
#[cfg(feature = "test-fast")]
pub const ACTIVE_SECONDS: i64 = 45;

pub const REGISTRATION_BOND_LAMPORTS: u64 = 2_000_000;
pub const BOND_REFUND_BPS: u64 = 8_000;
pub const SETTLEMENT_BOUNTY_LAMPORTS: u64 =
    REGISTRATION_BOND_LAMPORTS * (BPS_DENOMINATOR - BOND_REFUND_BPS) / BPS_DENOMINATOR;

pub const MIN_REWARDED_PLAYERS: u32 = 20;
pub const MIN_REVEALED_PLAYERS: u32 = 2;
pub const WINNER_BPS: u64 = 500;
pub const KEEPER_REWARD_BPS: u64 = 100;

pub const GLORY_DECIMALS: u8 = 6;
pub const GLORY_SCALE: u64 = 1_000_000;
pub const MAX_GLORY_SUPPLY: u64 = 10_000_000 * GLORY_SCALE;
pub const BASE_EPOCH_EMISSION: u64 = 100_000 * GLORY_SCALE;
pub const REFERENCE_PLAYERS: u32 = 100;
pub const EPOCHS_PER_ERA: u64 = 12;
pub const MIN_EMISSION_MULTIPLIER_MILLI: u64 = 250;
pub const MAX_EMISSION_MULTIPLIER_MILLI: u64 = 4_000;

pub const HEAT_CAP: u32 = 10_000;
#[cfg(not(feature = "test-fast"))]
pub const HEAT_RECOVERY_SECONDS: i64 = 6 * 60 * 60;
#[cfg(feature = "test-fast")]
pub const HEAT_RECOVERY_SECONDS: i64 = 4;
pub const MIN_ACTION_BPS: u64 = 10;
pub const GUARD_CONVERSION_BPS: u64 = 5_000;
pub const TOTAL_GUARD_CAP_BPS: u64 = 2_500;
#[cfg(not(feature = "test-fast"))]
pub const ABSORB_LOCK_SECONDS: i64 = 6 * 60 * 60;
#[cfg(feature = "test-fast")]
pub const ABSORB_LOCK_SECONDS: i64 = 4;

#[cfg(not(feature = "test-fast"))]
pub const REDIRECT_REARM_SECONDS: i64 = 15 * 60;
#[cfg(feature = "test-fast")]
pub const REDIRECT_REARM_SECONDS: i64 = 2;
pub const PAIR_IMPACT_CAP_PPM: u64 = 500_000;

pub const BADGE_ESCAPE_ARTIST: u64 = 1 << 0;
pub const BADGE_HUMAN_SHIELD: u64 = 1 << 1;
pub const BADGE_RICOCHET: u64 = 1 << 2;
pub const BADGE_LAST_LAUGH: u64 = 1 << 3;
pub const BADGE_GARBAGE_EMPEROR: u64 = 1 << 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Heat {
    pub units: u32,
    pub updated_at: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Standing {
    pub player: [u8; 32],
    pub score: u64,
    pub impact_ppm: u64,
    pub distinct_opponents: u16,
    pub tie_breaker: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleError {
    ArithmeticOverflow,
    InvalidAmount,
    InvalidDuration,
    HeatCapacityExceeded,
}

pub fn minimum_action(starting_allocation: u64) -> Result<u64, RuleError> {
    mul_div_ceil(starting_allocation, MIN_ACTION_BPS, BPS_DENOMINATOR)
}

pub fn heat_at(heat: Heat, now: i64) -> u32 {
    if now <= heat.updated_at || heat.units == 0 {
        return heat.units;
    }

    let elapsed = now.saturating_sub(heat.updated_at);
    if elapsed >= HEAT_RECOVERY_SECONDS {
        return 0;
    }

    let decay = (u64::try_from(elapsed).unwrap_or(0) * u64::from(HEAT_CAP)
        / u64::try_from(HEAT_RECOVERY_SECONDS).unwrap_or(1)) as u32;
    heat.units.saturating_sub(decay)
}

pub fn charge_heat(
    heat: Heat,
    now: i64,
    amount: u64,
    starting_allocation: u64,
) -> Result<Heat, RuleError> {
    if amount < minimum_action(starting_allocation)? || starting_allocation == 0 {
        return Err(RuleError::InvalidAmount);
    }

    let current = heat_at(heat, now);
    let cost = mul_div_ceil(amount, u64::from(HEAT_CAP), starting_allocation)?;
    let cost = u32::try_from(cost).map_err(|_| RuleError::HeatCapacityExceeded)?;
    let units = current
        .checked_add(cost)
        .ok_or(RuleError::HeatCapacityExceeded)?;
    if units > HEAT_CAP {
        return Err(RuleError::HeatCapacityExceeded);
    }

    Ok(Heat {
        units,
        updated_at: now,
    })
}

pub fn seconds_until_heat_available(
    heat: Heat,
    now: i64,
    amount: u64,
    starting_allocation: u64,
) -> Result<i64, RuleError> {
    if starting_allocation == 0 {
        return Err(RuleError::InvalidAmount);
    }
    let current = heat_at(heat, now);
    let cost = mul_div_ceil(amount, u64::from(HEAT_CAP), starting_allocation)?;
    if cost > u64::from(HEAT_CAP) {
        return Err(RuleError::HeatCapacityExceeded);
    }
    let needed_decay = current.saturating_add(cost as u32).saturating_sub(HEAT_CAP);
    if needed_decay == 0 {
        return Ok(0);
    }
    let seconds = mul_div_ceil(
        u64::from(needed_decay),
        u64::try_from(HEAT_RECOVERY_SECONDS).map_err(|_| RuleError::InvalidDuration)?,
        u64::from(HEAT_CAP),
    )?;
    i64::try_from(seconds).map_err(|_| RuleError::ArithmeticOverflow)
}

pub fn weighted_area_scaled(
    from_elapsed: i64,
    to_elapsed: i64,
    duration: i64,
) -> Result<u128, RuleError> {
    if duration <= 0 || from_elapsed < 0 || to_elapsed < from_elapsed {
        return Err(RuleError::InvalidDuration);
    }
    let duration = u128::try_from(duration).map_err(|_| RuleError::InvalidDuration)?;
    let from = u128::try_from(from_elapsed)
        .map_err(|_| RuleError::InvalidDuration)?
        .min(duration);
    let to = u128::try_from(to_elapsed)
        .map_err(|_| RuleError::InvalidDuration)?
        .min(duration);
    let duration_squared = duration
        .checked_mul(duration)
        .ok_or(RuleError::ArithmeticOverflow)?;
    let linear = (to - from)
        .checked_mul(duration_squared)
        .ok_or(RuleError::ArithmeticOverflow)?;
    let cubic = cube(to)?
        .checked_sub(cube(from)?)
        .ok_or(RuleError::ArithmeticOverflow)?;
    linear
        .checked_add(cubic)
        .ok_or(RuleError::ArithmeticOverflow)
}

pub fn checkpoint_weighted_balance(
    cumulative: u128,
    balance: u64,
    from_timestamp: i64,
    to_timestamp: i64,
    epoch_start: i64,
    epoch_end: i64,
) -> Result<u128, RuleError> {
    if epoch_end <= epoch_start || to_timestamp < from_timestamp {
        return Err(RuleError::InvalidDuration);
    }
    let from = from_timestamp.clamp(epoch_start, epoch_end) - epoch_start;
    let to = to_timestamp.clamp(epoch_start, epoch_end) - epoch_start;
    let area = weighted_area_scaled(from, to, epoch_end - epoch_start)?;
    cumulative
        .checked_add(
            u128::from(balance)
                .checked_mul(area)
                .ok_or(RuleError::ArithmeticOverflow)?,
        )
        .ok_or(RuleError::ArithmeticOverflow)
}

pub fn weighted_score(cumulative: u128, duration: i64) -> Result<u64, RuleError> {
    if duration <= 0 {
        return Err(RuleError::InvalidDuration);
    }
    let duration = u128::try_from(duration).map_err(|_| RuleError::InvalidDuration)?;
    let denominator = cube(duration)?
        .checked_mul(2)
        .ok_or(RuleError::ArithmeticOverflow)?;
    u64::try_from(cumulative / denominator).map_err(|_| RuleError::ArithmeticOverflow)
}

pub fn allocation_from_entropy(entropy: u64) -> u64 {
    (entropy % DUMP_TIER_COUNT + 1) * DUMP_TIER_SIZE
}

pub fn split_allocation(total: u64) -> [u64; LANE_COUNT] {
    let base = total / LANE_COUNT as u64;
    let mut lanes = [base; LANE_COUNT];
    let remainder = total % LANE_COUNT as u64;
    let mut index = 0usize;
    while index < remainder as usize {
        lanes[index] += 1;
        index += 1;
    }
    lanes
}

pub fn lane_guard_cap(starting_allocation: u64) -> Result<u64, RuleError> {
    let total = mul_div_floor(starting_allocation, TOTAL_GUARD_CAP_BPS, BPS_DENOMINATOR)?;
    Ok(total / LANE_COUNT as u64)
}

pub fn grant_guard(
    current_guard: u64,
    absorbed: u64,
    starting_allocation: u64,
) -> Result<u64, RuleError> {
    let grant = mul_div_floor(absorbed, GUARD_CONVERSION_BPS, BPS_DENOMINATOR)?;
    Ok(current_guard
        .checked_add(grant)
        .ok_or(RuleError::ArithmeticOverflow)?
        .min(lane_guard_cap(starting_allocation)?))
}

pub fn apply_redirect(incoming: u64, guard: u64, armed: bool) -> (u64, u64, u64) {
    if !armed || guard == 0 || incoming == 0 {
        return (incoming, 0, guard);
    }
    let redirected = incoming.min(guard);
    (incoming - redirected, redirected, guard - redirected)
}

pub fn impact_credit(
    amount: u64,
    starting_allocation: u64,
    pair_credit_so_far: u64,
) -> Result<u64, RuleError> {
    if starting_allocation == 0 {
        return Err(RuleError::InvalidAmount);
    }
    let normalized = mul_div_floor(amount, PPM_DENOMINATOR, starting_allocation)?;
    Ok(normalized.min(PAIR_IMPACT_CAP_PPM.saturating_sub(pair_credit_so_far)))
}

pub fn winner_count(eligible_players: u32) -> u16 {
    if eligible_players < MIN_REWARDED_PLAYERS {
        return 0;
    }
    let count = (u64::from(eligible_players) * WINNER_BPS).div_ceil(BPS_DENOMINATOR);
    count.clamp(1, MAX_WINNERS as u64) as u16
}

pub fn epoch_emission(
    epoch_number: u64,
    eligible_players: u32,
    remaining_supply: u64,
) -> Result<u64, RuleError> {
    if eligible_players < MIN_REWARDED_PLAYERS || remaining_supply == 0 {
        return Ok(0);
    }
    let era = epoch_number.saturating_sub(1) / EPOCHS_PER_ERA;
    let base = if era >= 64 {
        0
    } else {
        BASE_EPOCH_EMISSION >> era
    };
    if base == 0 {
        return Ok(0);
    }

    let scaled = u128::from(eligible_players)
        .checked_mul(1_000_000)
        .ok_or(RuleError::ArithmeticOverflow)?
        / u128::from(REFERENCE_PLAYERS);
    let multiplier_milli = integer_sqrt(scaled).clamp(
        u128::from(MIN_EMISSION_MULTIPLIER_MILLI),
        u128::from(MAX_EMISSION_MULTIPLIER_MILLI),
    );
    let pool = u128::from(base)
        .checked_mul(multiplier_milli)
        .ok_or(RuleError::ArithmeticOverflow)?
        / 1_000;
    Ok(u64::try_from(pool)
        .map_err(|_| RuleError::ArithmeticOverflow)?
        .min(remaining_supply))
}

pub fn keeper_pool(total_pool: u64) -> Result<u64, RuleError> {
    mul_div_floor(total_pool, KEEPER_REWARD_BPS, BPS_DENOMINATOR)
}

pub fn player_pool(total_pool: u64) -> Result<u64, RuleError> {
    total_pool
        .checked_sub(keeper_pool(total_pool)?)
        .ok_or(RuleError::ArithmeticOverflow)
}

pub fn rank_reward(
    total_player_pool: u64,
    winners: u16,
    zero_based_rank: u16,
) -> Result<u64, RuleError> {
    if winners == 0 || zero_based_rank >= winners {
        return Err(RuleError::InvalidAmount);
    }
    let winners = u128::from(winners);
    let rank = u128::from(zero_based_rank);
    let weight = winners
        .checked_mul(2)
        .and_then(|value| value.checked_sub(rank))
        .ok_or(RuleError::ArithmeticOverflow)?;
    let total_weight = winners
        .checked_mul(
            winners
                .checked_mul(3)
                .and_then(|value| value.checked_add(1))
                .ok_or(RuleError::ArithmeticOverflow)?,
        )
        .and_then(|value| value.checked_div(2))
        .ok_or(RuleError::ArithmeticOverflow)?;
    let ordinary = u128::from(total_player_pool)
        .checked_mul(weight)
        .ok_or(RuleError::ArithmeticOverflow)?
        / total_weight;

    let reward = if zero_based_rank == 0 {
        let mut distributed = 0u128;
        let mut current = 0u128;
        while current < winners {
            let current_weight = winners * 2 - current;
            distributed = distributed
                .checked_add(
                    u128::from(total_player_pool)
                        .checked_mul(current_weight)
                        .ok_or(RuleError::ArithmeticOverflow)?
                        / total_weight,
                )
                .ok_or(RuleError::ArithmeticOverflow)?;
            current += 1;
        }
        ordinary
            .checked_add(u128::from(total_player_pool) - distributed)
            .ok_or(RuleError::ArithmeticOverflow)?
    } else {
        ordinary
    };

    u64::try_from(reward).map_err(|_| RuleError::ArithmeticOverflow)
}

pub fn keeper_reward(
    total_keeper_pool: u64,
    settled_by_keeper: u32,
    participant_count: u32,
) -> Result<u64, RuleError> {
    if participant_count == 0 {
        return Err(RuleError::InvalidAmount);
    }
    mul_div_floor(
        total_keeper_pool,
        u64::from(settled_by_keeper),
        u64::from(participant_count),
    )
}

pub fn earned_badges(
    starting_allocation: u64,
    final_balance: u64,
    final_score: u64,
    redirected_volume: u64,
    late_impact_ppm: u64,
) -> u64 {
    if starting_allocation == 0 {
        return 0;
    }

    let mut badges = 0u64;
    if starting_allocation >= 8 * DUMP_TIER_SIZE
        && u128::from(final_score) * u128::from(BPS_DENOMINATOR)
            <= u128::from(starting_allocation) * 2_500
    {
        badges |= BADGE_ESCAPE_ARTIST;
    }
    if u128::from(final_balance) * u128::from(BPS_DENOMINATOR)
        >= u128::from(starting_allocation) * 15_000
    {
        badges |= BADGE_HUMAN_SHIELD;
    }
    if u128::from(redirected_volume) * u128::from(BPS_DENOMINATOR)
        >= u128::from(starting_allocation) * 1_000
    {
        badges |= BADGE_RICOCHET;
    }
    if late_impact_ppm >= 100_000 {
        badges |= BADGE_LAST_LAUGH;
    }
    badges
}

pub fn compare_standings(left: &Standing, right: &Standing) -> Ordering {
    left.score
        .cmp(&right.score)
        .then_with(|| right.impact_ppm.cmp(&left.impact_ppm))
        .then_with(|| right.distinct_opponents.cmp(&left.distinct_opponents))
        .then_with(|| left.tie_breaker.cmp(&right.tie_breaker))
        .then_with(|| left.player.cmp(&right.player))
}

pub fn is_better(left: &Standing, right: &Standing) -> bool {
    compare_standings(left, right) == Ordering::Less
}

fn cube(value: u128) -> Result<u128, RuleError> {
    value
        .checked_mul(value)
        .and_then(|squared| squared.checked_mul(value))
        .ok_or(RuleError::ArithmeticOverflow)
}

fn mul_div_floor(left: u64, right: u64, denominator: u64) -> Result<u64, RuleError> {
    if denominator == 0 {
        return Err(RuleError::InvalidAmount);
    }
    let value = u128::from(left)
        .checked_mul(u128::from(right))
        .ok_or(RuleError::ArithmeticOverflow)?
        / u128::from(denominator);
    u64::try_from(value).map_err(|_| RuleError::ArithmeticOverflow)
}

fn mul_div_ceil(left: u64, right: u64, denominator: u64) -> Result<u64, RuleError> {
    if denominator == 0 {
        return Err(RuleError::InvalidAmount);
    }
    let numerator = u128::from(left)
        .checked_mul(u128::from(right))
        .ok_or(RuleError::ArithmeticOverflow)?;
    let denominator = u128::from(denominator);
    let value = numerator.div_ceil(denominator);
    u64::try_from(value).map_err(|_| RuleError::ArithmeticOverflow)
}

fn integer_sqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    let mut low = 1u128;
    let mut high = value.min(u128::from(u64::MAX));
    let mut answer = 1u128;
    while low <= high {
        let mid = low + (high - low) / 2;
        if mid <= value / mid {
            answer = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    answer
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn allocation_has_exactly_ten_discrete_tiers() {
        let allocations: Vec<u64> = (0..100).map(allocation_from_entropy).collect();
        assert_eq!(allocations.iter().copied().min(), Some(DUMP_TIER_SIZE));
        assert_eq!(allocations.iter().copied().max(), Some(MAX_STARTING_DUMP));
        for tier in 1..=DUMP_TIER_COUNT {
            assert!(allocations.contains(&(tier * DUMP_TIER_SIZE)));
        }
    }

    #[test]
    fn allocation_split_is_conservative() {
        for tier in 1..=DUMP_TIER_COUNT {
            let total = tier * DUMP_TIER_SIZE;
            assert_eq!(split_allocation(total).iter().sum::<u64>(), total);
        }
    }

    #[test]
    fn weighted_score_is_exact_for_a_constant_balance() {
        let start = 1_000;
        let end = start + ACTIVE_SECONDS;
        let cumulative =
            checkpoint_weighted_balance(0, DUMP_TIER_SIZE, start, end, start, end).unwrap();
        assert_eq!(
            weighted_score(cumulative, ACTIVE_SECONDS).unwrap(),
            DUMP_TIER_SIZE
        );
    }

    #[test]
    fn a_last_percent_dump_does_not_erase_the_epoch() {
        let held_until = ACTIVE_SECONDS * 99 / 100;
        let cumulative =
            checkpoint_weighted_balance(0, DUMP_TIER_SIZE, 0, held_until, 0, ACTIVE_SECONDS)
                .unwrap();
        let score = weighted_score(cumulative, ACTIVE_SECONDS).unwrap();
        assert!((980_000_000..=981_000_000).contains(&score), "{score}");
    }

    #[test]
    fn later_half_has_more_weight_without_zeroing_the_opening() {
        let half = ACTIVE_SECONDS / 2;
        let first = weighted_area_scaled(0, half, ACTIVE_SECONDS).unwrap();
        let second = weighted_area_scaled(half, ACTIVE_SECONDS, ACTIVE_SECONDS).unwrap();
        assert_eq!(first * 11, second * 5);
    }

    #[test]
    fn heat_is_split_invariant_and_recovers() {
        let start = DUMP_TIER_SIZE;
        let one = charge_heat(Heat::default(), 0, start / 10, start).unwrap();
        let mut split = Heat::default();
        for _ in 0..10 {
            split = charge_heat(split, 0, start / 100, start).unwrap();
        }
        assert_eq!(one.units, split.units);
        assert_eq!(heat_at(one, HEAT_RECOVERY_SECONDS), 0);
    }

    #[test]
    fn heat_rejects_more_than_one_starting_allocation_at_once() {
        let result = charge_heat(Heat::default(), 0, DUMP_TIER_SIZE + 1, DUMP_TIER_SIZE);
        assert_eq!(result, Err(RuleError::HeatCapacityExceeded));
    }

    #[test]
    fn guard_is_lossy_and_capped() {
        let start = DUMP_TIER_SIZE;
        let cap = lane_guard_cap(start).unwrap();
        assert_eq!(cap * LANE_COUNT as u64, start / 4);
        assert_eq!(grant_guard(0, 10_000_000, start).unwrap(), 5_000_000);
        assert_eq!(grant_guard(cap, start, start).unwrap(), cap);
    }

    #[test]
    fn redirect_conserves_dump_and_consumes_guard() {
        let (landed, returned, remaining) = apply_redirect(100, 40, true);
        assert_eq!((landed, returned, remaining), (60, 40, 0));
        assert_eq!(landed + returned, 100);
    }

    #[test]
    fn pair_impact_has_diminishing_hard_cap() {
        let start = DUMP_TIER_SIZE;
        let first = impact_credit(start, start, 0).unwrap();
        assert_eq!(first, PAIR_IMPACT_CAP_PPM);
        assert_eq!(impact_credit(start, start, first).unwrap(), 0);
    }

    #[test]
    fn participant_scaling_is_sublinear_and_clamped() {
        let supply = MAX_GLORY_SUPPLY;
        let at_reference = epoch_emission(1, 100, supply).unwrap();
        let four_times = epoch_emission(1, 400, supply).unwrap();
        let sixteen_times = epoch_emission(1, 1_600, supply).unwrap();
        assert_eq!(at_reference, BASE_EPOCH_EMISSION);
        assert_eq!(four_times, BASE_EPOCH_EMISSION * 2);
        assert_eq!(sixteen_times, BASE_EPOCH_EMISSION * 4);
        assert_eq!(epoch_emission(1, 5_120, supply).unwrap(), sixteen_times);
    }

    #[test]
    fn emissions_halve_each_twelve_epochs_and_respect_supply() {
        let supply = MAX_GLORY_SUPPLY;
        assert_eq!(
            epoch_emission(13, 100, supply).unwrap(),
            BASE_EPOCH_EMISSION / 2
        );
        assert_eq!(epoch_emission(25, 100, 7).unwrap(), 7);
    }

    #[test]
    fn winner_count_is_ceil_five_percent_with_a_safe_cap() {
        assert_eq!(winner_count(19), 0);
        assert_eq!(winner_count(20), 1);
        assert_eq!(winner_count(21), 2);
        assert_eq!(winner_count(MAX_PARTICIPANTS), MAX_WINNERS as u16);
    }

    #[test]
    fn rank_rewards_conserve_the_entire_player_pool() {
        for winners in 1..=MAX_WINNERS as u16 {
            let pool = 99_000_003u64;
            let distributed: u64 = (0..winners)
                .map(|rank| rank_reward(pool, winners, rank).unwrap())
                .sum();
            assert_eq!(distributed, pool, "winner count {winners}");
        }
    }

    #[test]
    fn rank_curve_rewards_effort_without_crushing_last_place() {
        let pool = 1_000_000;
        let first = rank_reward(pool, 10, 0).unwrap();
        let last = rank_reward(pool, 10, 9).unwrap();
        let average = pool / 10;
        assert!(first > average);
        assert!(last < average);
        assert!(first < last * 2);
    }

    #[test]
    fn standing_order_uses_impact_then_distinct_then_randomness() {
        let base = Standing {
            player: [1; 32],
            score: 10,
            impact_ppm: 100,
            distinct_opponents: 2,
            tie_breaker: 7,
        };
        let mut candidate = base;
        candidate.impact_ppm = 101;
        assert!(is_better(&candidate, &base));
        candidate = base;
        candidate.distinct_opponents = 3;
        assert!(is_better(&candidate, &base));
        candidate = base;
        candidate.tie_breaker = 6;
        assert!(is_better(&candidate, &base));
    }

    #[test]
    fn achievement_thresholds_are_deterministic() {
        let start = 8 * DUMP_TIER_SIZE;
        let badges = earned_badges(start, start * 3 / 2, start / 4, start / 10, 100_000);
        assert_eq!(
            badges,
            BADGE_ESCAPE_ARTIST | BADGE_HUMAN_SHIELD | BADGE_RICOCHET | BADGE_LAST_LAUGH
        );
        assert_eq!(earned_badges(0, 0, 0, 0, 0), 0);
    }
}
