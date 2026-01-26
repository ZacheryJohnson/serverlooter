use std::sync::{Arc, Mutex};
use rand::distr::{Distribution, Uniform};
use rand::prelude::IndexedRandom;
use rand::Rng;
use uuid::Uuid;
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::{AlgorithmEffect, AlgorithmEffectTarget, AlgorithmEffectValue};
use crate::algorithm::id::AlgorithmId;
use crate::server::ServerStatType;

pub struct AlgorithmGenerator;

impl AlgorithmGenerator {
    pub fn generate() -> Arc<Mutex<Algorithm>> {
        let num_effects_distr = rand_distr::Normal::new(1.0f32, 0.7).unwrap();
        let num_effects = num_effects_distr
            .sample(&mut rand::rng())
            .floor()
            .max(1.0)
            as u8;

        let ph_val = 0;
        let effect_options = [
            AlgorithmEffect::Siphon { potency: AlgorithmEffectValue::Static(ph_val) },
            AlgorithmEffect::Siphon { potency: AlgorithmEffectValue::RangeInclusive(ph_val, ph_val) },
            AlgorithmEffect::Exfil { potency: AlgorithmEffectValue::Static(ph_val) },
            AlgorithmEffect::Exfil { potency: AlgorithmEffectValue::RangeInclusive(ph_val, ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::SelfServer, stat: ServerStatType::SiphonResist, potency: AlgorithmEffectValue::Static(ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::SelfServer, stat: ServerStatType::SiphonResist, potency: AlgorithmEffectValue::RangeInclusive(ph_val, ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::SelfServer, stat: ServerStatType::ExfilResist, potency: AlgorithmEffectValue::Static(ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::SelfServer, stat: ServerStatType::ExfilResist, potency: AlgorithmEffectValue::RangeInclusive(ph_val, ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::TargetServer, stat: ServerStatType::SiphonResist, potency: AlgorithmEffectValue::Static(ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::TargetServer, stat: ServerStatType::SiphonResist, potency: AlgorithmEffectValue::RangeInclusive(ph_val, ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::TargetServer, stat: ServerStatType::ExfilResist, potency: AlgorithmEffectValue::Static(ph_val) },
            AlgorithmEffect::Modify { target: AlgorithmEffectTarget::TargetServer, stat: ServerStatType::ExfilResist, potency: AlgorithmEffectValue::RangeInclusive(ph_val, ph_val) },
        ];

        let mut added_effects = vec![];
        let mut rng = rand::rng();
        for _ in 0..num_effects {
            let mut new_effect = effect_options.choose(&mut rng).unwrap().to_owned();
            match new_effect {
                AlgorithmEffect::Siphon { ref mut potency } => {
                    match potency {
                        AlgorithmEffectValue::Static(val) => {
                            *val = rng.random_range(1..10);
                        },
                        AlgorithmEffectValue::RangeInclusive(low, high) => {
                            *low = rng.random_range(1..=8);
                            *high = rng.random_range((*low + 1)..=10);
                        },
                    }
                }
                AlgorithmEffect::Exfil { ref mut potency } => {
                    match potency {
                        AlgorithmEffectValue::Static(val) => {
                            *val = rng.random_range(1..10);
                        }
                        AlgorithmEffectValue::RangeInclusive(low, high) => {
                            *low = rng.random_range(1..10);
                            *high = rng.random_range((*low + 1)..=10);
                        }
                    }
                }
                AlgorithmEffect::Modify { ref mut potency, .. } => {
                    match potency {
                        AlgorithmEffectValue::Static(val) => {
                            *val = rng.random_range(-10..10);
                        }
                        AlgorithmEffectValue::RangeInclusive(low, high) => {
                            *low = rng.random_range(-10..10);
                            *high = rng.random_range((*low + 1)..=10);
                        }
                    }
                }
                _ => todo!("generator: unsupported effect {:?}", new_effect),
            }

            added_effects.push(new_effect);
        }

        let instruction_count = rng.sample(Uniform::new(1_000_000, 3_000_000).unwrap()) * num_effects as u64;

        let instruction_effects = vec![(instruction_count, added_effects)];

        Arc::new(Mutex::new(Algorithm {
            id: AlgorithmId::Id(Uuid::new_v4()),
            instruction_count,
            instruction_effects,
        }))
    }
}
