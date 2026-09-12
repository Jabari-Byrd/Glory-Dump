use anchor_lang::prelude::*;
pub use glory_dump_core::{
    BADGE_ESCAPE_ARTIST, BADGE_GARBAGE_EMPEROR, BADGE_HUMAN_SHIELD, BADGE_LAST_LAUGH,
    BADGE_RICOCHET,
};
use glory_dump_core::{Heat, MAX_WINNERS, Standing, compare_standings, is_better};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Registration,
    Reveal,
    Planning,
    Active,
    Settling,
    Complete,
    Cancelled,
}

#[account]
#[derive(Debug)]
pub struct Protocol {
    pub version: u16,
    pub current_epoch: u64,
    pub glory_mint: Pubkey,
    pub glory_committed: u64,
    pub bump: u8,
}

impl Protocol {
    pub const VERSION: u16 = 3;
    pub const DATA_LEN: usize = 2 + 8 + 32 + 8 + 1;
    pub const SPACE: usize = 8 + Self::DATA_LEN;
}

#[account]
#[derive(Debug)]
pub struct Epoch {
    pub number: u64,
    pub phase: Phase,
    pub registration_started_at: i64,
    pub registration_ends_at: i64,
    pub reveal_ends_at: i64,
    pub active_starts_at: i64,
    pub active_ends_at: i64,
    pub entropy: [u8; 32],
    pub seed: [u8; 32],
    pub participant_count: u32,
    pub revealed_count: u32,
    pub settled_count: u32,
    pub eligible_count: u32,
    pub winner_count: u16,
    pub total_reward_pool: u64,
    pub player_reward_pool: u64,
    pub keeper_reward_pool: u64,
    pub worst_player: Pubkey,
    pub worst_score: u64,
    pub bonds_collected: u64,
    pub bond_refunds_paid: u64,
    pub settlement_bounties_paid: u64,
    pub bond_claim_ends_at: i64,
    pub swept_excess_bonds: u64,
    pub bump: u8,
}

impl Epoch {
    pub const DATA_LEN: usize = 236;
    pub const SPACE: usize = 8 + Self::DATA_LEN;

    pub fn registration_open(&self, now: i64) -> bool {
        self.phase == Phase::Registration && now < self.registration_ends_at
    }

    pub fn gameplay_open(&self, now: i64) -> bool {
        self.phase == Phase::Active && now >= self.active_starts_at && now < self.active_ends_at
    }
}

#[account]
#[derive(Debug)]
pub struct PlayerEpoch {
    pub epoch: u64,
    pub owner: Pubkey,
    pub commitment: [u8; 32],
    pub revealed: bool,
    pub allocation_claimed: bool,
    pub settled: bool,
    pub bond_claimed: bool,
    pub starting_allocation: u64,
    pub final_score: u64,
    pub dump_heat: HeatState,
    pub absorb_heat: HeatState,
    pub impact_ppm: u64,
    pub late_impact_ppm: u64,
    pub meaningful_actions: u32,
    pub distinct_opponents: u16,
    pub session_delegate: Pubkey,
    pub session_expires_at: i64,
    pub session_actions_remaining: u16,
    pub badges: u64,
    pub bump: u8,
}

impl PlayerEpoch {
    pub const DATA_LEN: usize = 189;
    pub const SPACE: usize = 8 + Self::DATA_LEN;

    pub fn eligible(&self) -> bool {
        self.revealed && self.meaningful_actions > 0
    }

