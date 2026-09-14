use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use glory_dump_sim::{
    ActionCapacity, BalanceSignal, ExecutionModel, GlobalScaleConfig, ResearchRules, Scenario,
    SimulationConfig, SimulationError, StrategyReport, TemporalScoring, TierReport, TotalsReport,
    model_global_scale, simulate,
};
use serde::Serialize;

const HELP: &str = r#"GLORY/DUMP deterministic economic simulator

USAGE
  cargo run -p glory-dump-sim -- run [OPTIONS]
  cargo run -p glory-dump-sim -- sweep [OPTIONS]
  cargo run -p glory-dump-sim -- v4-sweep [OPTIONS]
  cargo run -p glory-dump-sim -- scale [OPTIONS]

RUN OPTIONS
  --config PATH                 Load a JSON SimulationConfig before applying flags
  --population N                Players per epoch (v3: 2..2560; research: 2..100000)
  --epochs N                    Number of epochs
  --seed N                      Deterministic u64 seed (decimal or 0x-prefixed)
  --scenario NAME               mixed, random_only, leader_hunt, guard_heavy,
                                last_window, or sybil_stress
  --decision-interval SECONDS   Bot decision cadence
  --information-delay SECONDS   Age of standings visible to bots
  --rpc-failure-bps N           Simulated failed submissions
  --sybil-bps N                 Mixed-room Sybil population share
  --glory-price-lamports N      Hypothetical value of one whole GLORY
  --minimum-action-bps N        Experimental rule override
  --heat-recovery-seconds N     Experimental rule override
  --guard-conversion-bps N      Experimental rule override
  --guard-cap-bps N             Experimental rule override
  --absorb-lock-seconds N       Experimental rule override
  --redirect-rearm-seconds N    Experimental rule override
  --v4                          Apply the simulator-only v4 candidate rules
  --temporal-scoring NAME       canonical_late_weighted, equal_chapters, mild_chapters
  --action-capacity NAME        canonical_heat or chapter_stamina
  --execution-model NAME        sequential or scheduled_batch
  --chapters N                  Number of equal-duration chapters (2..12)
  --chapter-capacity-bps N      Stamina per chapter over blended capacity basis
  --chapter-start-weight-bps N  0=fixed 5.5B basis; 10000=fully start-scaled
  --chapter-carry-bps N         Unused prior-chapter capacity allowed to carry
  --intents-per-window N        Maximum signed intents submitted per scheduled window
  --absorb-relief-cap-bps N     Target-wide ABSORB relief per chapter
  --inbound-cap-bps N           Target-wide landed DUMP per chapter
  --coordination-cost N         Modeled lamports per coalition helper per epoch
  --output PATH                 Write JSON atomically instead of stdout
  --compact                     Disable pretty JSON

SWEEP
  Runs a deterministic matrix over room size, information delay, RPC failure,
  Sybil share, Guard cap, Heat recovery, and endgame-heavy behavior. --epochs,
  --seed, --output, and --compact are accepted.

V4-SWEEP
  Compares the v4 candidate against scoring, cadence, relief-cap, and Sybil
  ablations. The v4 rules exist only in this simulator.

SCALE OPTIONS
  --scale-populations LIST      Comma-separated global populations
  --action-interval SECONDS     One signed intent per player per interval
  --modeled-direct-aps N        Planning budget for direct L1 actions/second
  --aggregate-size N            Signed intents represented by one proof settlement
  --intent-bytes N              Modeled off-chain bytes per signed intent
"#;

#[derive(Serialize)]
struct SweepReport {
    schema: &'static str,
    seed: u64,
    epochs_per_cell: u32,
    cells: Vec<SweepCell>,
}

