//! Runtime sysvars.

use {
    super::proto::{
        Clock as ProtoClock, EpochRewards as ProtoEpochRewards,
        EpochSchedule as ProtoEpochSchedule, Rent as ProtoRent,
        SlotHashEntry as ProtoSlotHashEntry, SlotHashes as ProtoSlotHashes,
        StakeHistory as ProtoStakeHistory, StakeHistoryEntry as ProtoStakeHistoryEntry,
        SysvarContext as ProtoSysvars,
    },
    solana_sdk::{
        clock::Clock,
        epoch_rewards::EpochRewards,
        epoch_schedule::EpochSchedule,
        hash::Hash,
        rent::Rent,
        slot_hashes::{SlotHash, SlotHashes},
        stake_history::{StakeHistory, StakeHistoryEntry},
    },
};

/// A fixture of runtime sysvars.
#[derive(Debug, Default, PartialEq)]
pub struct Sysvars {
    /// `Clock` sysvar.
    pub clock: Clock,
    /// `EpochRewards` sysvar.
    pub epoch_rewards: EpochRewards,
    /// `EpochSchedule` sysvar.
    pub epoch_schedule: EpochSchedule,
    /// `Rent` sysvar.
    pub rent: Rent,
    /// `SlotHashes` sysvar.
    pub slot_hashes: SlotHashes,
    /// `StakeHistory` sysvar.
    pub stake_history: StakeHistory,
}

impl Clone for Sysvars {
    fn clone(&self) -> Self {
        Self {
            clock: self.clock.clone(),
            epoch_rewards: self.epoch_rewards.clone(),
            epoch_schedule: self.epoch_schedule.clone(),
            rent: self.rent.clone(),
            slot_hashes: SlotHashes::new(self.slot_hashes.slot_hashes()),
            stake_history: self.stake_history.clone(),
        }
    }
}

// Clock sysvar.
impl From<ProtoClock> for Clock {
    fn from(value: ProtoClock) -> Self {
        Self {
            slot: value.slot,
            epoch_start_timestamp: value.epoch_start_timestamp,
            epoch: value.epoch,
            leader_schedule_epoch: value.leader_schedule_epoch,
            unix_timestamp: value.unix_timestamp,
        }
    }
}
impl From<Clock> for ProtoClock {
    fn from(value: Clock) -> Self {
        Self {
            slot: value.slot,
            epoch_start_timestamp: value.epoch_start_timestamp,
            epoch: value.epoch,
            leader_schedule_epoch: value.leader_schedule_epoch,
            unix_timestamp: value.unix_timestamp,
        }
    }
}

// Epoch rewards sysvar.
impl From<ProtoEpochRewards> for EpochRewards {
    fn from(value: ProtoEpochRewards) -> Self {
        let parent_blockhash_bytes: [u8; 32] = value
            .parent_blockhash
            .try_into()
            .expect("Invalid bytes for parent blockhash");
        let parent_blockhash = Hash::new_from_array(parent_blockhash_bytes);

        let total_points_bytes: [u8; 16] = value
            .total_points
            .try_into()
            .expect("Invalid bytes for total points");
        let total_points = u128::from_le_bytes(total_points_bytes);

        Self {
            distribution_starting_block_height: value.distribution_starting_block_height,
            num_partitions: value.num_partitions,
            parent_blockhash,
            total_points,
            total_rewards: value.total_rewards,
            distributed_rewards: value.distributed_rewards,
            active: value.active,
        }
    }
}
impl From<EpochRewards> for ProtoEpochRewards {
    fn from(value: EpochRewards) -> Self {
        Self {
            distribution_starting_block_height: value.distribution_starting_block_height,
            num_partitions: value.num_partitions,
            parent_blockhash: value.parent_blockhash.to_bytes().to_vec(),
            total_points: value.total_points.to_le_bytes().to_vec(),
            total_rewards: value.total_rewards,
            distributed_rewards: value.distributed_rewards,
            active: value.active,
        }
    }
}

