//! Solana runtime sysvars, as represented in the Solana SDK.

use {
    super::{error::FixtureError, proto},
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

fn try_to_hash(bytes: &[u8]) -> Result<Hash, FixtureError> {
    <[u8; 32]>::try_from(bytes)
        .map_err(|_| FixtureError::InvalidHashBytes)
        .map(Hash::new_from_array)
}

fn try_read_u128(bytes: &[u8]) -> Result<u128, FixtureError> {
    bytes
        .try_into()
        .map_err(|_| FixtureError::InvalidU128Bytes)
        .map(u128::from_le_bytes)
}

/// A fixture containing the Solana runtime sysvars.
#[derive(Debug, Default)]
pub struct FixtureSysvarContext {
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

impl From<proto::Clock> for Clock {
    fn from(input: proto::Clock) -> Self {
        Self {
            slot: input.slot,
            epoch_start_timestamp: input.epoch_start_timestamp,
            epoch: input.epoch,
            leader_schedule_epoch: input.leader_schedule_epoch,
            unix_timestamp: input.unix_timestamp,
        }
    }
}

impl TryFrom<proto::EpochRewards> for EpochRewards {
    type Error = FixtureError;

    fn try_from(input: proto::EpochRewards) -> Result<Self, Self::Error> {
        let parent_blockhash = try_to_hash(&input.parent_blockhash)?;
        let total_points = try_read_u128(&input.total_points)?;
        Ok(Self {
            distribution_starting_block_height: input.distribution_starting_block_height,
            num_partitions: input.num_partitions,
            parent_blockhash,
            total_points,
            total_rewards: input.total_rewards,
            distributed_rewards: input.distributed_rewards,
            active: input.active,
        })
    }
}

impl From<proto::EpochSchedule> for EpochSchedule {
    fn from(input: proto::EpochSchedule) -> Self {
        Self {
            slots_per_epoch: input.slots_per_epoch,
            leader_schedule_slot_offset: input.leader_schedule_slot_offset,
            warmup: input.warmup,
            first_normal_epoch: input.first_normal_epoch,
            first_normal_slot: input.first_normal_slot,
        }
    }
}

impl TryFrom<proto::Rent> for Rent {
    type Error = FixtureError;

    fn try_from(input: proto::Rent) -> Result<Self, Self::Error> {
        let burn_percent =
            u8::try_from(input.burn_percent).map_err(|_| FixtureError::IntegerOutOfRange)?;
        Ok(Self {
            lamports_per_byte_year: input.lamports_per_byte_year,
            exemption_threshold: input.exemption_threshold,
            burn_percent,
        })
    }
}

impl TryFrom<proto::SlotHashEntry> for SlotHash {
    type Error = FixtureError;

    fn try_from(input: proto::SlotHashEntry) -> Result<Self, Self::Error> {
        let hash = Hash::new_from_array(
            input
                .hash
                .try_into()
                .map_err(|_| FixtureError::InvalidHashBytes)?,
        );
        Ok((input.slot, hash))
    }
}

impl TryFrom<proto::SlotHashes> for SlotHashes {
    type Error = FixtureError;

    fn try_from(input: proto::SlotHashes) -> Result<Self, Self::Error> {
        let slot_hashes: Vec<SlotHash> = input
            .slot_hashes
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(&slot_hashes))
    }
}

impl From<proto::StakeHistoryEntry> for (u64, StakeHistoryEntry) {
    fn from(input: proto::StakeHistoryEntry) -> (u64, StakeHistoryEntry) {
        (
            input.epoch,
            StakeHistoryEntry {
                effective: input.effective,
                activating: input.activating,
                deactivating: input.deactivating,
            },
        )
    }
}

impl From<proto::StakeHistory> for StakeHistory {
    fn from(input: proto::StakeHistory) -> Self {
        let mut stake_history = StakeHistory::default();
        for (epoch, entry) in input.stake_history.into_iter().map(Into::into) {
            stake_history.add(epoch, entry);
        }
        stake_history
    }
}

impl TryFrom<proto::SysvarContext> for FixtureSysvarContext {
    type Error = FixtureError;

    fn try_from(input: proto::SysvarContext) -> Result<Self, Self::Error> {
        Ok(Self {
            clock: input.clock.map(Into::into).unwrap_or_default(),
            epoch_rewards: input
                .epoch_rewards
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or_default(),
            epoch_schedule: input.epoch_schedule.map(Into::into).unwrap_or_default(),
            rent: input
                .rent
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or_default(),
            slot_hashes: input
                .slot_hashes
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or_default(),
            stake_history: input.stake_history.map(Into::into).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_proto_clock() {
        let input = proto::Clock {
            slot: 42,
            epoch_start_timestamp: 1_000_000,
            epoch: 1,
            leader_schedule_epoch: 1,
            unix_timestamp: 1_000_000,
        };
        let clock = Clock::from(input);
        assert_eq!(clock.slot, 42);
        assert_eq!(clock.epoch_start_timestamp, 1_000_000);
        assert_eq!(clock.epoch, 1);
        assert_eq!(clock.leader_schedule_epoch, 1);
        assert_eq!(clock.unix_timestamp, 1_000_000);
    }

    #[test]
    fn test_from_proto_epoch_rewards() {
        let input = proto::EpochRewards {
            distribution_starting_block_height: 42,
            num_partitions: 42,
            parent_blockhash: vec![0; 32],
            total_points: 16u128.to_le_bytes().to_vec(),
            total_rewards: 42,
            distributed_rewards: 42,
            active: true,
        };
        let epoch_rewards = EpochRewards::try_from(input).unwrap();
        assert_eq!(epoch_rewards.distribution_starting_block_height, 42);
        assert_eq!(epoch_rewards.num_partitions, 42);
        assert_eq!(epoch_rewards.parent_blockhash, Hash::default());
        assert_eq!(epoch_rewards.total_points, 16);
        assert_eq!(epoch_rewards.total_rewards, 42);
        assert_eq!(epoch_rewards.distributed_rewards, 42);
    }

    #[test]
    fn test_from_proto_epoch_schedule() {
        let input = proto::EpochSchedule {
            slots_per_epoch: 42,
            leader_schedule_slot_offset: 42,
            warmup: false,
            first_normal_epoch: 42,
            first_normal_slot: 42,
        };
        let epoch_schedule = EpochSchedule::from(input);
        assert_eq!(epoch_schedule.slots_per_epoch, 42);
        assert_eq!(epoch_schedule.leader_schedule_slot_offset, 42);
        assert!(!epoch_schedule.warmup);
        assert_eq!(epoch_schedule.first_normal_epoch, 42);
        assert_eq!(epoch_schedule.first_normal_slot, 42);
    }

    #[test]
    fn test_try_from_proto_rent() {
        let input = proto::Rent {
            lamports_per_byte_year: 42,
            exemption_threshold: 42.0,
            burn_percent: 42,
        };
        let rent = Rent::try_from(input).unwrap();
        assert_eq!(rent.lamports_per_byte_year, 42);
        assert_eq!(rent.exemption_threshold, 42.0);
        assert_eq!(rent.burn_percent, 42);

        // Fail integer out of range
        let input = proto::Rent {
            lamports_per_byte_year: 42,
            exemption_threshold: 42.0,
            burn_percent: 256,
        };
        assert_eq!(
            Rent::try_from(input).unwrap_err(),
            FixtureError::IntegerOutOfRange
        );
    }

    #[test]
    fn test_try_from_proto_slot_hash_entry() {
        let input = proto::SlotHashEntry {
            slot: 42,
            hash: vec![0; 32],
        };
        let slot_hash = SlotHash::try_from(input).unwrap();
        assert_eq!(slot_hash.0, 42);
        assert_eq!(slot_hash.1, Hash::default());

        // Fail invalid hash bytes
        let input = proto::SlotHashEntry {
            slot: 42,
            hash: vec![0; 31],
        };
        assert_eq!(
            SlotHash::try_from(input).unwrap_err(),
            FixtureError::InvalidHashBytes
        );
    }

    #[test]
    fn test_try_from_proto_slot_hashes() {
        let input = proto::SlotHashes {
            slot_hashes: vec![proto::SlotHashEntry {
                slot: 42,
                hash: vec![0; 32],
            }],
        };
        let slot_hashes = SlotHashes::try_from(input).unwrap();
        assert_eq!(slot_hashes.len(), 1);
        assert_eq!(slot_hashes.get(&42), Some(&Hash::default()));
    }

    #[test]
    fn test_from_proto_stake_history_entry() {
        let input = proto::StakeHistoryEntry {
            epoch: 42,
            effective: 42,
            activating: 42,
            deactivating: 42,
        };
        let (epoch, entry) = <(u64, StakeHistoryEntry)>::from(input);
        assert_eq!(epoch, 42);
        assert_eq!(entry.effective, 42);
        assert_eq!(entry.activating, 42);
        assert_eq!(entry.deactivating, 42);
    }

    #[test]
    fn test_from_proto_stake_history() {
        let input = proto::StakeHistory {
            stake_history: vec![proto::StakeHistoryEntry {
                epoch: 42,
                effective: 42,
                activating: 42,
                deactivating: 42,
            }],
        };
        let stake_history = StakeHistory::from(input);
        assert_eq!(stake_history.get(42).unwrap().effective, 42);
    }

    #[test]
    fn test_try_from_proto_sysvar_context() {
        let input = proto::SysvarContext {
            clock: Some(proto::Clock {
                slot: 42,
                epoch_start_timestamp: 1_000_000,
                epoch: 1,
                leader_schedule_epoch: 1,
                unix_timestamp: 1_000_000,
            }),
            epoch_rewards: Some(proto::EpochRewards {
                distribution_starting_block_height: 42,
                num_partitions: 42,
                parent_blockhash: vec![0; 32],
                total_points: vec![1; 16],
                total_rewards: 42,
                distributed_rewards: 42,
                active: true,
            }),
            epoch_schedule: Some(proto::EpochSchedule {
                slots_per_epoch: 42,
                leader_schedule_slot_offset: 42,
                warmup: false,
                first_normal_epoch: 42,
                first_normal_slot: 42,
            }),
            rent: Some(proto::Rent {
                lamports_per_byte_year: 42,
                exemption_threshold: 42.0,
                burn_percent: 42,
            }),
            slot_hashes: Some(proto::SlotHashes {
                slot_hashes: vec![proto::SlotHashEntry {
                    slot: 42,
                    hash: vec![0; 32],
                }],
            }),
            stake_history: Some(proto::StakeHistory {
                stake_history: vec![proto::StakeHistoryEntry {
                    epoch: 42,
                    effective: 42,
                    activating: 42,
                    deactivating: 42,
                }],
            }),
        };
        let sysvar_context = FixtureSysvarContext::try_from(input).unwrap();
        assert_eq!(sysvar_context.clock.slot, 42);
        assert_eq!(sysvar_context.epoch_rewards.total_rewards, 42);
        assert_eq!(sysvar_context.epoch_schedule.slots_per_epoch, 42);
        assert_eq!(sysvar_context.rent.lamports_per_byte_year, 42);
        assert_eq!(sysvar_context.slot_hashes.get(&42), Some(&Hash::default()));
        assert_eq!(sysvar_context.stake_history.get(42).unwrap().effective, 42);
    }
}
