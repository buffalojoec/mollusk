//! Module for working with Solana sysvars.

use {
    solana_program_runtime::sysvar_cache::SysvarCache,
    solana_sdk::{
        clock::Clock, epoch_schedule::EpochSchedule, rent::Rent, slot_hashes::SlotHashes,
        sysvar::SysvarId,
    },
};

/// Create a default sysvar cache instance.
pub fn default_sysvar_cache() -> SysvarCache {
    let mut cache = SysvarCache::default();
    cache.fill_missing_entries(|pubkey, set_sysvar| {
        if pubkey.eq(&Clock::id()) {
            set_sysvar(&bincode::serialize(&Clock::default()).unwrap());
        }
        if pubkey.eq(&EpochSchedule::id()) {
            set_sysvar(&bincode::serialize(&EpochSchedule::default()).unwrap());
        }
        if pubkey.eq(&Rent::id()) {
            set_sysvar(&bincode::serialize(&Rent::default()).unwrap());
        }
        if pubkey.eq(&SlotHashes::id()) {
            set_sysvar(&bincode::serialize(&SlotHashes::default()).unwrap());
        }
    });
    cache
}