// Epoch schedule sysvar.
impl From<ProtoEpochSchedule> for EpochSchedule {
    fn from(value: ProtoEpochSchedule) -> Self {
        Self {
            slots_per_epoch: value.slots_per_epoch,
            leader_schedule_slot_offset: value.leader_schedule_slot_offset,
            warmup: value.warmup,
            first_normal_epoch: value.first_normal_epoch,
            first_normal_slot: value.first_normal_slot,
        }
    }
}
impl From<EpochSchedule> for ProtoEpochSchedule {
    fn from(value: EpochSchedule) -> Self {
        Self {
            slots_per_epoch: value.slots_per_epoch,
            leader_schedule_slot_offset: value.leader_schedule_slot_offset,
            warmup: value.warmup,
            first_normal_epoch: value.first_normal_epoch,
            first_normal_slot: value.first_normal_slot,
        }
    }
}

// Rent sysvar.
impl From<ProtoRent> for Rent {
    fn from(value: ProtoRent) -> Self {
        let burn_percent =
            u8::try_from(value.burn_percent).expect("Invalid integer for burn percent");
        Self {
            lamports_per_byte_year: value.lamports_per_byte_year,
            exemption_threshold: value.exemption_threshold,
            burn_percent,
        }
    }
}
impl From<Rent> for ProtoRent {
    fn from(value: Rent) -> Self {
        Self {
            lamports_per_byte_year: value.lamports_per_byte_year,
            exemption_threshold: value.exemption_threshold,
            burn_percent: value.burn_percent.into(),
        }
    }
}

// Slot hashes sysvar.
impl From<ProtoSlotHashes> for SlotHashes {
    fn from(value: ProtoSlotHashes) -> Self {
        let slot_hashes: Vec<SlotHash> = value
            .slot_hashes
            .into_iter()
            .map(
                |ProtoSlotHashEntry {
                     slot,
                     hash: hash_bytes,
                 }| {
                    let hash_bytes: [u8; 32] =
                        hash_bytes.try_into().expect("Invalid bytes for slot hash");
                    let hash = Hash::new_from_array(hash_bytes);
                    (slot, hash)
                },
            )
            .collect();
        Self::new(&slot_hashes)
    }
}
impl From<SlotHashes> for ProtoSlotHashes {
    fn from(value: SlotHashes) -> Self {
        let slot_hashes = value
            .iter()
            .map(|(slot, hash)| ProtoSlotHashEntry {
                slot: *slot,
                hash: hash.to_bytes().to_vec(),
            })
            .collect();
        Self { slot_hashes }
    }
}

// Stake history sysvar.
impl From<ProtoStakeHistory> for StakeHistory {
    fn from(value: ProtoStakeHistory) -> Self {
        let mut stake_history = StakeHistory::default();
        for entry in value.stake_history.into_iter() {
            stake_history.add(
                entry.epoch,
                StakeHistoryEntry {
                    effective: entry.effective,
                    activating: entry.activating,
                    deactivating: entry.deactivating,
                },
            );
        }
        stake_history
    }
}
impl From<StakeHistory> for ProtoStakeHistory {
    fn from(value: StakeHistory) -> Self {
        let stake_history = value
            .iter()
            .map(|(epoch, entry)| ProtoStakeHistoryEntry {
                epoch: *epoch,
                effective: entry.effective,
                activating: entry.activating,
                deactivating: entry.deactivating,
            })
            .collect();
        Self { stake_history }
    }
}

// Sysvars.
impl From<ProtoSysvars> for Sysvars {
    fn from(value: ProtoSysvars) -> Self {
        Self {
            clock: value.clock.map(Into::into).unwrap_or_default(),
            epoch_rewards: value.epoch_rewards.map(Into::into).unwrap_or_default(),
            epoch_schedule: value.epoch_schedule.map(Into::into).unwrap_or_default(),
            rent: value.rent.map(Into::into).unwrap_or_default(),
            slot_hashes: value.slot_hashes.map(Into::into).unwrap_or_default(),
            stake_history: value.stake_history.map(Into::into).unwrap_or_default(),
        }
    }
}
impl From<Sysvars> for ProtoSysvars {
    fn from(value: Sysvars) -> Self {
        Self {
            clock: Some(value.clock.into()),
            epoch_rewards: Some(value.epoch_rewards.into()),
            epoch_schedule: Some(value.epoch_schedule.into()),
            rent: Some(value.rent.into()),
            slot_hashes: Some(value.slot_hashes.into()),
            stake_history: Some(value.stake_history.into()),
        }
    }
}