#[derive(Serialize)]
struct SweepCell {
    label: String,
    config: SimulationConfig,
    totals: TotalsReport,
    strategies: Vec<StrategyReport>,
    starting_tiers: Vec<TierReport>,
    signals: Vec<BalanceSignal>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), SimulationError> {
    let mut arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.is_empty() || matches!(arguments[0].as_str(), "help" | "--help" | "-h") {
        print!("{HELP}");
        return Ok(());
    }
    let command = arguments.remove(0);
    if arguments.len() == 1 && matches!(arguments[0].as_str(), "help" | "--help" | "-h") {
        print!("{HELP}");
        return Ok(());
    }
    let options = Options::parse(&arguments)?;
    match command.as_str() {
        "run" => {
            let config = options.config()?;
            let report = simulate(config)?;
            emit_json(&report, options.output.as_deref(), options.pretty)?;
        }
        "sweep" => {
            let report = run_sweep(&options)?;
            emit_json(&report, options.output.as_deref(), options.pretty)?;
        }
        "v4-sweep" => {
            let report = run_v4_sweep(&options)?;
            emit_json(&report, options.output.as_deref(), options.pretty)?;
        }
        "scale" => {
            let report = model_global_scale(options.scale_config()?)?;
            emit_json(&report, options.output.as_deref(), options.pretty)?;
        }
        _ => {
            return Err(SimulationError::InvalidConfig(format!(
                "unknown command {command:?}; use run, sweep, v4-sweep, scale, or --help"
            )));
        }
    }
    Ok(())
}

#[derive(Default)]
struct Options {
    config_path: Option<PathBuf>,
    output: Option<PathBuf>,
    pretty: bool,
    population: Option<u32>,
    epochs: Option<u32>,
    seed: Option<u64>,
    scenario: Option<Scenario>,
    decision_interval_seconds: Option<i64>,
    information_delay_seconds: Option<i64>,
    rpc_failure_bps: Option<u64>,
    sybil_fraction_bps: Option<u64>,
    glory_price_lamports: Option<u64>,
    minimum_action_bps: Option<u64>,
    heat_recovery_seconds: Option<i64>,
    guard_conversion_bps: Option<u64>,
    guard_cap_bps: Option<u64>,
    absorb_lock_seconds: Option<i64>,
    redirect_rearm_seconds: Option<i64>,
    v4: bool,
    temporal_scoring: Option<TemporalScoring>,
    action_capacity: Option<ActionCapacity>,
    execution_model: Option<ExecutionModel>,
    chapters: Option<u8>,
    chapter_capacity_bps: Option<u64>,
    chapter_start_weight_bps: Option<u64>,
    chapter_carry_bps: Option<u64>,
    intents_per_window: Option<u8>,
    absorb_relief_cap_bps: Option<u64>,
    inbound_cap_bps: Option<u64>,
    coordination_cost_lamports: Option<u64>,
    scale_populations: Option<Vec<u64>>,
    action_interval_seconds: Option<u64>,
    modeled_direct_actions_per_second: Option<u64>,
    aggregate_size: Option<u64>,
    intent_bytes: Option<u64>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, SimulationError> {
        let mut options = Self {
            pretty: true,
            ..Self::default()
        };
        let mut index = 0;
        while index < arguments.len() {
            let flag = arguments[index].as_str();
            if matches!(flag, "--compact" | "--v4") {
                if flag == "--compact" {
                    options.pretty = false;
                } else {
                    options.v4 = true;
                }
                index += 1;
                continue;
            }
            let value = arguments.get(index + 1).ok_or_else(|| {
                SimulationError::InvalidConfig(format!("{flag} requires a value"))
            })?;
            match flag {
                "--config" => options.config_path = Some(PathBuf::from(value)),
                "--output" => options.output = Some(PathBuf::from(value)),
                "--population" => options.population = Some(parse(value, flag)?),
                "--epochs" => options.epochs = Some(parse(value, flag)?),
                "--seed" => options.seed = Some(parse_u64(value, flag)?),
                "--scenario" => options.scenario = Some(parse_scenario(value)?),
                "--decision-interval" => {
                    options.decision_interval_seconds = Some(parse(value, flag)?)
                }
                "--information-delay" => {
                    options.information_delay_seconds = Some(parse(value, flag)?)
                }
                "--rpc-failure-bps" => options.rpc_failure_bps = Some(parse(value, flag)?),
                "--sybil-bps" => options.sybil_fraction_bps = Some(parse(value, flag)?),
                "--glory-price-lamports" => {
                    options.glory_price_lamports = Some(parse(value, flag)?)
                }
                "--minimum-action-bps" => options.minimum_action_bps = Some(parse(value, flag)?),
                "--heat-recovery-seconds" => {
                    options.heat_recovery_seconds = Some(parse(value, flag)?)
                }
                "--guard-conversion-bps" => {
                    options.guard_conversion_bps = Some(parse(value, flag)?)
                }
                "--guard-cap-bps" => options.guard_cap_bps = Some(parse(value, flag)?),
                "--absorb-lock-seconds" => options.absorb_lock_seconds = Some(parse(value, flag)?),
                "--redirect-rearm-seconds" => {
                    options.redirect_rearm_seconds = Some(parse(value, flag)?)
                }
                "--temporal-scoring" => {
                    options.temporal_scoring = Some(parse_temporal_scoring(value)?)
                }
                "--action-capacity" => {
                    options.action_capacity = Some(parse_action_capacity(value)?)
                }
                "--execution-model" => {
                    options.execution_model = Some(parse_execution_model(value)?)
                }
                "--chapters" => options.chapters = Some(parse(value, flag)?),
                "--chapter-capacity-bps" => {
                    options.chapter_capacity_bps = Some(parse(value, flag)?)
                }
                "--chapter-start-weight-bps" => {
                    options.chapter_start_weight_bps = Some(parse(value, flag)?)
                }
                "--chapter-carry-bps" => options.chapter_carry_bps = Some(parse(value, flag)?),
                "--intents-per-window" => options.intents_per_window = Some(parse(value, flag)?),
                "--absorb-relief-cap-bps" => {
                    options.absorb_relief_cap_bps = Some(parse(value, flag)?)
                }
                "--inbound-cap-bps" => options.inbound_cap_bps = Some(parse(value, flag)?),
                "--coordination-cost" => {
                    options.coordination_cost_lamports = Some(parse(value, flag)?)
                }
                "--scale-populations" => {
                    options.scale_populations = Some(parse_u64_list(value, flag)?)
                }
                "--action-interval" => options.action_interval_seconds = Some(parse(value, flag)?),
                "--modeled-direct-aps" => {
                    options.modeled_direct_actions_per_second = Some(parse(value, flag)?)
                }
                "--aggregate-size" => options.aggregate_size = Some(parse(value, flag)?),
                "--intent-bytes" => options.intent_bytes = Some(parse(value, flag)?),
                _ => {
                    return Err(SimulationError::InvalidConfig(format!(
                        "unknown option {flag:?}"
                    )));
                }
            }
            index += 2;
        }
        Ok(options)
    }

