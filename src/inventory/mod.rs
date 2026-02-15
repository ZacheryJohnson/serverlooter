use std::sync::{Arc, Mutex};
use bevy::prelude::Event;
use uuid::Uuid;
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::{AlgorithmEffect, AlgorithmEffectTarget};
use crate::algorithm::id::AlgorithmId;
use crate::server::ServerStatType;

pub mod plugin;
pub mod systems;

pub enum InventoryItem {
    Algorithm(Arc<Mutex<Algorithm>>),
}

#[derive(Event)]
pub struct InventoryItemAdded {
    pub item: InventoryItem,
}

#[derive(Event)]
pub struct InventoryItemRemoved {
    pub item: InventoryItem,
}

pub struct Inventory {
    pub algorithms: Vec<Arc<Mutex<Algorithm>>>,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            algorithms: vec![
                Arc::new(Mutex::new(Algorithm {
                    id: AlgorithmId::Id(Uuid::new_v4()),
                    instruction_count: 1_000_000,
                    instruction_effects: vec![
                        (1_000_000, vec![
                            AlgorithmEffect::Siphon { potency: (5..10).into() },
                        ])
                    ],
                })),
                Arc::new(Mutex::new(Algorithm {
                    id: AlgorithmId::Id(Uuid::new_v4()),
                    instruction_count: 5_000_000,
                    instruction_effects: vec![
                        (5_000_000, vec![
                            AlgorithmEffect::Modify {
                                target: AlgorithmEffectTarget::TargetServer,
                                stat: ServerStatType::SiphonResist,
                                potency: (-5..-1).into(),
                            }
                        ])
                    ]
                })),
                Arc::new(Mutex::new(Algorithm {
                    id: AlgorithmId::Id(Uuid::new_v4()),
                    instruction_count: 3_000_000,
                    instruction_effects: vec![
                        (3_000_000, vec![
                            AlgorithmEffect::Exfil {
                                potency: (5..10).into(),
                            }
                        ])
                    ]
                })),
            ],
        }
    }
}