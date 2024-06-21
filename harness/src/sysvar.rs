//! Module for working with Solana sysvars.

use {
    solana_program_runtime::sysvar_cache::SysvarCache,
    solana_sdk::{
        clock::Clock, epoch_schedule::EpochSchedule, rent::Rent, slot_hashes::SlotHashes,
    },
};

/// Create a default sysvar cache instance.
pub fn default_sysvar_cache() -> SysvarCache {
    let mut cache = SysvarCache::default();
    cache.set_clock(Clock::default());
    cache.set_epoch_schedule(EpochSchedule::default());
    cache.set_rent(Rent::default());
    cache.set_slot_hashes(SlotHashes::default());
    cache
}
