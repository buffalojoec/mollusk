//! Module for working with Solana sysvars.

use {
    solana_program_runtime::sysvar_cache::SysvarCache,
    solana_sdk::{
        account::{AccountSharedData, ReadableAccount},
        clock::{Clock, Slot},
        epoch_rewards::EpochRewards,
        epoch_schedule::EpochSchedule,
        hash::Hash,
        pubkey::Pubkey,
        rent::Rent,
        slot_hashes::{self, SlotHashes},
        stake_history::{StakeHistory, StakeHistoryEntry},
        sysvar::{self, last_restart_slot::LastRestartSlot, Sysvar, SysvarId},
    },
};

// Agave's sysvar cache is difficult to work with, so Mollusk offers a wrapper
// around it for modifying its contents.
/// Mollusk sysvars.
pub struct Sysvars {
    pub clock: Clock,
    pub epoch_rewards: EpochRewards,
    pub epoch_schedule: EpochSchedule,
    pub last_restart_slot: LastRestartSlot,
    pub rent: Rent,
    pub slot_hashes: SlotHashes,
    pub stake_history: StakeHistory,
}

impl Default for Sysvars {
    fn default() -> Self {
        let clock = Clock::default();
        let epoch_rewards = EpochRewards::default();
        let epoch_schedule = EpochSchedule::default();
        let last_restart_slot = LastRestartSlot::default();
        let rent = Rent::default();

        let slot_hashes = {
            let mut default_slot_hashes = vec![(0, Hash::default()); slot_hashes::MAX_ENTRIES];
            default_slot_hashes[0] = (clock.slot, Hash::default());
            SlotHashes::new(&default_slot_hashes)
        };

        let mut stake_history = StakeHistory::default();
        stake_history.add(clock.slot, StakeHistoryEntry::default());

        Self {
            clock,
            epoch_rewards,
            epoch_schedule,
            last_restart_slot,
            rent,
            slot_hashes,
            stake_history,
        }
    }
}

impl Sysvars {
    /// Create a `Sysvars` instance from a list of accounts. Any missing
    /// sysvars will be filled with default values.
    pub fn fill_from_accounts(accounts: &[(Pubkey, AccountSharedData)]) -> Self {
        let mut me = Self::default();
        for (key, account) in accounts {
            if key == &Clock::id() {
                me.clock = bincode::deserialize(account.data()).unwrap();
            }
            if key == &EpochRewards::id() {
                me.epoch_rewards = bincode::deserialize(account.data()).unwrap();
            }
            if key == &EpochSchedule::id() {
                me.epoch_schedule = bincode::deserialize(account.data()).unwrap();
            }
            if key == &LastRestartSlot::id() {
                me.last_restart_slot = bincode::deserialize(account.data()).unwrap();
            }
            if key == &Rent::id() {
                me.rent = bincode::deserialize(account.data()).unwrap();
            }
            if key == &SlotHashes::id() {
                me.slot_hashes = bincode::deserialize(account.data()).unwrap();
            }
            if key == &StakeHistory::id() {
                me.stake_history = bincode::deserialize(account.data()).unwrap();
            }
        }
        me
    }

    fn sysvar_account<T: SysvarId + Sysvar>(&self, sysvar: &T) -> (Pubkey, AccountSharedData) {
        let data = bincode::serialize::<T>(sysvar).unwrap();
        let space = data.len();
        let lamports = self.rent.minimum_balance(space);
        let mut account = AccountSharedData::new(lamports, space, &sysvar::id());
        account.set_data_from_slice(&data);
        (T::id(), account)
    }

    /// Get the key and account for the clock sysvar.
    pub fn keyed_account_for_clock_sysvar(&self) -> (Pubkey, AccountSharedData) {
        self.sysvar_account(&self.clock)
    }

    /// Get the key and account for the epoch rewards sysvar.
    pub fn keyed_account_for_epoch_rewards_sysvar(&self) -> (Pubkey, AccountSharedData) {
        self.sysvar_account(&self.epoch_rewards)
    }

    /// Get the key and account for the epoch schedule sysvar.
    pub fn keyed_account_for_epoch_schedule_sysvar(&self) -> (Pubkey, AccountSharedData) {
        self.sysvar_account(&self.epoch_schedule)
    }

    /// Get the key and account for the last restart slot sysvar.
    pub fn keyed_account_for_last_restart_slot_sysvar(&self) -> (Pubkey, AccountSharedData) {
        self.sysvar_account(&self.last_restart_slot)
    }

    /// Get the key and account for the rent sysvar.
    pub fn keyed_account_for_rent_sysvar(&self) -> (Pubkey, AccountSharedData) {
        self.sysvar_account(&self.rent)
    }

    /// Get the key and account for the slot hashes sysvar.
    pub fn keyed_account_for_slot_hashes_sysvar(&self) -> (Pubkey, AccountSharedData) {
        self.sysvar_account(&self.slot_hashes)
    }

    /// Get the key and account for the stake history sysvar.
    pub fn keyed_account_for_stake_history_sysvar(&self) -> (Pubkey, AccountSharedData) {
        self.sysvar_account(&self.stake_history)
    }

