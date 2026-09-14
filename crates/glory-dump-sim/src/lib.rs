use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Display, Formatter};

use glory_dump_core as rules;
use serde::{Deserialize, Serialize};

pub const REPORT_SCHEMA: &str = "glory-dump-simulation-report-v2";
pub const GLOBAL_SCALE_REPORT_SCHEMA: &str = "glory-dump-global-scale-report-v1";
const LATE_WINDOW_BPS: u64 = 9_000;
const MAX_DECISION_ROUNDS: u32 = 1_440;
const MAX_EXPERIMENTAL_POPULATION: u32 = 100_000;
const REFERENCE_ACTION_CAPACITY: u64 = 5_500_000_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Random,
    Inactive,
    SmallPacketDumper,
    GreedyDumper,
    LeaderHunter,
    QuietSmallFry,
    GuardBuilder,
    RedirectBluffer,
    LastWindow,
    ReciprocalAlliance,
    RotatingCoalition,
    SacrificialSybil,
    SelfDogpile,
    BribedCoalition,
    RevealWithholder,
}

impl Strategy {
    pub const ALL: [Self; 15] = [
        Self::Random,
        Self::Inactive,
        Self::SmallPacketDumper,
        Self::GreedyDumper,
        Self::LeaderHunter,
        Self::QuietSmallFry,
        Self::GuardBuilder,
        Self::RedirectBluffer,
        Self::LastWindow,
        Self::ReciprocalAlliance,
        Self::RotatingCoalition,
        Self::SacrificialSybil,
        Self::SelfDogpile,
        Self::BribedCoalition,
        Self::RevealWithholder,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::Inactive => "inactive",
            Self::SmallPacketDumper => "small_packet_dumper",
            Self::GreedyDumper => "greedy_dumper",
            Self::LeaderHunter => "leader_hunter",
            Self::QuietSmallFry => "quiet_small_fry",
            Self::GuardBuilder => "guard_builder",
            Self::RedirectBluffer => "redirect_bluffer",
            Self::LastWindow => "last_window",
            Self::ReciprocalAlliance => "reciprocal_alliance",
            Self::RotatingCoalition => "rotating_coalition",
            Self::SacrificialSybil => "sacrificial_sybil",
            Self::SelfDogpile => "self_dogpile",
            Self::BribedCoalition => "bribed_coalition",
            Self::RevealWithholder => "reveal_withholder",
        }
    }
}