    pub fn authorize_action(&mut self, signer: Pubkey, now: i64) -> Result<()> {
        if signer == self.owner {
            return Ok(());
        }
        require!(
            signer == self.session_delegate
                && now < self.session_expires_at
                && self.session_actions_remaining > 0,
            crate::errors::GloryDumpError::Unauthorized
        );
        self.session_actions_remaining = self
            .session_actions_remaining
            .checked_sub(1)
            .ok_or(crate::errors::GloryDumpError::ArithmeticOverflow)?;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeatState {
    pub units: u32,
    pub updated_at: i64,
}

impl From<HeatState> for Heat {
    fn from(value: HeatState) -> Self {
        Self {
            units: value.units,
            updated_at: value.updated_at,
        }
    }
}

impl From<Heat> for HeatState {
    fn from(value: Heat) -> Self {
        Self {
            units: value.units,
            updated_at: value.updated_at,
        }
    }
}

#[account]
#[derive(Debug)]
pub struct BalanceLane {
    pub epoch: u64,
    pub owner: Pubkey,
    pub index: u8,
    pub balance: u64,
    pub guard: u64,
    pub locked_amount: u64,
    pub locked_until: i64,
    pub redirect_armed: bool,
    pub redirect_ready_at: i64,
    pub redirected_volume: u64,
    pub cumulative_weighted: u128,
    pub last_checkpoint_at: i64,
    pub bump: u8,
}

impl BalanceLane {
    pub const DATA_LEN: usize = 115;
    pub const SPACE: usize = 8 + Self::DATA_LEN;

    pub fn release_expired_lock(&mut self, now: i64) {
        if now >= self.locked_until {
            self.locked_amount = 0;
        }
    }

    pub fn spendable(&self, now: i64) -> u64 {
        if now >= self.locked_until {
            self.balance
        } else {
            self.balance.saturating_sub(self.locked_amount)
        }
    }

    pub fn checkpoint(&mut self, now: i64, epoch: &Epoch) -> Result<()> {
        self.cumulative_weighted = glory_dump_core::checkpoint_weighted_balance(
            self.cumulative_weighted,
            self.balance,
            self.last_checkpoint_at,
            now,
            epoch.active_starts_at,
            epoch.active_ends_at,
        )
        .map_err(crate::errors::map_rule_error)?;
        self.last_checkpoint_at = now.clamp(epoch.active_starts_at, epoch.active_ends_at);
        self.release_expired_lock(now);
        Ok(())
    }

    pub fn score(&self, epoch: &Epoch) -> Result<u64> {
        glory_dump_core::weighted_score(
            self.cumulative_weighted,
            epoch.active_ends_at - epoch.active_starts_at,
        )
        .map_err(crate::errors::map_rule_error)
    }
}

#[account]
#[derive(Debug)]
pub struct Rivalry {
    pub epoch: u64,
    pub actor: Pubkey,
    pub target: Pubkey,
    pub rent_payer: Pubkey,
    pub impact_credited_ppm: u64,
    pub action_count: u32,
    pub bump: u8,
}

impl Rivalry {
    pub const DATA_LEN: usize = 117;
    pub const SPACE: usize = 8 + Self::DATA_LEN;
}

#[account]
#[derive(Debug)]
pub struct KeeperCredit {
    pub epoch: u64,
    pub keeper: Pubkey,
    pub players_settled: u32,
    pub claimed: bool,
    pub bump: u8,
}

impl KeeperCredit {
    pub const DATA_LEN: usize = 46;
    pub const SPACE: usize = 8 + Self::DATA_LEN;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WinnerEntry {
    pub player: Pubkey,
    pub score: u64,
    pub impact_ppm: u64,
    pub distinct_opponents: u16,
    pub tie_breaker: u64,
    pub claimed: bool,
}

impl WinnerEntry {
    pub const SIZE: usize = 32 + 8 + 8 + 2 + 8 + 1;

    pub fn standing(&self) -> Standing {
        Standing {
            player: self.player.to_bytes(),
            score: self.score,
            impact_ppm: self.impact_ppm,
            distinct_opponents: self.distinct_opponents,
            tie_breaker: self.tie_breaker,
        }
    }
}

#[account]
#[derive(Debug)]
pub struct Leaderboard {
    pub epoch: u64,
    pub entries: Vec<WinnerEntry>,
    pub bump: u8,
}

impl Leaderboard {
    pub const SPACE: usize = 8 + 8 + 4 + MAX_WINNERS * WinnerEntry::SIZE + 1;

    pub fn insert(&mut self, candidate: WinnerEntry) {
        if self.entries.len() < MAX_WINNERS {
            self.entries.push(candidate);
            self.bubble_up(self.entries.len() - 1);
            return;
        }

        if is_better(&candidate.standing(), &self.entries[0].standing()) {
            self.entries[0] = candidate;
            self.sift_down(0);
        }
    }

    pub fn finalize(&mut self, winner_count: usize) {
        self.entries
            .sort_unstable_by(|left, right| compare_standings(&left.standing(), &right.standing()));
        self.entries.truncate(winner_count);
    }

    fn bubble_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if compare_standings(
                &self.entries[index].standing(),
                &self.entries[parent].standing(),
            ) != core::cmp::Ordering::Greater
            {
                break;
            }
            self.entries.swap(index, parent);
            index = parent;
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        loop {
            let left = index * 2 + 1;
            let right = left + 1;
            let mut worst = index;
            if left < self.entries.len()
                && compare_standings(
                    &self.entries[left].standing(),
                    &self.entries[worst].standing(),
                ) == core::cmp::Ordering::Greater
            {
                worst = left;
            }
            if right < self.entries.len()
                && compare_standings(
                    &self.entries[right].standing(),
                    &self.entries[worst].standing(),
                ) == core::cmp::Ordering::Greater
            {
                worst = right;
            }
            if worst == index {
                break;
            }
            self.entries.swap(index, worst);
            index = worst;
        }
    }
}

const _: () = assert!(Leaderboard::SPACE < 10 * 1024);

#[cfg(test)]
mod tests {
    use super::*;

    fn serialized_len<T: AnchorSerialize>(value: &T) -> usize {
        let mut serialized = Vec::new();
        value.serialize(&mut serialized).unwrap();
        serialized.len()
    }

    #[test]
    fn fixed_account_spaces_are_exact() {
        let protocol = Protocol {
            version: Protocol::VERSION,
            current_epoch: 1,
            glory_mint: Pubkey::default(),
            glory_committed: 0,
            bump: 1,
        };
        let epoch = Epoch {
            number: 1,
            phase: Phase::Registration,
            registration_started_at: 0,
            registration_ends_at: 1,
            reveal_ends_at: 2,
            active_starts_at: 3,
            active_ends_at: 4,
            entropy: [0; 32],
            seed: [0; 32],
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
            bump: 1,
        };
        let player = PlayerEpoch {
            epoch: 1,
            owner: Pubkey::default(),
            commitment: [0; 32],
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
            bump: 1,
        };
        let lane = BalanceLane {
            epoch: 1,
            owner: Pubkey::default(),
            index: 0,
            balance: 0,
            guard: 0,
            locked_amount: 0,
            locked_until: 0,
            redirect_armed: false,
            redirect_ready_at: 0,
            redirected_volume: 0,
            cumulative_weighted: 0,
            last_checkpoint_at: 0,
            bump: 1,
        };
        let rivalry = Rivalry {
            epoch: 1,
            actor: Pubkey::default(),
            target: Pubkey::default(),
            rent_payer: Pubkey::default(),
            impact_credited_ppm: 0,
            action_count: 0,
            bump: 1,
        };
        let keeper = KeeperCredit {
            epoch: 1,
            keeper: Pubkey::default(),
            players_settled: 0,
            claimed: false,
            bump: 1,
        };

        assert_eq!(serialized_len(&protocol), Protocol::DATA_LEN);
        assert_eq!(serialized_len(&epoch), Epoch::DATA_LEN);
        assert_eq!(serialized_len(&player), PlayerEpoch::DATA_LEN);
        assert_eq!(serialized_len(&lane), BalanceLane::DATA_LEN);
        assert_eq!(serialized_len(&rivalry), Rivalry::DATA_LEN);
        assert_eq!(serialized_len(&keeper), KeeperCredit::DATA_LEN);
    }

    #[test]
    fn leaderboard_heap_retains_exact_best_set() {
        let mut leaderboard = Leaderboard {
            epoch: 1,
            entries: Vec::new(),
            bump: 1,
        };
        for score in (0..(MAX_WINNERS * 2) as u64).rev() {
            let mut player = [0u8; 32];
            player[..8].copy_from_slice(&score.to_le_bytes());
            leaderboard.insert(WinnerEntry {
                player: Pubkey::new_from_array(player),
                score,
                impact_ppm: 0,
                distinct_opponents: 0,
                tie_breaker: score,
                claimed: false,
            });
        }
        leaderboard.finalize(MAX_WINNERS);
        assert_eq!(leaderboard.entries.len(), MAX_WINNERS);
        assert_eq!(
            leaderboard.entries.first().map(|entry| entry.score),
            Some(0)
        );
        assert_eq!(
            leaderboard.entries.last().map(|entry| entry.score),
            Some(MAX_WINNERS as u64 - 1)
        );
    }

    #[test]
    fn maximum_leaderboard_serialization_fits_declared_space() {
        let entries = (0..MAX_WINNERS)
            .map(|index| WinnerEntry {
                player: Pubkey::new_from_array([index as u8; 32]),
                score: index as u64,
                impact_ppm: index as u64,
                distinct_opponents: index as u16,
                tie_breaker: index as u64,
                claimed: false,
            })
            .collect();
        let leaderboard = Leaderboard {
            epoch: 1,
            entries,
            bump: 1,
        };
        let mut serialized = Vec::new();
        leaderboard.serialize(&mut serialized).unwrap();
        assert!(8 + serialized.len() <= Leaderboard::SPACE);
    }
}