    /// Warp the test environment to a slot by updating sysvars.
    pub fn warp_to_slot(&mut self, slot: Slot) {
        // First update `Clock`.
        let epoch = self.epoch_schedule.get_epoch(slot);
        let leader_schedule_epoch = self.epoch_schedule.get_leader_schedule_epoch(slot);
        self.clock = Clock {
            slot,
            epoch,
            leader_schedule_epoch,
            ..Default::default()
        };

        // Then update `SlotHashes`.
        let i = if let Some(most_recent_slot_hash) = self.slot_hashes.first() {
            most_recent_slot_hash.0
        } else {
            // By default, this zero is never used, but a user can overwrite
            // `SlotHashes`.
            0
        };
        // Don't include the target slot, since it will become the "current"
        // slot.
        for slot in i..slot {
            self.slot_hashes.add(slot, Hash::default());
        }
    }
}

impl From<&Sysvars> for SysvarCache {
    fn from(mollusk_cache: &Sysvars) -> Self {
        let mut sysvar_cache = SysvarCache::default();
        sysvar_cache.fill_missing_entries(|pubkey, set_sysvar| {
            if pubkey.eq(&Clock::id()) {
                set_sysvar(&bincode::serialize(&mollusk_cache.clock).unwrap());
            }
            if pubkey.eq(&EpochRewards::id()) {
                set_sysvar(&bincode::serialize(&mollusk_cache.epoch_rewards).unwrap());
            }
            if pubkey.eq(&EpochSchedule::id()) {
                set_sysvar(&bincode::serialize(&mollusk_cache.epoch_schedule).unwrap());
            }
            if pubkey.eq(&LastRestartSlot::id()) {
                set_sysvar(&bincode::serialize(&mollusk_cache.last_restart_slot).unwrap());
            }
            if pubkey.eq(&Rent::id()) {
                set_sysvar(&bincode::serialize(&mollusk_cache.rent).unwrap());
            }
            if pubkey.eq(&SlotHashes::id()) {
                set_sysvar(&bincode::serialize(&mollusk_cache.slot_hashes).unwrap());
            }
            if pubkey.eq(&StakeHistory::id()) {
                set_sysvar(&bincode::serialize(&mollusk_cache.stake_history).unwrap());
            }
        });
        sysvar_cache
    }
}

#[cfg(test)]
mod tests {
    use {super::*, solana_sdk::stake_history::StakeHistoryEntry, std::ops::Deref};

    #[test]
    fn test_warp_to_slot() {
        let mut sysvars = Sysvars::default();

        let slot = 0;
        assert_eq!(sysvars.clock.slot, slot);
        assert_eq!(sysvars.clock.epoch, sysvars.epoch_schedule.get_epoch(slot));
        assert_eq!(
            sysvars.slot_hashes.as_slice(),
            &[(slot, Hash::default()); slot_hashes::MAX_ENTRIES]
        );
        assert_eq!(sysvars.slot_hashes.len(), slot_hashes::MAX_ENTRIES);

        let mut warp_and_check = |slot: Slot| {
            sysvars.warp_to_slot(slot);
            assert_eq!(sysvars.clock.slot, slot);
            assert_eq!(sysvars.clock.epoch, sysvars.epoch_schedule.get_epoch(slot));
            assert_eq!(
                sysvars.slot_hashes.first(),
                Some(&(slot - 1, Hash::default())),
            );
            assert_eq!(sysvars.slot_hashes.len(), slot_hashes::MAX_ENTRIES);
        };

        warp_and_check(200);
        warp_and_check(4_000);
        warp_and_check(800_000);
    }

    #[test]
    fn test_to_sysvar_cache() {
        let clock = Clock {
            slot: 1,
            epoch: 2,
            leader_schedule_epoch: 3,
            ..Default::default()
        };
        let epoch_rewards = EpochRewards {
            total_rewards: 4,
            ..Default::default()
        };
        let epoch_schedule = EpochSchedule {
            slots_per_epoch: 5,
            ..Default::default()
        };
        let last_restart_slot = LastRestartSlot {
            last_restart_slot: 6,
        };
        let rent = Rent {
            lamports_per_byte_year: 7,
            ..Default::default()
        };
        let slot_hashes = SlotHashes::new(&[(8, Hash::default())]);
        let stake_history = {
            let mut stake_history = StakeHistory::default();
            stake_history.add(9, StakeHistoryEntry::default());
            stake_history
        };

        let sysvars = Sysvars {
            clock,
            epoch_rewards,
            epoch_schedule,
            last_restart_slot,
            rent,
            slot_hashes,
            stake_history,
        };

        let sysvar_cache: SysvarCache = (&sysvars).into();
        assert_eq!(sysvar_cache.get_clock().unwrap().deref(), &sysvars.clock);
        assert_eq!(
            sysvar_cache.get_epoch_rewards().unwrap().deref(),
            &sysvars.epoch_rewards
        );
        assert_eq!(
            sysvar_cache.get_epoch_schedule().unwrap().deref(),
            &sysvars.epoch_schedule
        );
        assert_eq!(
            sysvar_cache.get_last_restart_slot().unwrap().deref(),
            &sysvars.last_restart_slot
        );
        assert_eq!(sysvar_cache.get_rent().unwrap().deref(), &sysvars.rent);
        assert_eq!(
            sysvar_cache.get_slot_hashes().unwrap().deref(),
            &sysvars.slot_hashes
        );
        assert_eq!(
            sysvar_cache.get_stake_history().unwrap().deref(),
            &sysvars.stake_history
        );
    }
}