impl Display for Strategy {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scenario {
    #[default]
    Mixed,
    RandomOnly,
    LeaderHunt,
    GuardHeavy,
    LastWindow,
    SybilStress,
}

impl Display for Scenario {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Mixed => "mixed",
            Self::RandomOnly => "random_only",
            Self::LeaderHunt => "leader_hunt",
            Self::GuardHeavy => "guard_heavy",
            Self::LastWindow => "last_window",
            Self::SybilStress => "sybil_stress",
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalScoring {
    #[default]
    CanonicalLateWeighted,
    EqualChapters,
    MildChapters,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCapacity {
    #[default]
    CanonicalHeat,
    ChapterStamina,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionModel {
    #[default]
    Sequential,
    ScheduledBatch,
}

/// Simulator-only mechanics. The default is intentionally inert so loading an
/// older JSON config continues to exercise the committed v3 rules.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ResearchRules {
    pub temporal_scoring: TemporalScoring,
    pub action_capacity: ActionCapacity,
    pub execution_model: ExecutionModel,
    pub chapter_count: u8,
    pub chapter_capacity_bps: u64,
    pub chapter_start_weight_bps: u64,
    pub chapter_carry_bps: u64,
    pub max_intents_per_window: u8,
    pub target_absorb_relief_cap_bps_per_chapter: u64,
    pub target_inbound_cap_bps_per_chapter: u64,
    pub coalition_coordination_cost_lamports_per_helper_epoch: u64,
}

impl Default for ResearchRules {
    fn default() -> Self {
        Self {
            temporal_scoring: TemporalScoring::CanonicalLateWeighted,
            action_capacity: ActionCapacity::CanonicalHeat,
            execution_model: ExecutionModel::Sequential,
            chapter_count: 6,
            chapter_capacity_bps: rules::BPS_DENOMINATOR,
            chapter_start_weight_bps: 5_000,
            chapter_carry_bps: rules::BPS_DENOMINATOR,
            max_intents_per_window: 1,
            target_absorb_relief_cap_bps_per_chapter: 0,
            target_inbound_cap_bps_per_chapter: 0,
            coalition_coordination_cost_lamports_per_helper_epoch: 0,
        }
    }
}

impl ResearchRules {
    /// First v4 candidate. These numbers are hypotheses to test, not promoted
    /// protocol constants.
    #[must_use]
    pub fn v4_candidate() -> Self {
        Self {
            temporal_scoring: TemporalScoring::EqualChapters,
            action_capacity: ActionCapacity::ChapterStamina,
            execution_model: ExecutionModel::ScheduledBatch,
            chapter_count: 6,
            chapter_capacity_bps: 10_000,
            chapter_start_weight_bps: 1_500,
            chapter_carry_bps: rules::BPS_DENOMINATOR,
            max_intents_per_window: 4,
            target_absorb_relief_cap_bps_per_chapter: 2_500,
            target_inbound_cap_bps_per_chapter: 0,
            coalition_coordination_cost_lamports_per_helper_epoch: 100_000,
        }
    }

    #[must_use]
    pub fn is_canonical(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RuleTuning {
    pub minimum_action_bps: u64,
    pub heat_recovery_seconds: i64,
    pub guard_conversion_bps: u64,
    pub total_guard_cap_bps: u64,
    pub absorb_lock_seconds: i64,
    pub redirect_rearm_seconds: i64,
}

impl Default for RuleTuning {
    fn default() -> Self {
        Self {
            minimum_action_bps: rules::MIN_ACTION_BPS,
            heat_recovery_seconds: rules::HEAT_RECOVERY_SECONDS,
            guard_conversion_bps: rules::GUARD_CONVERSION_BPS,
            total_guard_cap_bps: rules::TOTAL_GUARD_CAP_BPS,
            absorb_lock_seconds: rules::ABSORB_LOCK_SECONDS,
            redirect_rearm_seconds: rules::REDIRECT_REARM_SECONDS,
        }
    }
}

impl RuleTuning {
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationConfig {
    pub population: u32,
    pub epochs: u32,
    pub seed: u64,
    pub scenario: Scenario,
    pub decision_interval_seconds: i64,
    pub information_delay_seconds: i64,
    pub rpc_failure_bps: u64,
    pub sybil_fraction_bps: u64,
    pub estimated_signature_fee_lamports: u64,
    pub hypothetical_glory_price_lamports: u64,
    pub tuning: RuleTuning,
    #[serde(default)]
    pub research: ResearchRules,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            population: 100,
            epochs: 100,
            seed: 0x474c_4f52_5944_554d,
            scenario: Scenario::Mixed,
            decision_interval_seconds: 6 * 60 * 60,
            information_delay_seconds: 0,
            rpc_failure_bps: 0,
            sybil_fraction_bps: 1_000,
            estimated_signature_fee_lamports: 5_000,
            hypothetical_glory_price_lamports: 0,
            tuning: RuleTuning::default(),
            research: ResearchRules::default(),
        }
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<(), SimulationError> {
        let maximum_population = if self.research.is_canonical() {
            rules::MAX_PARTICIPANTS
        } else {
            MAX_EXPERIMENTAL_POPULATION
        };
        if !(2..=maximum_population).contains(&self.population) {
            return Err(SimulationError::InvalidConfig(format!(
                "population must be between 2 and {}",
                maximum_population
            )));
        }
        if self.epochs == 0 {
            return Err(SimulationError::InvalidConfig(
                "epochs must be positive".into(),
            ));
        }
        if self.decision_interval_seconds <= 0 {
            return Err(SimulationError::InvalidConfig(
                "decision interval must be positive".into(),
            ));
        }
        if self.information_delay_seconds < 0 {
            return Err(SimulationError::InvalidConfig(
                "information delay cannot be negative".into(),
            ));
        }
        for (label, value) in [
            ("rpc failure", self.rpc_failure_bps),
            ("sybil fraction", self.sybil_fraction_bps),
            ("minimum action", self.tuning.minimum_action_bps),
            ("guard conversion", self.tuning.guard_conversion_bps),
            ("guard cap", self.tuning.total_guard_cap_bps),
        ] {
            if value > rules::BPS_DENOMINATOR {
                return Err(SimulationError::InvalidConfig(format!(
                    "{label} basis points cannot exceed {}",
                    rules::BPS_DENOMINATOR
                )));
            }
        }
        if self.tuning.heat_recovery_seconds <= 0
            || self.tuning.absorb_lock_seconds < 0
            || self.tuning.redirect_rearm_seconds < 0
        {
            return Err(SimulationError::InvalidConfig(
                "rule durations must be non-negative and Heat recovery must be positive".into(),
            ));
        }
        if !(2..=12).contains(&self.research.chapter_count) {
            return Err(SimulationError::InvalidConfig(
                "research chapter count must be between 2 and 12".into(),
            ));
        }
        if self.research.chapter_capacity_bps == 0
            || self.research.chapter_capacity_bps > 100_000
            || self.research.chapter_start_weight_bps > rules::BPS_DENOMINATOR
            || self.research.chapter_carry_bps > rules::BPS_DENOMINATOR
            || self.research.target_absorb_relief_cap_bps_per_chapter > rules::BPS_DENOMINATOR
            || self.research.target_inbound_cap_bps_per_chapter > 50_000
            || !(1..=16).contains(&self.research.max_intents_per_window)
        {
            return Err(SimulationError::InvalidConfig(
                "invalid research capacity, carry, relief-cap, or batch setting".into(),
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn decision_rounds(&self) -> u32 {
        u32::try_from((rules::ACTIVE_SECONDS / self.decision_interval_seconds).max(1))
            .unwrap_or(MAX_DECISION_ROUNDS)
            .min(MAX_DECISION_ROUNDS)
    }
}

#[derive(Debug)]
pub enum SimulationError {
    InvalidConfig(String),
    Arithmetic(&'static str),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl Display for SimulationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(message) => {
                write!(formatter, "invalid simulation config: {message}")
            }
            Self::Arithmetic(message) => {
                write!(formatter, "simulation arithmetic failed: {message}")
            }
            Self::Io(error) => Display::fmt(error, formatter),
            Self::Json(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for SimulationError {}

impl From<std::io::Error> for SimulationError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for SimulationError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct GlobalScaleConfig {
    pub populations: Vec<u64>,
    pub action_interval_seconds: u64,
    pub registration_window_seconds: u64,
    pub reveal_window_seconds: u64,
    /// Deliberately a scenario input, not a claim about Solana's maximum TPS.
    pub modeled_direct_actions_per_second: u64,
    pub observed_program_action_compute_units: u64,
    pub intents_per_aggregate_proof: u64,
    pub signed_intent_bytes: u64,
}

impl Default for GlobalScaleConfig {
    fn default() -> Self {
        Self {
            populations: vec![50_000, 1_000_000, 1_000_000_000, 7_000_000_000],
            action_interval_seconds: 3 * 24 * 60 * 60,
            registration_window_seconds: 7 * 24 * 60 * 60,
            reveal_window_seconds: 12 * 60 * 60,
            modeled_direct_actions_per_second: 1_000,
            observed_program_action_compute_units: 29_264,
            intents_per_aggregate_proof: 10_000,
            signed_intent_bytes: 128,
        }
    }
}

impl GlobalScaleConfig {
    pub fn validate(&self) -> Result<(), SimulationError> {
        if self.populations.is_empty() || self.populations.iter().any(|value| *value < 2) {
            return Err(SimulationError::InvalidConfig(
                "global-scale populations must contain values of at least two".into(),
            ));
        }
        if self.action_interval_seconds == 0
            || self.registration_window_seconds == 0
            || self.reveal_window_seconds == 0
            || self.modeled_direct_actions_per_second == 0
            || self.observed_program_action_compute_units == 0
            || self.intents_per_aggregate_proof == 0
            || self.signed_intent_bytes == 0
        {
            return Err(SimulationError::InvalidConfig(
                "global-scale rates, windows, batch size, and byte size must be positive".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleDisposition {
    DirectWithinModeledBudget,
    RequiresAggregatedSettlement,
}

#[derive(Clone, Debug, Serialize)]
pub struct GlobalScaleRow {
    pub population: u64,
    pub scheduled_intents_per_round: u64,
    pub average_intents_per_second: f64,
    pub registration_transactions_per_second: f64,
    pub reveal_transactions_per_second: f64,
    pub modeled_direct_action_budget_per_second: u64,
    pub modeled_direct_within_budget: bool,
    pub minimum_direct_action_interval_seconds: u64,
    pub minimum_direct_action_interval_days: f64,
    pub direct_compute_units_per_second: f64,
    pub aggregate_proofs_per_round: u64,
    pub aggregate_proof_transactions_per_second: f64,
    pub signed_intent_data_bytes_per_round: u128,
    pub disposition: ScaleDisposition,
}

#[derive(Clone, Debug, Serialize)]
pub struct GlobalScaleReport {
    pub schema: &'static str,
    pub model_status: &'static str,
    pub global_targeting_preserved: bool,
    pub config: GlobalScaleConfig,
    pub rows: Vec<GlobalScaleRow>,
    pub caveats: Vec<&'static str>,
}

pub fn model_global_scale(config: GlobalScaleConfig) -> Result<GlobalScaleReport, SimulationError> {
    config.validate()?;
    let mut rows = Vec::with_capacity(config.populations.len());
    for population in config.populations.iter().copied() {
        let intents_per_second = population as f64 / config.action_interval_seconds as f64;
        let direct_within = intents_per_second <= config.modeled_direct_actions_per_second as f64;
        let minimum_interval = population.div_ceil(config.modeled_direct_actions_per_second);
        let aggregate_proofs = population.div_ceil(config.intents_per_aggregate_proof);
        let signed_intent_data = u128::from(population)
            .checked_mul(u128::from(config.signed_intent_bytes))
            .ok_or(SimulationError::Arithmetic("global signed-intent data"))?;
        rows.push(GlobalScaleRow {
            population,
            scheduled_intents_per_round: population,
            average_intents_per_second: intents_per_second,
            registration_transactions_per_second: population as f64
                / config.registration_window_seconds as f64,
            reveal_transactions_per_second: population as f64 / config.reveal_window_seconds as f64,
            modeled_direct_action_budget_per_second: config.modeled_direct_actions_per_second,
            modeled_direct_within_budget: direct_within,
            minimum_direct_action_interval_seconds: minimum_interval,
            minimum_direct_action_interval_days: minimum_interval as f64 / 86_400.0,
            direct_compute_units_per_second: intents_per_second
                * config.observed_program_action_compute_units as f64,
            aggregate_proofs_per_round: aggregate_proofs,
            aggregate_proof_transactions_per_second: aggregate_proofs as f64
                / config.action_interval_seconds as f64,
            signed_intent_data_bytes_per_round: signed_intent_data,
            disposition: if direct_within {
                ScaleDisposition::DirectWithinModeledBudget
            } else {
                ScaleDisposition::RequiresAggregatedSettlement
            },
        });
    }
    Ok(GlobalScaleReport {
        schema: GLOBAL_SCALE_REPORT_SCHEMA,
        model_status: "analytical_research_model_only",
        global_targeting_preserved: true,
        config,
        rows,
        caveats: vec![
            "The direct-action budget is a configurable planning assumption, not measured Mainnet capacity.",
            "Aggregate-proof rows estimate transaction-count reduction only; no prover, circuit, sequencer, or data-availability layer exists in this repository.",
            "Every player may address every other player, but global addressing does not remove hot-target contention, censorship, proof, or indexing risks.",
            "Registration and reveal bursts require their own aggregation and data-availability design at very large populations.",
        ],
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulationReport {
    pub schema: &'static str,
    pub config: SimulationConfig,
    pub canonical_rules: bool,
    pub implementation_status: &'static str,
    pub action_fee_model: &'static str,
    pub totals: TotalsReport,
    pub strategies: Vec<StrategyReport>,
    pub starting_tiers: Vec<TierReport>,
    pub balance_signals: Vec<BalanceSignal>,
    pub epoch_samples: Vec<EpochSummary>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct TotalsReport {
    pub simulated_epochs: u32,
    pub player_epochs: u64,
    pub eligible_player_epochs: u64,
    pub rewarded_player_epochs: u64,
    pub inactive_epochs: u32,
    pub dump_actions: u64,
    pub absorb_actions: u64,
    pub redirect_arms: u64,
    pub failed_heat_actions: u64,
    pub failed_capacity_actions: u64,
    pub failed_relief_cap_actions: u64,
    pub failed_inbound_cap_actions: u64,
    pub failed_balance_actions: u64,
    pub simulated_rpc_failures: u64,
    pub dump_moved: u128,
    pub redirected_dump: u128,
    pub guard_created: u128,
    pub guard_stranded: u128,
    pub late_actions: u64,
    pub final_window_winner_turnover_rate: f64,
    pub final_chapter_winner_turnover_rate: f64,
    pub mean_chapter_boundary_turnover_rates: Vec<f64>,
    pub mean_target_concentration: f64,
    pub glory_emitted: u64,
    pub glory_gini: f64,
    pub estimated_action_fees_lamports: u128,
    pub hypothetical_glory_value_lamports: u128,
}

#[derive(Clone, Debug, Serialize)]
pub struct StrategyReport {
    pub strategy: Strategy,
    pub player_epochs: u64,
    pub eligible_player_epochs: u64,
    pub wins: u64,
    pub win_rate: f64,
    pub winner_share: f64,
    pub relative_win_advantage: f64,
    pub controller_player_epochs: u64,
    pub controller_wins: u64,
    pub controller_win_rate: f64,
    pub controller_relative_win_advantage: f64,
    pub controller_glory_earned: u64,
    pub mean_rank_percentile: f64,
    pub mean_final_score_ratio: f64,
    pub mean_actions: f64,
    pub glory_earned: u64,
    pub estimated_fees_lamports: u128,
    pub estimated_bond_cost_lamports: u128,
    pub coalition_helper_player_epochs: u64,
    pub estimated_coordination_cost_lamports: u128,
    pub estimated_total_cost_lamports: u128,
    pub break_even_glory_price_lamports: Option<u128>,
    pub hypothetical_glory_value_lamports: u128,
    pub hypothetical_net_before_rent_lamports: i128,
    pub hypothetical_return_on_cost: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TierReport {
    pub tier_billions: u64,
    pub player_epochs: u64,
    pub eligible_player_epochs: u64,
    pub wins: u64,
    pub win_rate: f64,
    pub all_player_win_rate: f64,
    pub mean_final_score_ratio: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct BalanceSignal {
    pub severity: SignalSeverity,
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalSeverity {
    Information,
    Watch,
    Risk,
}

#[derive(Clone, Debug, Serialize)]
pub struct EpochSummary {
    pub epoch: u64,
    pub eligible_players: u32,
    pub winners: u16,
    pub actions: u64,
    pub target_concentration: f64,
    pub final_window_turnover_rate: f64,
    pub final_chapter_turnover_rate: f64,
    pub chapter_boundary_turnover_rates: Vec<f64>,
    pub glory_emitted: u64,
    pub top_strategy: Option<Strategy>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Lane {
    balance: u64,
    guard: u64,
    locked_amount: u64,
    locked_until: i64,
    redirect_armed: bool,
    redirect_ready_at: i64,
    redirected_volume: u64,
    cumulative_weighted: u128,
    last_checkpoint_at: i64,
}

impl Lane {
    fn checkpoint(&mut self, now: i64, research: &ResearchRules) -> Result<(), SimulationError> {
        self.cumulative_weighted = checkpoint_balance_area(
            self.cumulative_weighted,
            self.balance,
            self.last_checkpoint_at,
            now,
            research,
        )?;
        self.last_checkpoint_at = now.clamp(0, rules::ACTIVE_SECONDS);
        if now >= self.locked_until {
            self.locked_amount = 0;
        }
        Ok(())
    }

    fn spendable(&self, now: i64) -> u64 {
        if now >= self.locked_until {
            self.balance
        } else {
            self.balance.saturating_sub(self.locked_amount)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ChapterBudget {
    initialized: bool,
    chapter: u8,
    dump_remaining: u64,
    absorb_remaining: u64,
}

#[derive(Clone, Copy, Debug)]
enum CapacityChannel {
    Dump,
    Absorb,
}

#[derive(Clone, Debug)]
struct Player {
    id: usize,
    strategy: Strategy,
    starting_allocation: u64,
    lanes: [Lane; rules::LANE_COUNT],
    dump_heat: rules::Heat,
    absorb_heat: rules::Heat,
    chapter_budget: ChapterBudget,
    absorb_relief_initialized: bool,
    absorb_relief_chapter: u8,
    absorb_relief_used: u64,
    inbound_initialized: bool,
    inbound_chapter: u8,
    inbound_used: u64,
    revealed: bool,
    meaningful_actions: u32,
    impact_ppm: u64,
    late_impact_ppm: u64,
    opponents: BTreeSet<usize>,
}

impl Player {
    fn new(id: usize, strategy: Strategy, allocation: u64, revealed: bool) -> Self {
        let split = rules::split_allocation(allocation);
        let mut lanes = [Lane::default(); rules::LANE_COUNT];
        for (lane, balance) in lanes.iter_mut().zip(split) {
            lane.balance = balance;
        }
        Self {
            id,
            strategy,
            starting_allocation: allocation,
            lanes,
            dump_heat: rules::Heat::default(),
            absorb_heat: rules::Heat::default(),
            chapter_budget: ChapterBudget::default(),
            absorb_relief_initialized: false,
            absorb_relief_chapter: 0,
            absorb_relief_used: 0,
            inbound_initialized: false,
            inbound_chapter: 0,
            inbound_used: 0,
            revealed,
            meaningful_actions: 0,
            impact_ppm: 0,
            late_impact_ppm: 0,
            opponents: BTreeSet::new(),
        }
    }

    fn guard(&self) -> u64 {
        self.lanes.iter().map(|lane| lane.guard).sum()
    }

    fn current_score(&self, now: i64, research: &ResearchRules) -> Result<u64, SimulationError> {
        let mut cumulative = 0u128;
        for lane in &self.lanes {
            let projected = checkpoint_balance_area(
                lane.cumulative_weighted,
                lane.balance,
                lane.last_checkpoint_at,
                now,
                research,
            )?;
            cumulative = cumulative
                .checked_add(projected)
                .ok_or(SimulationError::Arithmetic("projected score sum"))?;
        }
        score_from_area(cumulative, research)
    }

    fn standing(
        &self,
        now: i64,
        epoch_seed: u64,
        research: &ResearchRules,
    ) -> Result<rules::Standing, SimulationError> {
        Ok(rules::Standing {
            player: player_key(self.id),
            score: self.current_score(now, research)?,
            impact_ppm: self.impact_ppm,
            distinct_opponents: u16::try_from(self.opponents.len()).unwrap_or(u16::MAX),
            tie_breaker: mix64(epoch_seed ^ self.id as u64),
        })
    }

    fn projected_no_action_standing(
        &self,
        now: i64,
        epoch_seed: u64,
        research: &ResearchRules,
    ) -> Result<rules::Standing, SimulationError> {
        let mut cumulative = 0u128;
        for lane in &self.lanes {
            let at_snapshot = checkpoint_balance_area(
                lane.cumulative_weighted,
                lane.balance,
                lane.last_checkpoint_at,
                now,
                research,
            )?;
            let at_end = checkpoint_balance_area(
                at_snapshot,
                lane.balance,
                now,
                rules::ACTIVE_SECONDS,
                research,
            )?;
            cumulative = cumulative
                .checked_add(at_end)
                .ok_or(SimulationError::Arithmetic("projected no-action score sum"))?;
        }
        Ok(rules::Standing {
            player: player_key(self.id),
            score: score_from_area(cumulative, research)?,
            impact_ppm: self.impact_ppm,
            distinct_opponents: u16::try_from(self.opponents.len()).unwrap_or(u16::MAX),
            tie_breaker: mix64(epoch_seed ^ self.id as u64),
        })
    }

    fn eligible(&self) -> bool {
        self.revealed && self.meaningful_actions > 0
    }

    fn refresh_chapter_budget(&mut self, now: i64, research: &ResearchRules) {
        if research.action_capacity != ActionCapacity::ChapterStamina {
            return;
        }
        let chapter = chapter_at(now, research.chapter_count);
        let basis =
            chapter_capacity_basis(self.starting_allocation, research.chapter_start_weight_bps);
        let allowance = mul_bps(basis, research.chapter_capacity_bps);
        if !self.chapter_budget.initialized {
            self.chapter_budget = ChapterBudget {
                initialized: true,
                chapter,
                dump_remaining: allowance,
                absorb_remaining: allowance,
            };
            return;
        }
        if chapter <= self.chapter_budget.chapter {
            return;
        }
        let crossed_one = chapter == self.chapter_budget.chapter.saturating_add(1);
        let dump_prior = if crossed_one {
            self.chapter_budget.dump_remaining.min(allowance)
        } else {
            allowance
        };
        let absorb_prior = if crossed_one {
            self.chapter_budget.absorb_remaining.min(allowance)
        } else {
            allowance
        };
        self.chapter_budget.chapter = chapter;
        self.chapter_budget.dump_remaining =
            allowance.saturating_add(mul_bps(dump_prior, research.chapter_carry_bps));
        self.chapter_budget.absorb_remaining =
            allowance.saturating_add(mul_bps(absorb_prior, research.chapter_carry_bps));
    }

    fn available_capacity(
        &mut self,
        channel: CapacityChannel,
        now: i64,
        research: &ResearchRules,
    ) -> u64 {
        if research.action_capacity == ActionCapacity::CanonicalHeat {
            return u64::MAX;
        }
        self.refresh_chapter_budget(now, research);
        match channel {
            CapacityChannel::Dump => self.chapter_budget.dump_remaining,
            CapacityChannel::Absorb => self.chapter_budget.absorb_remaining,
        }
    }

    fn charge_capacity(
        &mut self,
        channel: CapacityChannel,
        amount: u64,
        research: &ResearchRules,
    ) -> Result<(), SimulationError> {
        if research.action_capacity == ActionCapacity::CanonicalHeat {
            return Ok(());
        }
        let remaining = match channel {
            CapacityChannel::Dump => &mut self.chapter_budget.dump_remaining,
            CapacityChannel::Absorb => &mut self.chapter_budget.absorb_remaining,
        };
        *remaining = remaining
            .checked_sub(amount)
            .ok_or(SimulationError::Arithmetic("chapter capacity"))?;
        Ok(())
    }

    fn available_absorb_relief(&mut self, now: i64, research: &ResearchRules) -> u64 {
        let cap_bps = research.target_absorb_relief_cap_bps_per_chapter;
        if cap_bps == 0 {
            return u64::MAX;
        }
        let chapter = chapter_at(now, research.chapter_count);
        if !self.absorb_relief_initialized || self.absorb_relief_chapter != chapter {
            self.absorb_relief_initialized = true;
            self.absorb_relief_chapter = chapter;
            self.absorb_relief_used = 0;
        }
        mul_bps(self.starting_allocation, cap_bps).saturating_sub(self.absorb_relief_used)
    }

    fn available_inbound(&mut self, now: i64, research: &ResearchRules) -> u64 {
        let cap_bps = research.target_inbound_cap_bps_per_chapter;
        if cap_bps == 0 {
            return u64::MAX;
        }
        let chapter = chapter_at(now, research.chapter_count);
        if !self.inbound_initialized || self.inbound_chapter != chapter {
            self.inbound_initialized = true;
            self.inbound_chapter = chapter;
            self.inbound_used = 0;
        }
        mul_bps(self.starting_allocation, cap_bps).saturating_sub(self.inbound_used)
    }
}

#[derive(Clone, Copy, Debug)]
enum Intent {
    Dump { target: usize, percent: u64 },
    Absorb { target: usize, percent: u64 },
    Arm,
}

#[derive(Default)]
struct EpochCounters {
    dump_actions: u64,
    absorb_actions: u64,
    redirect_arms: u64,
    failed_heat_actions: u64,
    failed_capacity_actions: u64,
    failed_relief_cap_actions: u64,
    failed_inbound_cap_actions: u64,
    failed_balance_actions: u64,
    simulated_rpc_failures: u64,
    dump_moved: u128,
    redirected_dump: u128,
    guard_created: u128,
    late_actions: u64,
    incoming_by_player: Vec<u128>,
}

impl EpochCounters {
    fn action_count(&self) -> u64 {
        self.dump_actions + self.absorb_actions + self.redirect_arms
    }
}

struct EpochResult {
    summary: EpochSummary,
    player_rows: Vec<PlayerResult>,
    counters: EpochCounters,
    guard_stranded: u128,
    reward_by_player: Vec<u64>,
}

struct PlayerResult {
    strategy: Strategy,
    coalition_controller: bool,
    starting_allocation: u64,
    final_score: u64,
    final_rank: usize,
    actions: u32,
    eligible: bool,
    winner: bool,
}

#[derive(Default)]
struct StrategyAccumulator {
    player_epochs: u64,
    eligible: u64,
    wins: u64,
    controller_player_epochs: u64,
    controller_wins: u64,
    controller_glory: u64,
    rank_percentile_sum: f64,
    score_ratio_sum: f64,
    actions: u64,
    glory: u64,
}

#[derive(Default)]
struct TierAccumulator {
    player_epochs: u64,
    eligible: u64,
    wins: u64,
    score_ratio_sum: f64,
}

pub fn simulate(config: SimulationConfig) -> Result<SimulationReport, SimulationError> {
    config.validate()?;
    let canonical_rules = config.tuning.is_canonical() && config.research.is_canonical();
    let mut rng = SplitMix64::new(config.seed);
    let mut totals = TotalsReport::default();
    let mut strategies: BTreeMap<Strategy, StrategyAccumulator> = BTreeMap::new();
    let mut tiers: BTreeMap<u64, TierAccumulator> = BTreeMap::new();
    let mut epoch_samples = Vec::with_capacity(config.epochs as usize);
    let mut reward_by_player = vec![0u64; config.population as usize];
    let mut glory_committed = 0u64;
    let mut turnover_sum = 0.0;
    let mut final_chapter_turnover_sum = 0.0;
    let mut chapter_turnover_sums =
        vec![0.0; usize::from(config.research.chapter_count.saturating_sub(1))];
    let mut concentration_sum = 0.0;

    for epoch_index in 0..config.epochs {
        let epoch_number = u64::from(epoch_index) + 1;
        let result = simulate_epoch(
            &config,
            epoch_number,
            rng.next_u64(),
            rules::MAX_GLORY_SUPPLY.saturating_sub(glory_committed),
        )?;
        glory_committed = glory_committed
            .checked_add(result.summary.glory_emitted)
            .ok_or(SimulationError::Arithmetic("GLORY commitment"))?;
        totals.simulated_epochs += 1;
        totals.player_epochs += u64::from(config.population);
        totals.eligible_player_epochs += u64::from(result.summary.eligible_players);
        totals.rewarded_player_epochs += u64::from(result.summary.winners);
        if result.summary.eligible_players < rules::MIN_REWARDED_PLAYERS {
            totals.inactive_epochs += 1;
        }
        totals.dump_actions += result.counters.dump_actions;
        totals.absorb_actions += result.counters.absorb_actions;
        totals.redirect_arms += result.counters.redirect_arms;
        totals.failed_heat_actions += result.counters.failed_heat_actions;
        totals.failed_capacity_actions += result.counters.failed_capacity_actions;
        totals.failed_relief_cap_actions += result.counters.failed_relief_cap_actions;
        totals.failed_inbound_cap_actions += result.counters.failed_inbound_cap_actions;
        totals.failed_balance_actions += result.counters.failed_balance_actions;
        totals.simulated_rpc_failures += result.counters.simulated_rpc_failures;
        totals.dump_moved += result.counters.dump_moved;
        totals.redirected_dump += result.counters.redirected_dump;
        totals.guard_created += result.counters.guard_created;
        totals.guard_stranded += result.guard_stranded;
        totals.late_actions += result.counters.late_actions;
        totals.glory_emitted = totals
            .glory_emitted
            .checked_add(result.summary.glory_emitted)
            .ok_or(SimulationError::Arithmetic("total GLORY emission"))?;
        turnover_sum += result.summary.final_window_turnover_rate;
        final_chapter_turnover_sum += result.summary.final_chapter_turnover_rate;
        for (sum, value) in chapter_turnover_sums
            .iter_mut()
            .zip(&result.summary.chapter_boundary_turnover_rates)
        {
            *sum += value;
        }
        concentration_sum += result.summary.target_concentration;

        for (index, reward) in result.reward_by_player.iter().copied().enumerate() {
            reward_by_player[index] = reward_by_player[index]
                .checked_add(reward)
                .ok_or(SimulationError::Arithmetic("player GLORY accumulation"))?;
        }
        for (row, reward) in result
            .player_rows
            .iter()
            .zip(result.reward_by_player.iter())
        {
            let strategy = strategies.entry(row.strategy).or_default();
            strategy.player_epochs += 1;
            strategy.eligible += u64::from(row.eligible);
            strategy.wins += u64::from(row.winner);
            if row.coalition_controller {
                strategy.controller_player_epochs += 1;
                strategy.controller_wins += u64::from(row.winner);
                strategy.controller_glory = strategy
                    .controller_glory
                    .checked_add(*reward)
                    .ok_or(SimulationError::Arithmetic("controller GLORY accumulation"))?;
            }
            strategy.rank_percentile_sum += if config.population <= 1 {
                0.0
            } else {
                row.final_rank as f64 / f64::from(config.population - 1)
            };
            strategy.score_ratio_sum += ratio(row.final_score, row.starting_allocation);
            strategy.actions += u64::from(row.actions);
            strategy.glory = strategy
                .glory
                .checked_add(*reward)
                .ok_or(SimulationError::Arithmetic("strategy GLORY accumulation"))?;

            let tier = tiers
                .entry(row.starting_allocation / rules::DUMP_TIER_SIZE)
                .or_default();
            tier.player_epochs += 1;
            tier.eligible += u64::from(row.eligible);
            tier.wins += u64::from(row.winner);
            if row.eligible {
                tier.score_ratio_sum += ratio(row.final_score, row.starting_allocation);
            }
        }
        epoch_samples.push(result.summary);
    }

    totals.final_window_winner_turnover_rate = turnover_sum / f64::from(config.epochs);
    totals.final_chapter_winner_turnover_rate =
        final_chapter_turnover_sum / f64::from(config.epochs);
    totals.mean_chapter_boundary_turnover_rates = chapter_turnover_sums
        .into_iter()
        .map(|sum| sum / f64::from(config.epochs))
        .collect();
    totals.mean_target_concentration = concentration_sum / f64::from(config.epochs);
    totals.glory_gini = gini(&reward_by_player);
    let billed_actions = totals.dump_actions + totals.absorb_actions + totals.redirect_arms;
    totals.estimated_action_fees_lamports =
        u128::from(billed_actions) * u128::from(config.estimated_signature_fee_lamports);
    totals.hypothetical_glory_value_lamports = u128::from(totals.glory_emitted)
        * u128::from(config.hypothetical_glory_price_lamports)
        / u128::from(rules::GLORY_SCALE);

    let overall_win_rate = safe_div(totals.rewarded_player_epochs, totals.player_epochs);
    let strategy_reports = strategies
        .into_iter()
        .map(|(strategy, value)| {
            let fees =
                u128::from(value.actions) * u128::from(config.estimated_signature_fee_lamports);
            let bonds = u128::from(value.eligible)
                * u128::from(
                    rules::REGISTRATION_BOND_LAMPORTS
                        - rules::REGISTRATION_BOND_LAMPORTS * rules::BOND_REFUND_BPS
                            / rules::BPS_DENOMINATOR,
                )
                + u128::from(value.player_epochs - value.eligible)
                    * u128::from(rules::REGISTRATION_BOND_LAMPORTS);
            let coalition_helpers = if is_coalition_strategy(strategy) {
                value
                    .player_epochs
                    .saturating_sub(value.controller_player_epochs)
            } else {
                0
            };
            let coordination = u128::from(coalition_helpers)
                * u128::from(
                    config
                        .research
                        .coalition_coordination_cost_lamports_per_helper_epoch,
                );
            let total_cost = fees.saturating_add(bonds).saturating_add(coordination);
            let proceeds = u128::from(value.glory)
                * u128::from(config.hypothetical_glory_price_lamports)
                / u128::from(rules::GLORY_SCALE);
            StrategyReport {
                estimated_fees_lamports: fees,
                estimated_bond_cost_lamports: bonds,
                coalition_helper_player_epochs: coalition_helpers,
                estimated_coordination_cost_lamports: coordination,
                estimated_total_cost_lamports: total_cost,
                break_even_glory_price_lamports: (value.glory > 0).then(|| {
                    (total_cost * u128::from(rules::GLORY_SCALE)).div_ceil(u128::from(value.glory))
                }),
                hypothetical_glory_value_lamports: proceeds,
                hypothetical_net_before_rent_lamports: saturating_i128(proceeds)
                    .saturating_sub(saturating_i128(total_cost)),
                hypothetical_return_on_cost: (total_cost > 0)
                    .then(|| proceeds as f64 / total_cost as f64 - 1.0),
                strategy,
                player_epochs: value.player_epochs,
                eligible_player_epochs: value.eligible,
                wins: value.wins,
                win_rate: safe_div(value.wins, value.player_epochs),
                winner_share: safe_div(value.wins, totals.rewarded_player_epochs),
                relative_win_advantage: if overall_win_rate == 0.0 {
                    0.0
                } else {
                    safe_div(value.wins, value.player_epochs) / overall_win_rate
                },
                controller_player_epochs: value.controller_player_epochs,
                controller_wins: value.controller_wins,
                controller_win_rate: safe_div(
                    value.controller_wins,
                    value.controller_player_epochs,
                ),
                controller_relative_win_advantage: if overall_win_rate == 0.0 {
                    0.0
                } else {
                    safe_div(value.controller_wins, value.controller_player_epochs)
                        / overall_win_rate
                },
                controller_glory_earned: value.controller_glory,
                mean_rank_percentile: value.rank_percentile_sum / value.player_epochs as f64,
                mean_final_score_ratio: value.score_ratio_sum / value.player_epochs as f64,
                mean_actions: value.actions as f64 / value.player_epochs as f64,
                glory_earned: value.glory,
            }
        })
        .collect::<Vec<_>>();
    let tier_reports = tiers
        .into_iter()
        .map(|(tier_billions, value)| TierReport {
            tier_billions,
            player_epochs: value.player_epochs,
            eligible_player_epochs: value.eligible,
            wins: value.wins,
            win_rate: safe_div(value.wins, value.eligible),
            all_player_win_rate: safe_div(value.wins, value.player_epochs),
            mean_final_score_ratio: if value.eligible == 0 {
                0.0
            } else {
                value.score_ratio_sum / value.eligible as f64
            },
        })
        .collect::<Vec<_>>();
    let balance_signals = build_signals(&totals, &strategy_reports, &tier_reports, canonical_rules);

    Ok(SimulationReport {
        schema: REPORT_SCHEMA,
        canonical_rules,
        implementation_status: if canonical_rules {
            "committed_v3_rules"
        } else {
            "simulator_only_research_rules"
        },
        action_fee_model: if config.research.execution_model == ExecutionModel::Sequential {
            "one_signature_fee_per_successful_action"
        } else {
            "direct_l1_counterfactual_excludes_aggregate_settlement_cost"
        },
        config,
        totals,
        strategies: strategy_reports,
        starting_tiers: tier_reports,
        balance_signals,
        epoch_samples,
    })
}

fn simulate_epoch(
    config: &SimulationConfig,
    epoch_number: u64,
    epoch_seed: u64,
    remaining_glory: u64,
) -> Result<EpochResult, SimulationError> {
    let population = config.population as usize;
    let mut rng = SplitMix64::new(epoch_seed);
    let strategies = assign_strategies(config, population);
    let mut players = strategies
        .into_iter()
        .enumerate()
        .map(|(id, strategy)| {
            let revealed = strategy != Strategy::RevealWithholder;
            let allocation = if revealed {
                rules::allocation_from_entropy(rng.next_u64())
            } else {
                rules::MAX_STARTING_DUMP
            };
            Player::new(id, strategy, allocation, revealed)
        })
        .collect::<Vec<_>>();
    let coalition_controllers = coalition_controllers(&players);
    let mut rivalry_credit: HashMap<(usize, usize), u64> = HashMap::new();
    let mut rivalry_actions: HashMap<(usize, usize), u32> = HashMap::new();
    let mut counters = EpochCounters {
        incoming_by_player: vec![0; population],
        ..EpochCounters::default()
    };
    let rounds = config.decision_rounds();
    let delay_rounds =
        u32::try_from((config.information_delay_seconds / config.decision_interval_seconds).max(0))
            .unwrap_or(rounds)
            .min(rounds);
    let mut rank_history: Vec<Vec<usize>> = Vec::with_capacity(rounds as usize + 1);
    rank_history.push(rank_players(&players, 0, epoch_seed, &config.research)?);
    let mut late_window_winners = None;
    let mut chapter_boundary_winners =
        vec![None; usize::from(config.research.chapter_count.saturating_sub(1))];

    for round in 0..rounds {
        let now = (i64::from(round + 1) * rules::ACTIVE_SECONDS / i64::from(rounds + 1))
            .clamp(0, rules::ACTIVE_SECONDS - 1);
        let late_boundary = rules::ACTIVE_SECONDS
            * i64::try_from(LATE_WINDOW_BPS).unwrap_or_default()
            / i64::try_from(rules::BPS_DENOMINATOR).unwrap_or(1);
        if late_window_winners.is_none() && now >= late_boundary {
            let projected = rank_players_projected_no_action(
                &players,
                late_boundary,
                epoch_seed,
                &config.research,
            )?;
            late_window_winners = Some(provisional_winner_set(
                &players,
                &projected,
                &config.research,
            ));
        }
        for boundary in 1..config.research.chapter_count {
            let index = usize::from(boundary - 1);
            let boundary_at = i64::from(boundary) * rules::ACTIVE_SECONDS
                / i64::from(config.research.chapter_count);
            if chapter_boundary_winners[index].is_none() && now >= boundary_at {
                let projected = rank_players_projected_no_action(
                    &players,
                    boundary_at,
                    epoch_seed,
                    &config.research,
                )?;
                chapter_boundary_winners[index] = Some(provisional_winner_set(
                    &players,
                    &projected,
                    &config.research,
                ));
            }
        }
        let current_ranks = rank_players(&players, now, epoch_seed, &config.research)?;
        let stale_index = usize::try_from(round.saturating_sub(delay_rounds)).unwrap_or(0);
        let observed_ranks = rank_history
            .get(stale_index)
            .cloned()
            .unwrap_or_else(|| current_ranks.clone());
        let observed_positions = invert_ranks(&observed_ranks, population);
        let mut order = (0..population).collect::<Vec<_>>();
        rng.shuffle(&mut order);

        match config.research.execution_model {
            ExecutionModel::Sequential => {
                for actor in order {
                    let Some(intent) = choose_intent(
                        &players,
                        actor,
                        &observed_ranks,
                        &observed_positions,
                        &coalition_controllers,
                        now,
                        config,
                        &mut rng,
                    ) else {
                        continue;
                    };
                    execute_submission(
                        &mut players,
                        actor,
                        intent,
                        now,
                        epoch_seed,
                        config,
                        &mut rivalry_credit,
                        &mut rivalry_actions,
                        &mut counters,
                        &mut rng,
                    )?;
                }
            }
            ExecutionModel::ScheduledBatch => {
                let mut intents = Vec::with_capacity(
                    population * usize::from(config.research.max_intents_per_window),
                );
                for actor in order {
                    for _ in 0..config.research.max_intents_per_window {
                        let Some(intent) = choose_intent(
                            &players,
                            actor,
                            &observed_ranks,
                            &observed_positions,
                            &coalition_controllers,
                            now,
                            config,
                            &mut rng,
                        ) else {
                            break;
                        };
                        intents.push((actor, intent));
                    }
                }
                // Submission time cannot buy priority inside a window. The
                // deterministic epoch stream supplies the resolution order.
                rng.shuffle(&mut intents);
                for (actor, intent) in intents {
                    execute_submission(
                        &mut players,
                        actor,
                        intent,
                        now,
                        epoch_seed,
                        config,
                        &mut rivalry_credit,
                        &mut rivalry_actions,
                        &mut counters,
                        &mut rng,
                    )?;
                }
            }
        }
        let post_ranks = rank_players(&players, now, epoch_seed, &config.research)?;
        rank_history.push(post_ranks);
    }

    if late_window_winners.is_none() {
        let late_boundary = rules::ACTIVE_SECONDS
            * i64::try_from(LATE_WINDOW_BPS).unwrap_or_default()
            / i64::try_from(rules::BPS_DENOMINATOR).unwrap_or(1);
        let projected = rank_players_projected_no_action(
            &players,
            late_boundary,
            epoch_seed,
            &config.research,
        )?;
        late_window_winners = Some(provisional_winner_set(
            &players,
            &projected,
            &config.research,
        ));
    }
    for boundary in 1..config.research.chapter_count {
        let index = usize::from(boundary - 1);
        if chapter_boundary_winners[index].is_none() {
            let boundary_at = i64::from(boundary) * rules::ACTIVE_SECONDS
                / i64::from(config.research.chapter_count);
            let projected = rank_players_projected_no_action(
                &players,
                boundary_at,
                epoch_seed,
                &config.research,
            )?;
            chapter_boundary_winners[index] = Some(provisional_winner_set(
                &players,
                &projected,
                &config.research,
            ));
        }
    }

    for player in &mut players {
        for lane in &mut player.lanes {
            lane.checkpoint(rules::ACTIVE_SECONDS, &config.research)?;
        }
    }
    let final_order = rank_players(
        &players,
        rules::ACTIVE_SECONDS,
        epoch_seed,
        &config.research,
    )?;
    let eligible_order = final_order
        .iter()
        .copied()
        .filter(|index| players[*index].eligible())
        .collect::<Vec<_>>();
    let winner_count = simulated_winner_count(eligible_order.len() as u32, &config.research);
    let winner_order = eligible_order
        .iter()
        .copied()
        .take(usize::from(winner_count))
        .collect::<Vec<_>>();
    let winners = winner_order.iter().copied().collect::<BTreeSet<_>>();
    let total_pool =
        rules::epoch_emission(epoch_number, eligible_order.len() as u32, remaining_glory)
            .map_err(|_| SimulationError::Arithmetic("epoch emission"))?;
    let raw_keeper_pool = rules::keeper_pool(total_pool)
        .map_err(|_| SimulationError::Arithmetic("keeper reward pool"))?;
    let keeper_pool = if population == 0 {
        0
    } else {
        raw_keeper_pool / population as u64 * population as u64
    };
    let player_pool = total_pool
        .checked_sub(keeper_pool)
        .ok_or(SimulationError::Arithmetic("player reward pool"))?;
    let mut reward_by_player = vec![0u64; population];
    for (rank, player) in winner_order.iter().copied().enumerate() {
        reward_by_player[player] = rules::rank_reward(player_pool, winner_count, rank as u16)
            .map_err(|_| SimulationError::Arithmetic("rank reward"))?;
    }
    let distributed = reward_by_player.iter().sum::<u64>();
    if winner_count > 0 && distributed != player_pool {
        return Err(SimulationError::Arithmetic("rank rewards do not conserve"));
    }

    let final_positions = final_order
        .iter()
        .enumerate()
        .map(|(rank, player)| (*player, rank))
        .collect::<HashMap<_, _>>();
    let player_rows = players
        .iter()
        .map(|player| {
            Ok(PlayerResult {
                strategy: player.strategy,
                coalition_controller: coalition_controllers.get(&player.strategy).copied()
                    == Some(player.id),
                starting_allocation: player.starting_allocation,
                final_score: player.current_score(rules::ACTIVE_SECONDS, &config.research)?,
                final_rank: *final_positions.get(&player.id).unwrap_or(&population),
                actions: player.meaningful_actions,
                eligible: player.eligible(),
                winner: winners.contains(&player.id),
            })
        })
        .collect::<Result<Vec<_>, SimulationError>>()?;
    let total_incoming = counters.incoming_by_player.iter().sum::<u128>();
    let target_concentration = if total_incoming == 0 {
        0.0
    } else {
        counters
            .incoming_by_player
            .iter()
            .copied()
            .max()
            .unwrap_or_default() as f64
            / total_incoming as f64
    };
    let final_window_turnover_rate = if winners.is_empty() {
        0.0
    } else {
        winners
            .difference(late_window_winners.as_ref().expect("snapshot is populated"))
            .count() as f64
            / winners.len() as f64
    };
    let chapter_boundary_turnover_rates = chapter_boundary_winners
        .into_iter()
        .map(|snapshot| {
            if winners.is_empty() {
                0.0
            } else {
                let snapshot = snapshot.unwrap_or_default();
                winners.difference(&snapshot).count() as f64 / winners.len() as f64
            }
        })
        .collect::<Vec<_>>();
    let final_chapter_turnover_rate = chapter_boundary_turnover_rates
        .last()
        .copied()
        .unwrap_or_default();
    let top_strategy = winner_order
        .first()
        .copied()
        .map(|winner| players[winner].strategy);
    let initial_dump = players
        .iter()
        .map(|player| u128::from(player.starting_allocation))
        .sum::<u128>();
    let final_dump = players
        .iter()
        .flat_map(|player| player.lanes.iter())
        .map(|lane| u128::from(lane.balance))
        .sum::<u128>();
    if initial_dump != final_dump {
        return Err(SimulationError::Arithmetic("DUMP conservation"));
    }
    let guard_stranded = players
        .iter()
        .map(|player| u128::from(player.guard()))
        .sum();
    let summary = EpochSummary {
        epoch: epoch_number,
        eligible_players: eligible_order.len() as u32,
        winners: winner_count,
        actions: counters.action_count(),
        target_concentration,
        final_window_turnover_rate,
        final_chapter_turnover_rate,
        chapter_boundary_turnover_rates,
        glory_emitted: total_pool,
        top_strategy,
    };
    Ok(EpochResult {
        summary,
        player_rows,
        counters,
        guard_stranded,
        reward_by_player,
    })
}

fn assign_strategies(config: &SimulationConfig, population: usize) -> Vec<Strategy> {
    let base: &[Strategy] = match config.scenario {
        Scenario::RandomOnly => &[Strategy::Random],
        Scenario::LeaderHunt => &[
            Strategy::LeaderHunter,
            Strategy::GreedyDumper,
            Strategy::SmallPacketDumper,
            Strategy::QuietSmallFry,
            Strategy::Random,
        ],
        Scenario::GuardHeavy => &[
            Strategy::GuardBuilder,
            Strategy::RedirectBluffer,
            Strategy::LeaderHunter,
            Strategy::Random,
        ],
        Scenario::LastWindow => &[
            Strategy::LastWindow,
            Strategy::LeaderHunter,
            Strategy::QuietSmallFry,
            Strategy::Random,
        ],
        Scenario::SybilStress => &[
            Strategy::Random,
            Strategy::SmallPacketDumper,
            Strategy::GreedyDumper,
            Strategy::LeaderHunter,
            Strategy::GuardBuilder,
            Strategy::QuietSmallFry,
        ],
        Scenario::Mixed => &[
            Strategy::Random,
            Strategy::Inactive,
            Strategy::SmallPacketDumper,
            Strategy::GreedyDumper,
            Strategy::LeaderHunter,
            Strategy::QuietSmallFry,
            Strategy::GuardBuilder,
            Strategy::RedirectBluffer,
            Strategy::LastWindow,
            Strategy::ReciprocalAlliance,
            Strategy::RotatingCoalition,
            Strategy::BribedCoalition,
            Strategy::RevealWithholder,
        ],
    };
    let mut assigned = (0..population)
        .map(|index| base[index % base.len()])
        .collect::<Vec<_>>();
    if matches!(config.scenario, Scenario::Mixed | Scenario::SybilStress) {
        let sybil_count = (population as u128 * u128::from(config.sybil_fraction_bps)
            / u128::from(rules::BPS_DENOMINATOR)) as usize;
        let start = population.saturating_sub(sybil_count);
        for (offset, strategy) in assigned[start..].iter_mut().enumerate() {
            *strategy = if matches!(config.scenario, Scenario::Mixed) {
                if offset < sybil_count.div_ceil(2) {
                    Strategy::SacrificialSybil
                } else {
                    Strategy::SelfDogpile
                }
            } else {
                [
                    Strategy::SacrificialSybil,
                    Strategy::SelfDogpile,
                    Strategy::BribedCoalition,
                ][offset % 3]
            };
        }
    }
    assigned
}

fn coalition_controllers(players: &[Player]) -> BTreeMap<Strategy, usize> {
    let mut controllers = BTreeMap::new();
    for player in players {
        if is_coalition_strategy(player.strategy) {
            controllers.entry(player.strategy).or_insert(player.id);
        }
    }
    controllers
}

fn invert_ranks(ranks: &[usize], population: usize) -> Vec<usize> {
    let mut positions = vec![population; population];
    for (rank, player) in ranks.iter().copied().enumerate() {
        if player < population {
            positions[player] = rank;
        }
    }
    positions
}

#[allow(clippy::too_many_arguments)]
fn choose_intent(
    players: &[Player],
    actor: usize,
    ranks: &[usize],
    rank_by_player: &[usize],
    coalition_controllers: &BTreeMap<Strategy, usize>,
    now: i64,
    config: &SimulationConfig,
    rng: &mut SplitMix64,
) -> Option<Intent> {
    let player = &players[actor];
    let progress_bps = u64::try_from(now).ok()? * rules::BPS_DENOMINATOR
        / u64::try_from(rules::ACTIVE_SECONDS).ok()?;
    let leader = ranks.iter().copied().find(|target| *target != actor)?;
    let burdened = ranks
        .iter()
        .rev()
        .copied()
        .find(|target| *target != actor)?;
    let random = random_other(players.len(), actor, rng);
    let ready_guard = player
        .lanes
        .iter()
        .any(|lane| lane.guard > 0 && !lane.redirect_armed && now >= lane.redirect_ready_at);
    let rank = rank_by_player.get(actor).copied().unwrap_or(players.len());

    match player.strategy {
        Strategy::Inactive | Strategy::RevealWithholder => None,
        Strategy::Random => match rng.next_u64() % 10 {
            0..=5 => Some(Intent::Dump {
                target: random,
                percent: 5 + rng.next_u64() % 21,
            }),
            6..=8 => Some(Intent::Absorb {
                target: random,
                percent: 3 + rng.next_u64() % 13,
            }),
            _ if ready_guard => Some(Intent::Arm),
            _ => None,
        },
        Strategy::SmallPacketDumper => Some(Intent::Dump {
            target: random,
            percent: 3 + rng.next_u64() % 7,
        }),
        Strategy::GreedyDumper => Some(Intent::Dump {
            target: if rng.chance_bps(6_000) {
                leader
            } else {
                random
            },
            percent: 15 + rng.next_u64() % 26,
        }),
        Strategy::LeaderHunter => Some(Intent::Dump {
            target: leader,
            percent: 8 + rng.next_u64() % 18,
        }),
        Strategy::QuietSmallFry => {
            if rank < players.len().div_ceil(5) && !rng.chance_bps(2_000) {
                ready_guard.then_some(Intent::Arm)
            } else {
                Some(Intent::Dump {
                    target: burdened,
                    percent: 4 + rng.next_u64() % 9,
                })
            }
        }
        Strategy::GuardBuilder => {
            let desired_guard = mul_bps(
                player.starting_allocation,
                config.tuning.total_guard_cap_bps.saturating_mul(3) / 4,
            );
            if player.guard() < desired_guard && progress_bps < 7_500 {
                Some(Intent::Absorb {
                    target: burdened,
                    percent: 5 + rng.next_u64() % 11,
                })
            } else if ready_guard {
                Some(Intent::Arm)
            } else {
                Some(Intent::Dump {
                    target: leader,
                    percent: 8 + rng.next_u64() % 13,
                })
            }
        }
        Strategy::RedirectBluffer => {
            if ready_guard && rng.chance_bps(7_500) {
                Some(Intent::Arm)
            } else if player.guard() == 0 && progress_bps < 8_000 {
                Some(Intent::Absorb {
                    target: random,
                    percent: 4 + rng.next_u64() % 8,
                })
            } else {
                Some(Intent::Dump {
                    target: leader,
                    percent: 5 + rng.next_u64() % 11,
                })
            }
        }
        Strategy::LastWindow => {
            if progress_bps < 7_500 {
                None
            } else {
                Some(Intent::Dump {
                    target: leader,
                    percent: 20 + rng.next_u64() % 31,
                })
            }
        }
        Strategy::ReciprocalAlliance => {
            let team = actor % 4;
            let target = ranks
                .iter()
                .copied()
                .find(|candidate| *candidate != actor && *candidate % 4 != team)
                .unwrap_or(random);
            Some(Intent::Dump {
                target,
                percent: 7 + rng.next_u64() % 14,
            })
        }
        Strategy::RotatingCoalition => {
            let coalition = ((now / config.decision_interval_seconds) as usize + actor) % 5;
            let target = ranks
                .iter()
                .copied()
                .find(|candidate| *candidate != actor && *candidate % 5 != coalition)
                .unwrap_or(leader);
            Some(Intent::Dump {
                target,
                percent: 9 + rng.next_u64() % 12,
            })
        }
        Strategy::SacrificialSybil => {
            let controller = coalition_controllers
                .get(&Strategy::SacrificialSybil)
                .copied()
                .unwrap_or(actor);
            if actor == controller {
                Some(Intent::Dump {
                    target: leader,
                    percent: 14 + rng.next_u64() % 17,
                })
            } else {
                Some(Intent::Absorb {
                    target: controller,
                    percent: 12 + rng.next_u64() % 18,
                })
            }
        }
        Strategy::SelfDogpile => {
            let controller = coalition_controllers
                .get(&Strategy::SelfDogpile)
                .copied()
                .unwrap_or(actor);
            if actor == controller {
                Some(Intent::Dump {
                    target: random,
                    percent: 16 + rng.next_u64() % 20,
                })
            } else {
                Some(Intent::Dump {
                    target: controller,
                    percent: 3 + rng.next_u64() % 7,
                })
            }
        }
        Strategy::BribedCoalition => {
            let beneficiary = coalition_controllers
                .get(&Strategy::BribedCoalition)
                .copied()
                .unwrap_or(actor);
            if actor == beneficiary {
                Some(Intent::Dump {
                    target: leader,
                    percent: 15 + rng.next_u64() % 21,
                })
            } else {
                Some(Intent::Absorb {
                    target: beneficiary,
                    percent: 8 + rng.next_u64() % 13,
                })
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_submission(
    players: &mut [Player],
    actor: usize,
    intent: Intent,
    now: i64,
    epoch_seed: u64,
    config: &SimulationConfig,
    rivalry_credit: &mut HashMap<(usize, usize), u64>,
    rivalry_actions: &mut HashMap<(usize, usize), u32>,
    counters: &mut EpochCounters,
    rng: &mut SplitMix64,
) -> Result<(), SimulationError> {
    if rng.chance_bps(config.rpc_failure_bps) {
        counters.simulated_rpc_failures += 1;
        return Ok(());
    }
    execute_intent(
        players,
        actor,
        intent,
        now,
        epoch_seed,
        config,
        rivalry_credit,
        rivalry_actions,
        counters,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_intent(
    players: &mut [Player],
    actor: usize,
    intent: Intent,
    now: i64,
    epoch_seed: u64,
    config: &SimulationConfig,
    rivalry_credit: &mut HashMap<(usize, usize), u64>,
    rivalry_actions: &mut HashMap<(usize, usize), u32>,
    counters: &mut EpochCounters,
) -> Result<(), SimulationError> {
    match intent {
        Intent::Arm => execute_arm(players, actor, now, counters),
        Intent::Dump { target, percent } => execute_dump(
            players,
            actor,
            target,
            percent,
            now,
            epoch_seed,
            config,
            rivalry_credit,
            rivalry_actions,
            counters,
        ),
        Intent::Absorb { target, percent } => execute_absorb(
            players,
            actor,
            target,
            percent,
            now,
            config,
            rivalry_credit,
            rivalry_actions,
            counters,
        ),
    }
}

fn execute_arm(
    players: &mut [Player],
    actor: usize,
    now: i64,
    counters: &mut EpochCounters,
) -> Result<(), SimulationError> {
    let Some((_, lane)) = players[actor]
        .lanes
        .iter_mut()
        .enumerate()
        .filter(|(_, lane)| lane.guard > 0 && !lane.redirect_armed && now >= lane.redirect_ready_at)
        .max_by_key(|(_, lane)| lane.guard)
    else {
        counters.failed_balance_actions += 1;
        return Ok(());
    };
    lane.redirect_armed = true;
    counters.redirect_arms += 1;
    if in_late_window(now) {
        counters.late_actions += 1;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn execute_dump(
    players: &mut [Player],
    actor: usize,
    target: usize,
    percent: u64,
    now: i64,
    epoch_seed: u64,
    config: &SimulationConfig,
    rivalry_credit: &mut HashMap<(usize, usize), u64>,
    rivalry_actions: &mut HashMap<(usize, usize), u32>,
    counters: &mut EpochCounters,
) -> Result<(), SimulationError> {
    if actor == target || target >= players.len() {
        counters.failed_balance_actions += 1;
        return Ok(());
    }
    let start = players[actor].starting_allocation;
    let intent_basis = match config.research.action_capacity {
        ActionCapacity::CanonicalHeat => start,
        ActionCapacity::ChapterStamina => {
            chapter_capacity_basis(start, config.research.chapter_start_weight_bps)
        }
    };
    let desired = intent_basis.saturating_mul(percent) / 100;
    let minimum = minimum_action(start, &config.tuning);
    let action_limited = match config.research.action_capacity {
        ActionCapacity::CanonicalHeat => {
            let current_heat = tuned_heat_at(players[actor].dump_heat, now, &config.tuning);
            max_amount_for_heat(current_heat, start)
        }
        ActionCapacity::ChapterStamina => {
            players[actor].available_capacity(CapacityChannel::Dump, now, &config.research)
        }
    };
    let inbound_limited = players[target].available_inbound(now, &config.research);
    let Some((source_index, spendable)) = players[actor]
        .lanes
        .iter()
        .enumerate()
        .map(|(index, lane)| (index, lane.spendable(now)))
        .max_by_key(|(_, amount)| *amount)
    else {
        counters.failed_balance_actions += 1;
        return Ok(());
    };
    let amount = desired
        .min(spendable)
        .min(action_limited)
        .min(inbound_limited);
    if amount < minimum {
        if inbound_limited < minimum {
            counters.failed_inbound_cap_actions += 1;
        } else if action_limited < minimum {
            match config.research.action_capacity {
                ActionCapacity::CanonicalHeat => counters.failed_heat_actions += 1,
                ActionCapacity::ChapterStamina => counters.failed_capacity_actions += 1,
            }
        } else {
            counters.failed_balance_actions += 1;
        }
        return Ok(());
    }
    let action_count = *rivalry_actions.get(&(actor, target)).unwrap_or(&0);
    let target_index = deterministic_target_lane(epoch_seed, actor, target, action_count);
    let next_heat = if config.research.action_capacity == ActionCapacity::CanonicalHeat {
        Some(
            tuned_charge_heat(players[actor].dump_heat, now, amount, start, &config.tuning)
                .ok_or(SimulationError::Arithmetic("prechecked DUMP Heat"))?,
        )
    } else {
        None
    };

    let (actor_player, target_player) = two_players_mut(players, actor, target);
    actor_player.lanes[source_index].checkpoint(now, &config.research)?;
    target_player.lanes[target_index].checkpoint(now, &config.research)?;
    if actor_player.lanes[source_index].spendable(now) < amount {
        counters.failed_balance_actions += 1;
        return Ok(());
    }
    let target_lane = &mut target_player.lanes[target_index];
    let (landed, redirected, guard_after) =
        rules::apply_redirect(amount, target_lane.guard, target_lane.redirect_armed);
    actor_player.lanes[source_index].balance = actor_player.lanes[source_index]
        .balance
        .checked_sub(amount)
        .and_then(|value| value.checked_add(redirected))
        .ok_or(SimulationError::Arithmetic("DUMP source balance"))?;
    target_lane.balance = target_lane
        .balance
        .checked_add(landed)
        .ok_or(SimulationError::Arithmetic("DUMP target balance"))?;
    target_lane.guard = guard_after;
    if redirected > 0 {
        target_lane.redirect_armed = false;
        target_lane.redirect_ready_at = now.saturating_add(config.tuning.redirect_rearm_seconds);
        target_lane.redirected_volume = target_lane
            .redirected_volume
            .checked_add(redirected)
            .ok_or(SimulationError::Arithmetic("redirected volume"))?;
    }
    if config.research.target_inbound_cap_bps_per_chapter > 0 {
        target_player.inbound_used = target_player
            .inbound_used
            .checked_add(landed)
            .ok_or(SimulationError::Arithmetic("target inbound accounting"))?;
    }
    if let Some(next_heat) = next_heat {
        actor_player.dump_heat = next_heat;
    } else {
        actor_player.charge_capacity(CapacityChannel::Dump, amount, &config.research)?;
    }
    counters.dump_actions += 1;
    counters.dump_moved += u128::from(amount);
    counters.redirected_dump += u128::from(redirected);
    counters.incoming_by_player[target] += u128::from(landed);
    if in_late_window(now) {
        counters.late_actions += 1;
    }
    record_action(
        players,
        actor,
        target,
        amount,
        now,
        rivalry_credit,
        rivalry_actions,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_absorb(
    players: &mut [Player],
    actor: usize,
    target: usize,
    percent: u64,
    now: i64,
    config: &SimulationConfig,
    rivalry_credit: &mut HashMap<(usize, usize), u64>,
    rivalry_actions: &mut HashMap<(usize, usize), u32>,
    counters: &mut EpochCounters,
) -> Result<(), SimulationError> {
    if actor == target || target >= players.len() {
        counters.failed_balance_actions += 1;
        return Ok(());
    }
    let start = players[actor].starting_allocation;
    let intent_basis = match config.research.action_capacity {
        ActionCapacity::CanonicalHeat => start,
        ActionCapacity::ChapterStamina => {
            chapter_capacity_basis(start, config.research.chapter_start_weight_bps)
        }
    };
    let desired = intent_basis.saturating_mul(percent) / 100;
    let minimum = minimum_action(start, &config.tuning);
    let action_limited = match config.research.action_capacity {
        ActionCapacity::CanonicalHeat => {
            let current_heat = tuned_heat_at(players[actor].absorb_heat, now, &config.tuning);
            max_amount_for_heat(current_heat, start)
        }
        ActionCapacity::ChapterStamina => {
            players[actor].available_capacity(CapacityChannel::Absorb, now, &config.research)
        }
    };
    let relief_limited = players[target].available_absorb_relief(now, &config.research);
    let Some((source_index, spendable)) = players[target]
        .lanes
        .iter()
        .enumerate()
        .map(|(index, lane)| (index, lane.spendable(now)))
        .max_by_key(|(_, amount)| *amount)
    else {
        counters.failed_balance_actions += 1;
        return Ok(());
    };
    let destination_index = players[actor]
        .lanes
        .iter()
        .enumerate()
        .min_by_key(|(_, lane)| (lane.guard, lane.balance))
        .map_or(0, |(index, _)| index);
    let amount = desired
        .min(spendable)
        .min(action_limited)
        .min(relief_limited);
    if amount < minimum {
        if relief_limited < minimum {
            counters.failed_relief_cap_actions += 1;
        } else if action_limited < minimum {
            match config.research.action_capacity {
                ActionCapacity::CanonicalHeat => counters.failed_heat_actions += 1,
                ActionCapacity::ChapterStamina => counters.failed_capacity_actions += 1,
            }
        } else {
            counters.failed_balance_actions += 1;
        }
        return Ok(());
    }
    let next_heat = if config.research.action_capacity == ActionCapacity::CanonicalHeat {
        Some(
            tuned_charge_heat(
                players[actor].absorb_heat,
                now,
                amount,
                start,
                &config.tuning,
            )
            .ok_or(SimulationError::Arithmetic("prechecked ABSORB Heat"))?,
        )
    } else {
        None
    };

    let (actor_player, target_player) = two_players_mut(players, actor, target);
    actor_player.lanes[destination_index].checkpoint(now, &config.research)?;
    target_player.lanes[source_index].checkpoint(now, &config.research)?;
    if target_player.lanes[source_index].spendable(now) < amount {
        counters.failed_balance_actions += 1;
        return Ok(());
    }
    target_player.lanes[source_index].balance = target_player.lanes[source_index]
        .balance
        .checked_sub(amount)
        .ok_or(SimulationError::Arithmetic("ABSORB source balance"))?;
    let destination = &mut actor_player.lanes[destination_index];
    destination.balance = destination
        .balance
        .checked_add(amount)
        .ok_or(SimulationError::Arithmetic("ABSORB destination balance"))?;
    destination.locked_amount = destination
        .locked_amount
        .checked_add(amount)
        .ok_or(SimulationError::Arithmetic("ABSORB locked amount"))?;
    destination.locked_until = destination
        .locked_until
        .max(now.saturating_add(config.tuning.absorb_lock_seconds));
    let guard_before = destination.guard;
    destination.guard = tuned_guard_after(guard_before, amount, start, &config.tuning);
    counters.guard_created += u128::from(destination.guard.saturating_sub(guard_before));
    if let Some(next_heat) = next_heat {
        actor_player.absorb_heat = next_heat;
    } else {
        actor_player.charge_capacity(CapacityChannel::Absorb, amount, &config.research)?;
    }
    if config.research.target_absorb_relief_cap_bps_per_chapter > 0 {
        target_player.absorb_relief_used = target_player
            .absorb_relief_used
            .checked_add(amount)
            .ok_or(SimulationError::Arithmetic("ABSORB relief accounting"))?;
    }
    counters.absorb_actions += 1;
    counters.dump_moved += u128::from(amount);
    if in_late_window(now) {
        counters.late_actions += 1;
    }
    record_action(
        players,
        actor,
        target,
        amount,
        now,
        rivalry_credit,
        rivalry_actions,
    )
}

fn record_action(
    players: &mut [Player],
    actor: usize,
    target: usize,
    amount: u64,
    now: i64,
    rivalry_credit: &mut HashMap<(usize, usize), u64>,
    rivalry_actions: &mut HashMap<(usize, usize), u32>,
) -> Result<(), SimulationError> {
    let pair = (actor, target);
    let credited = rivalry_credit.entry(pair).or_default();
    let credit = rules::impact_credit(amount, players[actor].starting_allocation, *credited)
        .map_err(|_| SimulationError::Arithmetic("impact credit"))?;
    *credited = credited
        .checked_add(credit)
        .ok_or(SimulationError::Arithmetic("pair impact"))?;
    *rivalry_actions.entry(pair).or_default() = rivalry_actions
        .get(&pair)
        .copied()
        .unwrap_or_default()
        .saturating_add(1);
    let player = &mut players[actor];
    player.impact_ppm = player
        .impact_ppm
        .checked_add(credit)
        .ok_or(SimulationError::Arithmetic("player impact"))?;
    player.meaningful_actions = player.meaningful_actions.saturating_add(1);
    if credit > 0 {
        player.opponents.insert(target);
    }
    if in_late_window(now) {
        player.late_impact_ppm = player.late_impact_ppm.saturating_add(credit);
    }
    Ok(())
}

fn rank_players(
    players: &[Player],
    now: i64,
    epoch_seed: u64,
    research: &ResearchRules,
) -> Result<Vec<usize>, SimulationError> {
    let mut rows = players
        .iter()
        .map(|player| Ok((player.id, player.standing(now, epoch_seed, research)?)))
        .collect::<Result<Vec<_>, SimulationError>>()?;
    rows.sort_by(|left, right| rules::compare_standings(&left.1, &right.1));
    Ok(rows.into_iter().map(|(id, _)| id).collect())
}

fn rank_players_projected_no_action(
    players: &[Player],
    now: i64,
    epoch_seed: u64,
    research: &ResearchRules,
) -> Result<Vec<usize>, SimulationError> {
    let mut rows = players
        .iter()
        .map(|player| {
            Ok((
                player.id,
                player.projected_no_action_standing(now, epoch_seed, research)?,
            ))
        })
        .collect::<Result<Vec<_>, SimulationError>>()?;
    rows.sort_by(|left, right| rules::compare_standings(&left.1, &right.1));
    Ok(rows.into_iter().map(|(id, _)| id).collect())
}

fn checkpoint_balance_area(
    cumulative: u128,
    balance: u64,
    from: i64,
    to: i64,
    research: &ResearchRules,
) -> Result<u128, SimulationError> {
    if research.temporal_scoring == TemporalScoring::CanonicalLateWeighted {
        return rules::checkpoint_weighted_balance(
            cumulative,
            balance,
            from,
            to,
            0,
            rules::ACTIVE_SECONDS,
        )
        .map_err(|_| SimulationError::Arithmetic("weighted checkpoint"));
    }
    if to < from {
        return Err(SimulationError::Arithmetic("chapter checkpoint duration"));
    }
    let start = from.clamp(0, rules::ACTIVE_SECONDS);
    let end = to.clamp(0, rules::ACTIVE_SECONDS);
    let mut cursor = start;
    let mut area = cumulative;
    while cursor < end {
        let chapter = chapter_at(cursor, research.chapter_count);
        let chapter_end =
            (i64::from(chapter) + 1) * rules::ACTIVE_SECONDS / i64::from(research.chapter_count);
        let segment_end = end.min(chapter_end.max(cursor + 1));
        let weight = chapter_weight(chapter, research);
        let segment = u128::try_from(segment_end - cursor)
            .map_err(|_| SimulationError::Arithmetic("chapter segment"))?;
        area = area
            .checked_add(
                u128::from(balance)
                    .checked_mul(segment)
                    .and_then(|value| value.checked_mul(u128::from(weight)))
                    .ok_or(SimulationError::Arithmetic("chapter score area"))?,
            )
            .ok_or(SimulationError::Arithmetic("chapter score accumulation"))?;
        cursor = segment_end;
    }
    Ok(area)
}

fn score_from_area(cumulative: u128, research: &ResearchRules) -> Result<u64, SimulationError> {
    if research.temporal_scoring == TemporalScoring::CanonicalLateWeighted {
        return rules::weighted_score(cumulative, rules::ACTIVE_SECONDS)
            .map_err(|_| SimulationError::Arithmetic("projected score"));
    }
    let denominator = chapter_score_denominator(research)?;
    u64::try_from(cumulative / denominator)
        .map_err(|_| SimulationError::Arithmetic("chapter score"))
}

fn chapter_score_denominator(research: &ResearchRules) -> Result<u128, SimulationError> {
    let mut denominator = 0u128;
    for chapter in 0..research.chapter_count {
        let start = i64::from(chapter) * rules::ACTIVE_SECONDS / i64::from(research.chapter_count);
        let end =
            (i64::from(chapter) + 1) * rules::ACTIVE_SECONDS / i64::from(research.chapter_count);
        denominator = denominator
            .checked_add(
                u128::try_from(end - start)
                    .map_err(|_| SimulationError::Arithmetic("chapter duration"))?
                    * u128::from(chapter_weight(chapter, research)),
            )
            .ok_or(SimulationError::Arithmetic("chapter score denominator"))?;
    }
    Ok(denominator)
}

fn chapter_weight(chapter: u8, research: &ResearchRules) -> u64 {
    match research.temporal_scoring {
        TemporalScoring::CanonicalLateWeighted | TemporalScoring::EqualChapters => 10_000,
        TemporalScoring::MildChapters => {
            10_000
                + u64::from(chapter) * 2_500
                    / u64::from(research.chapter_count.saturating_sub(1).max(1))
        }
    }
}

fn chapter_at(now: i64, chapter_count: u8) -> u8 {
    let elapsed = now.clamp(0, rules::ACTIVE_SECONDS.saturating_sub(1));
    let chapter = elapsed * i64::from(chapter_count) / rules::ACTIVE_SECONDS;
    u8::try_from(chapter)
        .unwrap_or(chapter_count.saturating_sub(1))
        .min(chapter_count.saturating_sub(1))
}

fn simulated_winner_count(eligible_players: u32, research: &ResearchRules) -> u16 {
    if research.is_canonical() {
        return rules::winner_count(eligible_players);
    }
    if eligible_players < rules::MIN_REWARDED_PLAYERS {
        return 0;
    }
    let count = (u64::from(eligible_players) * rules::WINNER_BPS).div_ceil(rules::BPS_DENOMINATOR);
    u16::try_from(count).unwrap_or(u16::MAX)
}

fn provisional_winner_set(
    players: &[Player],
    ranks: &[usize],
    research: &ResearchRules,
) -> BTreeSet<usize> {
    let eligible = ranks
        .iter()
        .copied()
        .filter(|index| players[*index].eligible())
        .collect::<Vec<_>>();
    let count = usize::from(simulated_winner_count(eligible.len() as u32, research));
    eligible.into_iter().take(count).collect()
}

fn is_coalition_strategy(strategy: Strategy) -> bool {
    matches!(
        strategy,
        Strategy::SacrificialSybil | Strategy::SelfDogpile | Strategy::BribedCoalition
    )
}

fn minimum_action(start: u64, tuning: &RuleTuning) -> u64 {
    mul_bps_ceil(start, tuning.minimum_action_bps)
}

fn tuned_heat_at(heat: rules::Heat, now: i64, tuning: &RuleTuning) -> u32 {
    if tuning.heat_recovery_seconds == rules::HEAT_RECOVERY_SECONDS {
        return rules::heat_at(heat, now);
    }
    if now <= heat.updated_at || heat.units == 0 {
        return heat.units;
    }
    let elapsed = now.saturating_sub(heat.updated_at);
    if elapsed >= tuning.heat_recovery_seconds {
        return 0;
    }
    let decay = (u64::try_from(elapsed).unwrap_or_default() * u64::from(rules::HEAT_CAP)
        / u64::try_from(tuning.heat_recovery_seconds).unwrap_or(1)) as u32;
    heat.units.saturating_sub(decay)
}

fn tuned_charge_heat(
    heat: rules::Heat,
    now: i64,
    amount: u64,
    start: u64,
    tuning: &RuleTuning,
) -> Option<rules::Heat> {
    if amount < minimum_action(start, tuning) || start == 0 {
        return None;
    }
    let current = tuned_heat_at(heat, now, tuning);
    let cost = u32::try_from(
        (u128::from(amount) * u128::from(rules::HEAT_CAP)).div_ceil(u128::from(start)),
    )
    .ok()?;
    let units = current.checked_add(cost)?;
    (units <= rules::HEAT_CAP).then_some(rules::Heat {
        units,
        updated_at: now,
    })
}

fn max_amount_for_heat(current_heat: u32, start: u64) -> u64 {
    let remaining = rules::HEAT_CAP.saturating_sub(current_heat);
    u64::try_from(u128::from(remaining) * u128::from(start) / u128::from(rules::HEAT_CAP))
        .unwrap_or(u64::MAX)
}

fn tuned_guard_after(current: u64, absorbed: u64, start: u64, tuning: &RuleTuning) -> u64 {
    let grant = mul_bps(absorbed, tuning.guard_conversion_bps);
    let total_cap = mul_bps(start, tuning.total_guard_cap_bps);
    current
        .saturating_add(grant)
        .min(total_cap / rules::LANE_COUNT as u64)
}

fn deterministic_target_lane(
    epoch_seed: u64,
    actor: usize,
    target: usize,
    action_count: u32,
) -> usize {
    let mixed = mix64(
        epoch_seed
            ^ (actor as u64).rotate_left(11)
            ^ (target as u64).rotate_left(29)
            ^ u64::from(action_count).rotate_left(43),
    );
    mixed as usize % rules::LANE_COUNT
}

fn in_late_window(now: i64) -> bool {
    u64::try_from(now).unwrap_or_default() * rules::BPS_DENOMINATOR
        >= u64::try_from(rules::ACTIVE_SECONDS).unwrap_or(1) * LATE_WINDOW_BPS
}

fn random_other(population: usize, actor: usize, rng: &mut SplitMix64) -> usize {
    let candidate = rng.index(population - 1);
    if candidate >= actor {
        candidate + 1
    } else {
        candidate
    }
}

fn two_players_mut(
    players: &mut [Player],
    left: usize,
    right: usize,
) -> (&mut Player, &mut Player) {
    debug_assert_ne!(left, right);
    if left < right {
        let (before, after) = players.split_at_mut(right);
        (&mut before[left], &mut after[0])
    } else {
        let (before, after) = players.split_at_mut(left);
        (&mut after[0], &mut before[right])
    }
}

fn build_signals(
    totals: &TotalsReport,
    strategies: &[StrategyReport],
    tiers: &[TierReport],
    canonical: bool,
) -> Vec<BalanceSignal> {
    let mut signals = Vec::new();
    if !canonical {
        signals.push(BalanceSignal {
            severity: SignalSeverity::Information,
            code: "experimental_rules",
            message: "This report changes at least one game parameter and is not an exact run of the committed rules.".into(),
        });
    }
    if let Some(dominant) = (totals.rewarded_player_epochs > 0)
        .then(|| {
            strategies.iter().max_by(|left, right| {
                left.relative_win_advantage
                    .total_cmp(&right.relative_win_advantage)
            })
        })
        .flatten()
    {
        let severity = if dominant.relative_win_advantage >= 3.0 {
            SignalSeverity::Risk
        } else if dominant.relative_win_advantage >= 2.0 {
            SignalSeverity::Watch
        } else {
            SignalSeverity::Information
        };
        signals.push(BalanceSignal {
            severity,
            code: "strategy_advantage",
            message: format!(
                "{} wins at {:.2}x the population-wide rate in this bot mixture.",
                dominant.strategy, dominant.relative_win_advantage
            ),
        });
    }
    if let Some(controller) = (totals.rewarded_player_epochs > 0)
        .then(|| {
            strategies
                .iter()
                .filter(|strategy| strategy.controller_player_epochs > 0)
                .max_by(|left, right| {
                    left.controller_relative_win_advantage
                        .total_cmp(&right.controller_relative_win_advantage)
                })
        })
        .flatten()
    {
        let severity = if controller.controller_relative_win_advantage >= 3.0 {
            SignalSeverity::Risk
        } else if controller.controller_relative_win_advantage >= 2.0 {
            SignalSeverity::Watch
        } else {
            SignalSeverity::Information
        };
        signals.push(BalanceSignal {
            severity,
            code: "coalition_controller_advantage",
            message: format!(
                "The {} controller wins at {:.2}x the population-wide rate; helper losses are reported separately.",
                controller.strategy, controller.controller_relative_win_advantage
            ),
        });
    }
    let nonzero_tiers = tiers
        .iter()
        .filter(|tier| tier.wins > 0 && tier.player_epochs > 0)
        .collect::<Vec<_>>();
    if let (Some(low), Some(high)) = (
        nonzero_tiers
            .iter()
            .min_by(|a, b| a.win_rate.total_cmp(&b.win_rate)),
        nonzero_tiers
            .iter()
            .max_by(|a, b| a.win_rate.total_cmp(&b.win_rate)),
    ) {
        let ratio = if low.win_rate == 0.0 {
            0.0
        } else {
            high.win_rate / low.win_rate
        };
        signals.push(BalanceSignal {
            severity: if ratio >= 5.0 { SignalSeverity::Risk } else if ratio >= 3.0 { SignalSeverity::Watch } else { SignalSeverity::Information },
            code: "starting_tier_advantage",
            message: format!(
                "The strongest observed starting tier wins {:.2}x as often as the weakest tier with a nonzero win count.",
                ratio
            ),
        });
    }
    let guard_use = if totals.guard_created == 0 {
        0.0
    } else {
        totals.redirected_dump as f64 / totals.guard_created as f64
    };
    signals.push(BalanceSignal {
        severity: if guard_use < 0.1 {
            SignalSeverity::Watch
        } else {
            SignalSeverity::Information
        },
        code: "guard_use",
        message: format!(
            "Players converted {:.1}% of forged Guard into actual ricochets.",
            guard_use * 100.0
        ),
    });
    if totals.final_window_winner_turnover_rate > 0.5 {
        signals.push(BalanceSignal {
            severity: SignalSeverity::Watch,
            code: "endgame_volatility",
            message: format!(
                "{:.1}% of final winners entered the winner set during the last tenth of the epoch.",
                totals.final_window_winner_turnover_rate * 100.0
            ),
        });
    }
    if totals.inactive_epochs > 0 {
        signals.push(BalanceSignal {
            severity: SignalSeverity::Risk,
            code: "inactive_epochs",
            message: format!(
                "{} simulated epochs had fewer than {} eligible players and emitted no GLORY.",
                totals.inactive_epochs,
                rules::MIN_REWARDED_PLAYERS
            ),
        });
    }
    signals
}

fn ratio(value: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        value as f64 / denominator as f64
    }
}

fn safe_div(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn saturating_i128(value: u128) -> i128 {
    i128::try_from(value).unwrap_or(i128::MAX)
}

fn mul_bps(value: u64, bps: u64) -> u64 {
    u64::try_from(u128::from(value) * u128::from(bps) / u128::from(rules::BPS_DENOMINATOR))
        .unwrap_or(u64::MAX)
}

fn mul_bps_ceil(value: u64, bps: u64) -> u64 {
    u64::try_from(
        (u128::from(value) * u128::from(bps)).div_ceil(u128::from(rules::BPS_DENOMINATOR)),
    )
    .unwrap_or(u64::MAX)
}

fn chapter_capacity_basis(starting_allocation: u64, start_weight_bps: u64) -> u64 {
    let reference_weight = rules::BPS_DENOMINATOR.saturating_sub(start_weight_bps);
    u64::try_from(
        (u128::from(starting_allocation) * u128::from(start_weight_bps)
            + u128::from(REFERENCE_ACTION_CAPACITY) * u128::from(reference_weight))
            / u128::from(rules::BPS_DENOMINATOR),
    )
    .unwrap_or(u64::MAX)
}

fn player_key(id: usize) -> [u8; 32] {
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&(id as u64).to_le_bytes());
    key
}

fn gini(values: &[u64]) -> f64 {
    if values.is_empty() || values.iter().all(|value| *value == 0) {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let count = sorted.len() as f64;
    let sum = sorted.iter().map(|value| *value as f64).sum::<f64>();
    let weighted = sorted
        .iter()
        .enumerate()
        .map(|(index, value)| (index + 1) as f64 * *value as f64)
        .sum::<f64>();
    (2.0 * weighted) / (count * sum) - (count + 1.0) / count
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Clone, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let value = mix64(self.state);
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        value
    }

    fn index(&mut self, length: usize) -> usize {
        if length <= 1 {
            0
        } else {
            self.next_u64() as usize % length
        }
    }

    fn chance_bps(&mut self, bps: u64) -> bool {
        self.next_u64() % rules::BPS_DENOMINATOR < bps
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            values.swap(index, self.index(index + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_matches_the_reference_stream() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(rng.next_u64(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(rng.next_u64(), 0x06c4_5d18_8009_454f);
    }

    #[test]
    fn child_epoch_stream_does_not_replay_the_parent_stream() {
        let mut parent = SplitMix64::new(0x474c_4f52_5944_554d);
        let epoch_seed = parent.next_u64();
        let next_epoch_seed = parent.next_u64();
        let mut child = SplitMix64::new(epoch_seed);
        assert_ne!(child.next_u64(), next_epoch_seed);
    }

    #[test]
    fn sybil_stress_honors_the_configured_population_share() {
        let config = SimulationConfig {
            population: 100,
            scenario: Scenario::SybilStress,
            sybil_fraction_bps: 3_000,
            ..SimulationConfig::default()
        };
        let assigned = assign_strategies(&config, config.population as usize);
        let sybils = assigned
            .iter()
            .filter(|strategy| {
                matches!(
                    strategy,
                    Strategy::SacrificialSybil | Strategy::SelfDogpile | Strategy::BribedCoalition
                )
            })
            .count();
        assert_eq!(sybils, 30);
    }

    #[test]
    fn coalition_reports_separate_controllers_from_helpers() {
        let config = SimulationConfig {
            population: 30,
            epochs: 2,
            scenario: Scenario::SybilStress,
            sybil_fraction_bps: 3_000,
            ..SimulationConfig::default()
        };
        let report = simulate(config).unwrap();
        for strategy in [
            Strategy::SacrificialSybil,
            Strategy::SelfDogpile,
            Strategy::BribedCoalition,
        ] {
            let row = report
                .strategies
                .iter()
                .find(|row| row.strategy == strategy)
                .unwrap();
            assert_eq!(row.controller_player_epochs, 2);
            assert!(row.player_epochs > row.controller_player_epochs);
        }
    }

    #[test]
    fn canonical_tuning_matches_core_heat_and_guard() {
        let tuning = RuleTuning::default();
        let heat = rules::Heat {
            units: 7_500,
            updated_at: 100,
        };
        assert_eq!(
            tuned_heat_at(heat, 3_700, &tuning),
            rules::heat_at(heat, 3_700)
        );
        let charged = tuned_charge_heat(heat, 3_700, 10_000_000, rules::DUMP_TIER_SIZE, &tuning);
        assert_eq!(
            charged,
            rules::charge_heat(heat, 3_700, 10_000_000, rules::DUMP_TIER_SIZE).ok()
        );
        assert_eq!(
            tuned_guard_after(0, 100_000_000, rules::DUMP_TIER_SIZE, &tuning),
            rules::grant_guard(0, 100_000_000, rules::DUMP_TIER_SIZE).unwrap()
        );
    }

    #[test]
    fn simulation_is_reproducible() {
        let config = SimulationConfig {
            population: 20,
            epochs: 3,
            ..SimulationConfig::default()
        };
        let first = serde_json::to_vec(&simulate(config.clone()).unwrap()).unwrap();
        let second = serde_json::to_vec(&simulate(config).unwrap()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn random_only_conserves_and_rewards() {
        let config = SimulationConfig {
            population: 20,
            epochs: 2,
            scenario: Scenario::RandomOnly,
            ..SimulationConfig::default()
        };
        let report = simulate(config).unwrap();
        assert_eq!(report.totals.simulated_epochs, 2);
        assert!(report.totals.dump_actions > 0);
        assert!(report.totals.rewarded_player_epochs > 0);
        assert!(report.totals.glory_emitted > 0);
        assert!(report.totals.redirected_dump <= report.totals.guard_created);
        assert!(
            report.strategies[0]
                .break_even_glory_price_lamports
                .is_some()
        );
    }

    #[test]
    fn withholding_and_inactivity_do_not_become_eligible() {
        let config = SimulationConfig {
            population: 24,
            epochs: 1,
            sybil_fraction_bps: 0,
            ..SimulationConfig::default()
        };
        let report = simulate(config).unwrap();
        let inactive = report
            .strategies
            .iter()
            .find(|row| row.strategy == Strategy::Inactive)
            .unwrap();
        let withholder = report
            .strategies
            .iter()
            .find(|row| row.strategy == Strategy::RevealWithholder)
            .unwrap();
        assert_eq!(inactive.eligible_player_epochs, 0);
        assert_eq!(withholder.eligible_player_epochs, 0);
        assert_eq!(inactive.wins + withholder.wins, 0);
    }

    #[test]
    fn tuned_reports_are_labeled_noncanonical() {
        let mut config = SimulationConfig {
            population: 20,
            epochs: 1,
            ..SimulationConfig::default()
        };
        config.tuning.total_guard_cap_bps = 1_000;
        let report = simulate(config).unwrap();
        assert!(!report.canonical_rules);
        assert!(
            report
                .balance_signals
                .iter()
                .any(|signal| signal.code == "experimental_rules")
        );
    }

    #[test]
    fn legacy_json_config_defaults_to_the_canonical_research_boundary() {
        let raw = r#"{
            "population": 20,
            "epochs": 1,
            "seed": 7,
            "scenario": "mixed",
            "decision_interval_seconds": 21600,
            "information_delay_seconds": 0,
            "rpc_failure_bps": 0,
            "sybil_fraction_bps": 1000,
            "estimated_signature_fee_lamports": 5000,
            "hypothetical_glory_price_lamports": 0,
            "tuning": {
                "minimum_action_bps": 10,
                "heat_recovery_seconds": 21600,
                "guard_conversion_bps": 5000,
                "total_guard_cap_bps": 2500,
                "absorb_lock_seconds": 21600,
                "redirect_rearm_seconds": 900
            }
        }"#;
        let config: SimulationConfig = serde_json::from_str(raw).unwrap();
        assert!(config.research.is_canonical());
        assert!(simulate(config).unwrap().canonical_rules);
    }

    #[test]
    fn chapter_scoring_preserves_a_constant_balance() {
        for temporal_scoring in [
            TemporalScoring::EqualChapters,
            TemporalScoring::MildChapters,
        ] {
            let research = ResearchRules {
                temporal_scoring,
                ..ResearchRules::v4_candidate()
            };
            let cumulative = checkpoint_balance_area(
                0,
                rules::DUMP_TIER_SIZE,
                0,
                rules::ACTIVE_SECONDS,
                &research,
            )
            .unwrap();
            assert_eq!(
                score_from_area(cumulative, &research).unwrap(),
                rules::DUMP_TIER_SIZE
            );
        }
    }

    #[test]
    fn mild_chapters_limit_endgame_weight_to_one_point_two_five_x() {
        let research = ResearchRules {
            temporal_scoring: TemporalScoring::MildChapters,
            ..ResearchRules::v4_candidate()
        };
        assert_eq!(chapter_weight(0, &research), 10_000);
        assert_eq!(
            chapter_weight(research.chapter_count - 1, &research),
            12_500
        );
    }

    #[test]
    fn chapter_stamina_carries_at_most_one_unused_chapter() {
        let research = ResearchRules::v4_candidate();
        let mut player = Player::new(0, Strategy::Random, rules::DUMP_TIER_SIZE, true);
        let allowance = mul_bps(
            chapter_capacity_basis(rules::DUMP_TIER_SIZE, research.chapter_start_weight_bps),
            research.chapter_capacity_bps,
        );
        assert_eq!(
            player.available_capacity(CapacityChannel::Dump, 0, &research),
            allowance
        );
        player
            .charge_capacity(CapacityChannel::Dump, rules::DUMP_TIER_SIZE, &research)
            .unwrap();
        let chapter_two = rules::ACTIVE_SECONDS / i64::from(research.chapter_count) + 1;
        assert_eq!(
            player.available_capacity(CapacityChannel::Dump, chapter_two, &research),
            allowance + allowance.saturating_sub(rules::DUMP_TIER_SIZE)
        );
        let chapter_four = chapter_two * 3;
        assert_eq!(
            player.available_capacity(CapacityChannel::Dump, chapter_four, &research),
            allowance * 2
        );
    }

    #[test]
    fn chapter_capacity_basis_interpolates_fixed_and_start_scaled_models() {
        assert_eq!(
            chapter_capacity_basis(rules::DUMP_TIER_SIZE, 0),
            REFERENCE_ACTION_CAPACITY
        );
        assert_eq!(
            chapter_capacity_basis(rules::DUMP_TIER_SIZE, rules::BPS_DENOMINATOR),
            rules::DUMP_TIER_SIZE
        );
        assert_eq!(
            chapter_capacity_basis(rules::DUMP_TIER_SIZE, 5_000),
            3_250_000_000
        );
    }

    #[test]
    fn absorb_relief_cap_is_shared_by_all_helpers_targeting_one_player() {
        let config = SimulationConfig {
            population: 3,
            epochs: 1,
            research: ResearchRules::v4_candidate(),
            ..SimulationConfig::default()
        };
        let mut players = vec![
            Player::new(
                0,
                Strategy::SacrificialSybil,
                rules::MAX_STARTING_DUMP,
                true,
            ),
            Player::new(
                1,
                Strategy::SacrificialSybil,
                rules::MAX_STARTING_DUMP,
                true,
            ),
            Player::new(
                2,
                Strategy::SacrificialSybil,
                rules::MAX_STARTING_DUMP,
                true,
            ),
        ];
        let before = players[0]
            .lanes
            .iter()
            .map(|lane| lane.balance)
            .sum::<u64>();
        let mut credit = HashMap::new();
        let mut actions = HashMap::new();
        let mut counters = EpochCounters {
            incoming_by_player: vec![0; players.len()],
            ..EpochCounters::default()
        };
        execute_absorb(
            &mut players,
            1,
            0,
            100,
            1,
            &config,
            &mut credit,
            &mut actions,
            &mut counters,
        )
        .unwrap();
        execute_absorb(
            &mut players,
            2,
            0,
            100,
            1,
            &config,
            &mut credit,
            &mut actions,
            &mut counters,
        )
        .unwrap();
        let after = players[0]
            .lanes
            .iter()
            .map(|lane| lane.balance)
            .sum::<u64>();
        assert_eq!(
            before - after,
            mul_bps(
                rules::MAX_STARTING_DUMP,
                config.research.target_absorb_relief_cap_bps_per_chapter
            )
        );
        assert_eq!(counters.absorb_actions, 1);
        assert_eq!(counters.failed_relief_cap_actions, 1);
    }

    #[test]
    fn inbound_cap_is_target_wide_but_not_part_of_the_v4_candidate() {
        let mut research = ResearchRules::v4_candidate();
        assert_eq!(research.target_inbound_cap_bps_per_chapter, 0);
        research.target_inbound_cap_bps_per_chapter = 10_000;
        research.chapter_capacity_bps = 20_000;
        let config = SimulationConfig {
            population: 2,
            epochs: 1,
            research,
            ..SimulationConfig::default()
        };
        let mut players = vec![
            Player::new(0, Strategy::GreedyDumper, rules::MAX_STARTING_DUMP, true),
            Player::new(1, Strategy::Random, rules::MAX_STARTING_DUMP, true),
        ];
        let before = players[1]
            .lanes
            .iter()
            .map(|lane| lane.balance)
            .sum::<u64>();
        let mut credit = HashMap::new();
        let mut actions = HashMap::new();
        let mut counters = EpochCounters {
            incoming_by_player: vec![0; players.len()],
            ..EpochCounters::default()
        };
        for _ in 0..5 {
            execute_dump(
                &mut players,
                0,
                1,
                100,
                1,
                7,
                &config,
                &mut credit,
                &mut actions,
                &mut counters,
            )
            .unwrap();
        }
        let after = players[1]
            .lanes
            .iter()
            .map(|lane| lane.balance)
            .sum::<u64>();
        assert_eq!(after - before, rules::MAX_STARTING_DUMP);
        assert_eq!(counters.dump_actions, 4);
        assert_eq!(counters.failed_inbound_cap_actions, 1);
    }

    #[test]
    fn v4_expands_exact_simulation_without_claiming_billion_player_execution() {
        let canonical = SimulationConfig {
            population: rules::MAX_PARTICIPANTS + 1,
            ..SimulationConfig::default()
        };
        assert!(canonical.validate().is_err());
        let research = SimulationConfig {
            population: 50_000,
            research: ResearchRules::v4_candidate(),
            ..SimulationConfig::default()
        };
        assert!(research.validate().is_ok());
        assert_eq!(simulated_winner_count(50_000, &research.research), 2_500);
    }

    #[test]
    fn global_scale_model_exposes_three_day_billion_player_pressure() {
        let report = model_global_scale(GlobalScaleConfig::default()).unwrap();
        let seven_billion = report
            .rows
            .iter()
            .find(|row| row.population == 7_000_000_000)
            .unwrap();
        assert!((seven_billion.average_intents_per_second - 27_006.172_8).abs() < 0.001);
        assert_eq!(
            seven_billion.disposition,
            ScaleDisposition::RequiresAggregatedSettlement
        );
        assert_eq!(seven_billion.aggregate_proofs_per_round, 700_000);
        assert_eq!(
            seven_billion.signed_intent_data_bytes_per_round,
            896_000_000_000
        );
        assert_eq!(
            seven_billion.minimum_direct_action_interval_seconds,
            7_000_000
        );
    }

    #[test]
    fn v4_scheduled_simulation_is_reproducible_and_explicitly_experimental() {
        let config = SimulationConfig {
            population: 20,
            epochs: 2,
            research: ResearchRules::v4_candidate(),
            ..SimulationConfig::default()
        };
        let first = serde_json::to_vec(&simulate(config.clone()).unwrap()).unwrap();
        let second_report = simulate(config).unwrap();
        assert_eq!(first, serde_json::to_vec(&second_report).unwrap());
        assert!(!second_report.canonical_rules);
        assert_eq!(
            second_report
                .totals
                .mean_chapter_boundary_turnover_rates
                .len(),
            5
        );
    }

    #[test]
    fn coarse_windows_with_no_endgame_action_have_zero_endgame_turnover() {
        let config = SimulationConfig {
            population: 20,
            epochs: 1,
            scenario: Scenario::RandomOnly,
            decision_interval_seconds: 5 * 24 * 60 * 60,
            research: ResearchRules::v4_candidate(),
            ..SimulationConfig::default()
        };
        let report = simulate(config).unwrap();
        assert_eq!(report.totals.final_window_winner_turnover_rate, 0.0);
    }
}