    fn config(&self) -> Result<SimulationConfig, SimulationError> {
        let mut config = if let Some(path) = &self.config_path {
            serde_json::from_slice(&fs::read(path)?)?
        } else {
            SimulationConfig::default()
        };
        if self.v4 {
            config.research = ResearchRules::v4_candidate();
        }
        if let Some(value) = self.population {
            config.population = value;
        }
        if let Some(value) = self.epochs {
            config.epochs = value;
        }
        if let Some(value) = self.seed {
            config.seed = value;
        }
        if let Some(value) = self.scenario {
            config.scenario = value;
        }
        if let Some(value) = self.decision_interval_seconds {
            config.decision_interval_seconds = value;
        }
        if let Some(value) = self.information_delay_seconds {
            config.information_delay_seconds = value;
        }
        if let Some(value) = self.rpc_failure_bps {
            config.rpc_failure_bps = value;
        }
        if let Some(value) = self.sybil_fraction_bps {
            config.sybil_fraction_bps = value;
        }
        if let Some(value) = self.glory_price_lamports {
            config.hypothetical_glory_price_lamports = value;
        }
        if let Some(value) = self.minimum_action_bps {
            config.tuning.minimum_action_bps = value;
        }
        if let Some(value) = self.heat_recovery_seconds {
            config.tuning.heat_recovery_seconds = value;
        }
        if let Some(value) = self.guard_conversion_bps {
            config.tuning.guard_conversion_bps = value;
        }
        if let Some(value) = self.guard_cap_bps {
            config.tuning.total_guard_cap_bps = value;
        }
        if let Some(value) = self.absorb_lock_seconds {
            config.tuning.absorb_lock_seconds = value;
        }
        if let Some(value) = self.redirect_rearm_seconds {
            config.tuning.redirect_rearm_seconds = value;
        }
        if let Some(value) = self.temporal_scoring {
            config.research.temporal_scoring = value;
        }
        if let Some(value) = self.action_capacity {
            config.research.action_capacity = value;
        }
        if let Some(value) = self.execution_model {
            config.research.execution_model = value;
        }
        if let Some(value) = self.chapters {
            config.research.chapter_count = value;
        }
        if let Some(value) = self.chapter_capacity_bps {
            config.research.chapter_capacity_bps = value;
        }
        if let Some(value) = self.chapter_start_weight_bps {
            config.research.chapter_start_weight_bps = value;
        }
        if let Some(value) = self.chapter_carry_bps {
            config.research.chapter_carry_bps = value;
        }
        if let Some(value) = self.intents_per_window {
            config.research.max_intents_per_window = value;
        }
        if let Some(value) = self.absorb_relief_cap_bps {
            config.research.target_absorb_relief_cap_bps_per_chapter = value;
        }
        if let Some(value) = self.inbound_cap_bps {
            config.research.target_inbound_cap_bps_per_chapter = value;
        }
        if let Some(value) = self.coordination_cost_lamports {
            config
                .research
                .coalition_coordination_cost_lamports_per_helper_epoch = value;
        }
        config.validate()?;
        Ok(config)
    }

