//! A Solana runtime feature set, as represented in the Solana SDK.

use {super::proto, solana_sdk::feature_set::*};

#[derive(Debug)]
pub struct FixtureFeatureSet {
    pub features: Vec<u64>,
}

impl Default for FixtureFeatureSet {
    fn default() -> Self {
        Self::from(&FeatureSet::all_enabled())
    }
}

impl From<proto::FeatureSet> for FixtureFeatureSet {
    fn from(input: proto::FeatureSet) -> Self {
        Self {
            features: input.features,
        }
    }
}

impl From<FixtureFeatureSet> for FeatureSet {
    fn from(input: FixtureFeatureSet) -> Self {
        let mut feature_set = FeatureSet::default();
        for id in std::mem::take(&mut feature_set.inactive).iter() {
            let discriminator = u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap());
            if input.features.contains(&discriminator) {
                feature_set.activate(id, 0);
            }
        }
        feature_set
    }
}

impl From<&FeatureSet> for FixtureFeatureSet {
    fn from(input: &FeatureSet) -> Self {
        let features = input
            .active
            .keys()
            .map(|id| u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap()))
            .collect();
        Self { features }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, solana_sdk::pubkey::Pubkey};

    #[test]
    fn test_from_proto_feature_set() {
        let try_conversion = |feature_ids: &[Pubkey]| {
            let features = feature_ids
                .iter()
                .map(|id| u64::from_le_bytes(id.to_bytes()[..8].try_into().unwrap()))
                .collect::<Vec<_>>();
            let proto = proto::FeatureSet { features };
            let fixture = FixtureFeatureSet::from(proto);
            FeatureSet::from(fixture)
        };

        // Success
        let features = &[
            vote_state_add_vote_latency::id(),
            checked_arithmetic_in_fee_validation::id(),
            last_restart_slot_sysvar::id(),
            reduce_stake_warmup_cooldown::id(),
            enable_poseidon_syscall::id(),
            require_rent_exempt_split_destination::id(),
            better_error_codes_for_tx_lamport_check::id(),
        ];
        let feature_set = try_conversion(features);
        for feature in features {
            assert!(feature_set.is_active(feature));
        }

        // Not valid features (not in the list)
        let features = &[
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let feature_set = try_conversion(features);
        for feature in features {
            assert!(!feature_set.is_active(feature));
        }
    }
}
