//! Module for working with Solana sysvars.

use {
    solana_program_runtime::sysvar_cache::SysvarCache,
    solana_sdk::{
        clock::{Clock, Slot},
        epoch_rewards::EpochRewards,
        epoch_schedule::EpochSchedule,
        hash::Hash,
        rent::Rent,
        slot_hashes::SlotHashes,
        stake_history::StakeHistory,
        sysvar::{last_restart_slot::LastRestartSlot, SysvarId},
    },
};

pub(crate) fn warp_sysvar_cache_to_slot(sysvar_cache: &mut SysvarCache, slot: Slot) {
    // First update `Clock`.
    let epoch_schedule = sysvar_cache.get_epoch_schedule().unwrap_or_default();
    let epoch = epoch_schedule.get_epoch(slot);
    let leader_schedule_epoch = epoch_schedule.get_leader_schedule_epoch(slot);
    let new_clock = Clock {
        slot,
        epoch,
        leader_schedule_epoch,
        ..Default::default()
    };

    // Then update `SlotHashes`.
    let mut i = 0;
    if let Some(most_recent_slot_hash) = sysvar_cache.get_slot_hashes().unwrap_or_default().first()
    {
        i = most_recent_slot_hash.0;
    }
    let mut new_slot_hashes = vec![];
    for slot in i..slot + 1 {
        new_slot_hashes.push((slot, Hash::default()));
    }
    let new_slot_hashes = SlotHashes::new(&new_slot_hashes);

    sysvar_cache.fill_missing_entries(|pubkey, set_sysvar| {
        if pubkey.eq(&Clock::id()) {
            set_sysvar(&bincode::serialize(&new_clock).unwrap());
        }
        if pubkey.eq(&SlotHashes::id()) {
            set_sysvar(&bincode::serialize(&new_slot_hashes).unwrap());
        }
    });
}

/// Create a default sysvar cache instance.
pub fn default_sysvar_cache() -> SysvarCache {
    let mut cache = SysvarCache::default();
    cache.fill_missing_entries(|pubkey, set_sysvar| {
        if pubkey.eq(&Clock::id()) {
            set_sysvar(&bincode::serialize(&Clock::default()).unwrap());
        }
        if pubkey.eq(&EpochRewards::id()) {
            set_sysvar(&bincode::serialize(&EpochRewards::default()).unwrap());
        }
        if pubkey.eq(&EpochSchedule::id()) {
            set_sysvar(&bincode::serialize(&EpochSchedule::default()).unwrap());
        }
        if pubkey.eq(&LastRestartSlot::id()) {
            set_sysvar(&bincode::serialize(&LastRestartSlot::default()).unwrap());
        }
        if pubkey.eq(&Rent::id()) {
            set_sysvar(&bincode::serialize(&Rent::default()).unwrap());
        }
        if pubkey.eq(&SlotHashes::id()) {
            set_sysvar(&bincode::serialize(&SlotHashes::default()).unwrap());
        }
        if pubkey.eq(&StakeHistory::id()) {
            set_sysvar(&bincode::serialize(&StakeHistory::default()).unwrap());
        }
    });
    cache
}