    fn scale_config(&self) -> Result<GlobalScaleConfig, SimulationError> {
        let mut config = GlobalScaleConfig::default();
        if let Some(value) = &self.scale_populations {
            config.populations.clone_from(value);
        }
        if let Some(value) = self.action_interval_seconds {
            config.action_interval_seconds = value;
        }
        if let Some(value) = self.modeled_direct_actions_per_second {
            config.modeled_direct_actions_per_second = value;
        }
        if let Some(value) = self.aggregate_size {
            config.intents_per_aggregate_proof = value;
        }
        if let Some(value) = self.intent_bytes {
            config.signed_intent_bytes = value;
        }
        config.validate()?;
        Ok(config)
    }
}

fn run_sweep(options: &Options) -> Result<SweepReport, SimulationError> {
    let seed = options.seed.unwrap_or(SimulationConfig::default().seed);
    let epochs = options.epochs.unwrap_or(25);
    let mut cells = Vec::new();
    let mut cases = Vec::new();

    for population in [20, 100, 500] {
        cases.push((
            format!("population_{population}"),
            SimulationConfig {
                population,
                epochs,
                seed: seed ^ u64::from(population),
                ..SimulationConfig::default()
            },
        ));
    }
    cases.push((
        "one_hour_information_delay".into(),
        SimulationConfig {
            epochs,
            seed: seed ^ 0x11,
            information_delay_seconds: 3_600,
            ..SimulationConfig::default()
        },
    ));
    cases.push((
        "five_percent_rpc_failure".into(),
        SimulationConfig {
            epochs,
            seed: seed ^ 0x22,
            rpc_failure_bps: 500,
            ..SimulationConfig::default()
        },
    ));
    cases.push((
        "thirty_percent_sybil".into(),
        SimulationConfig {
            epochs,
            seed: seed ^ 0x33,
            sybil_fraction_bps: 3_000,
            scenario: Scenario::SybilStress,
            ..SimulationConfig::default()
        },
    ));
    for (label, scenario) in [
        ("random_only", Scenario::RandomOnly),
        ("leader_hunt", Scenario::LeaderHunt),
        ("guard_heavy", Scenario::GuardHeavy),
    ] {
        cases.push((
            label.into(),
            SimulationConfig {
                epochs,
                seed: seed ^ scenario as u64 ^ 0x55,
                scenario,
                ..SimulationConfig::default()
            },
        ));
    }
    for (label, cap) in [
        ("guard_cap_10_percent", 1_000),
        ("guard_cap_40_percent", 4_000),
    ] {
        let mut config = SimulationConfig {
            epochs,
            seed: seed ^ cap,
            ..SimulationConfig::default()
        };
        config.tuning.total_guard_cap_bps = cap;
        cases.push((label.into(), config));
    }
    for (label, recovery) in [
        ("heat_recovery_3_hours", 10_800),
        ("heat_recovery_12_hours", 43_200),
    ] {
        let mut config = SimulationConfig {
            epochs,
            seed: seed ^ recovery as u64,
            ..SimulationConfig::default()
        };
        config.tuning.heat_recovery_seconds = recovery;
        cases.push((label.into(), config));
    }
    cases.push((
        "last_window_pressure".into(),
        SimulationConfig {
            epochs,
            seed: seed ^ 0x44,
            scenario: Scenario::LastWindow,
            ..SimulationConfig::default()
        },
    ));

    for (label, config) in cases {
        let report = simulate(config)?;
        cells.push(SweepCell {
            label,
            config: report.config,
            totals: report.totals,
            strategies: report.strategies,
            starting_tiers: report.starting_tiers,
            signals: report.balance_signals,
        });
    }
    Ok(SweepReport {
        schema: "glory-dump-simulation-sweep-v1",
        seed,
        epochs_per_cell: epochs,
        cells,
    })
}

