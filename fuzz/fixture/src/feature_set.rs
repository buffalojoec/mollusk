//! Runtime feature set.

use {
    super::proto::FeatureSet as ProtoFeatureSet,
    solana_sdk::{feature_set::FeatureSet, keccak::Hasher},
};

impl From<ProtoFeatureSet> for FeatureSet {
    fn from(value: ProtoFeatureSet) -> Self {
        let mut feature_set = Self::default();
        let inactive = std::mem::take(&mut feature_set.inactive);

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
            .map(|feature_id| {
                let discriminator = &feature_id.to_bytes()[0..8];
                u64::from_le_bytes(discriminator.try_into().unwrap())
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
