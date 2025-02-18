//! Runtime feature set.

use {
    super::proto::FeatureSet as ProtoFeatureSet,
    solana_sdk::{feature_set::FeatureSet, keccak::Hasher, pubkey::Pubkey},
};

// Omit "test features" (they have the same u64 ID).
pub static OMITTED_FEATURES: &[Pubkey] = &[];

impl From<ProtoFeatureSet> for FeatureSet {
    fn from(value: ProtoFeatureSet) -> Self {
        let mut feature_set = Self::default();
        let mut inactive = std::mem::take(&mut feature_set.inactive);
        OMITTED_FEATURES.iter().for_each(|f| {
            inactive.remove(f);
        });

        value.features.iter().for_each(|int_id| {
            let discriminator = int_id.to_le_bytes();
            let feature_id = inactive
                .iter()
                .find(|feature_id| feature_id.to_bytes()[0..8].eq(&discriminator));
            if let Some(feature_id) = feature_id {
                feature_set.activate(feature_id, 0);
            }
        });

        feature_set
    }
}

impl From<FeatureSet> for ProtoFeatureSet {
    fn from(value: FeatureSet) -> Self {
        let features = value
            .active
            .keys()
            .filter_map(|feature_id| {
                if OMITTED_FEATURES.contains(feature_id) {
                    return None;
                }
                let discriminator = &feature_id.to_bytes()[0..8];
                let int_id = u64::from_le_bytes(discriminator.try_into().unwrap());
                Some(int_id)
            })
            .collect();

        Self { features }
    }
}

pub(crate) fn hash_proto_feature_set(hasher: &mut Hasher, feature_set: &ProtoFeatureSet) {
    let mut features = feature_set.features.clone();
    features.sort();
    for f in &features {
        hasher.hash(&f.to_le_bytes());
    }
}