fn run_v4_sweep(options: &Options) -> Result<SweepReport, SimulationError> {
    let seed = options.seed.unwrap_or(SimulationConfig::default().seed);
    let epochs = options.epochs.unwrap_or(25);
    let mut baseline = SimulationConfig {
        population: options.population.unwrap_or(100),
        epochs,
        seed,
        hypothetical_glory_price_lamports: options.glory_price_lamports.unwrap_or(0),
        research: ResearchRules::v4_candidate(),
        ..SimulationConfig::default()
    };
    let mut cases = vec![("v4_candidate".to_owned(), baseline.clone())];

    let mut equal = baseline.clone();
    equal.seed ^= 0x101;
    equal.research.temporal_scoring = TemporalScoring::EqualChapters;
    cases.push(("equal_chapter_scoring".into(), equal));

    let mut canonical_score = baseline.clone();
    canonical_score.seed ^= 0x202;
    canonical_score.research.temporal_scoring = TemporalScoring::CanonicalLateWeighted;
    cases.push(("canonical_late_weighted_scoring".into(), canonical_score));

    let mut fixed_capacity = baseline.clone();
    fixed_capacity.seed ^= 0x252;
    fixed_capacity.research.chapter_start_weight_bps = 0;
    cases.push(("fixed_global_capacity_basis".into(), fixed_capacity));

    let mut start_scaled_capacity = baseline.clone();
    start_scaled_capacity.seed ^= 0x272;
    start_scaled_capacity.research.chapter_start_weight_bps = 10_000;
    cases.push((
        "fully_start_scaled_capacity_basis".into(),
        start_scaled_capacity,
    ));

    let mut no_relief_cap = baseline.clone();
    no_relief_cap.seed ^= 0x303;
    no_relief_cap
        .research
        .target_absorb_relief_cap_bps_per_chapter = 0;
    cases.push(("no_target_relief_cap".into(), no_relief_cap));

    let mut inbound_cap = baseline.clone();
    inbound_cap.seed ^= 0x353;
    inbound_cap.research.target_inbound_cap_bps_per_chapter = 10_000;
    cases.push(("target_inbound_cap_one_start".into(), inbound_cap));

    let mut sybil = baseline.clone();
    sybil.seed ^= 0x404;
    sybil.scenario = Scenario::SybilStress;
    sybil.sybil_fraction_bps = 3_000;
    cases.push(("thirty_percent_sybil".into(), sybil));

    let mut sybil_no_cap = baseline.clone();
    sybil_no_cap.seed ^= 0x505;
    sybil_no_cap.scenario = Scenario::SybilStress;
    sybil_no_cap.sybil_fraction_bps = 3_000;
    sybil_no_cap
        .research
        .target_absorb_relief_cap_bps_per_chapter = 0;
    sybil_no_cap.research.target_inbound_cap_bps_per_chapter = 0;
    cases.push(("thirty_percent_sybil_no_target_caps".into(), sybil_no_cap));

    let mut sybil_inbound_cap = baseline.clone();
    sybil_inbound_cap.seed ^= 0x555;
    sybil_inbound_cap.scenario = Scenario::SybilStress;
    sybil_inbound_cap.sybil_fraction_bps = 3_000;
    sybil_inbound_cap
        .research
        .target_inbound_cap_bps_per_chapter = 10_000;
    cases.push((
        "thirty_percent_sybil_with_inbound_cap".into(),
        sybil_inbound_cap,
    ));

    for (label, cadence, salt) in [
        ("hourly_windows", 60 * 60, 0x606),
        ("daily_windows", 24 * 60 * 60, 0x707),
        ("five_day_windows", 5 * 24 * 60 * 60, 0x808),
    ] {
        let mut config = baseline.clone();
        config.seed ^= salt;
        config.decision_interval_seconds = cadence;
        cases.push((label.into(), config));
    }

    baseline.seed ^= 0x909;
    baseline.population = 500;
    cases.push(("population_500".into(), baseline));

    let mut cells = Vec::with_capacity(cases.len());
    for (label, config) in cases {
        let report = simulate(config)?;
        cells.push(SweepCell {
            label,
            config: report.config,
            totals: report.totals,
            strategies: report.strategies,
            starting_tiers: report.starting_tiers,
            signals: report.balance_signals,
        });
    }
    Ok(SweepReport {
        schema: "glory-dump-v4-research-sweep-v1",
        seed,
        epochs_per_cell: epochs,
        cells,
    })
}

fn emit_json<T: Serialize>(
    value: &T,
    output: Option<&Path>,
    pretty: bool,
) -> Result<(), SimulationError> {
    let mut bytes = if pretty {
        serde_json::to_vec_pretty(value)?
    } else {
        serde_json::to_vec(value)?
    };
    bytes.push(b'\n');
    if let Some(path) = output {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("report.json");
        let temporary = path.with_file_name(format!(".{file_name}.tmp"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&temporary, &bytes)?;
        fs::rename(temporary, path)?;
    } else {
        io::stdout().write_all(&bytes)?;
    }
    Ok(())
}

fn parse<T>(raw: &str, flag: &str) -> Result<T, SimulationError>
where
    T: std::str::FromStr,
    T::Err: Display,
{
    raw.parse().map_err(|error| {
        SimulationError::InvalidConfig(format!("invalid value for {flag}: {error}"))
    })
}

fn parse_u64(raw: &str, flag: &str) -> Result<u64, SimulationError> {
    if let Some(hex) = raw.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).map_err(|error| {
            SimulationError::InvalidConfig(format!("invalid value for {flag}: {error}"))
        })
    } else {
        parse(raw, flag)
    }
}

fn parse_scenario(raw: &str) -> Result<Scenario, SimulationError> {
    match raw.replace('-', "_").as_str() {
        "mixed" => Ok(Scenario::Mixed),
        "random_only" => Ok(Scenario::RandomOnly),
        "leader_hunt" => Ok(Scenario::LeaderHunt),
        "guard_heavy" => Ok(Scenario::GuardHeavy),
        "last_window" => Ok(Scenario::LastWindow),
        "sybil_stress" => Ok(Scenario::SybilStress),
        _ => Err(SimulationError::InvalidConfig(format!(
            "unknown scenario {raw:?}"
        ))),
    }
}

fn parse_temporal_scoring(raw: &str) -> Result<TemporalScoring, SimulationError> {
    match raw.replace('-', "_").as_str() {
        "canonical_late_weighted" => Ok(TemporalScoring::CanonicalLateWeighted),
        "equal_chapters" => Ok(TemporalScoring::EqualChapters),
        "mild_chapters" => Ok(TemporalScoring::MildChapters),
        _ => Err(SimulationError::InvalidConfig(format!(
            "unknown temporal scoring model {raw:?}"
        ))),
    }
}

fn parse_action_capacity(raw: &str) -> Result<ActionCapacity, SimulationError> {
    match raw.replace('-', "_").as_str() {
        "canonical_heat" => Ok(ActionCapacity::CanonicalHeat),
        "chapter_stamina" => Ok(ActionCapacity::ChapterStamina),
        _ => Err(SimulationError::InvalidConfig(format!(
            "unknown action capacity model {raw:?}"
        ))),
    }
}

fn parse_execution_model(raw: &str) -> Result<ExecutionModel, SimulationError> {
    match raw.replace('-', "_").as_str() {
        "sequential" => Ok(ExecutionModel::Sequential),
        "scheduled_batch" => Ok(ExecutionModel::ScheduledBatch),
        _ => Err(SimulationError::InvalidConfig(format!(
            "unknown execution model {raw:?}"
        ))),
    }
}

fn parse_u64_list(raw: &str, flag: &str) -> Result<Vec<u64>, SimulationError> {
    let values = raw
        .split(',')
        .map(|value| parse_u64(value.trim(), flag))
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        return Err(SimulationError::InvalidConfig(format!(
            "{flag} requires at least one value"
        )));
    }
    Ok(values)
}
