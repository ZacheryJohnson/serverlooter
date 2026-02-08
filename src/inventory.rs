use uuid::Uuid;
use std::sync::{Arc, Mutex};
use bevy::prelude::{Event, On, ResMut};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::{AlgorithmEffect, AlgorithmEffectTarget};
use crate::algorithm::id::AlgorithmId;
use crate::PlayerState;
use crate::server::ServerStatType;

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

pub fn on_inventory_item_added(
    evt: On<InventoryItemAdded>,
    mut player_state: ResMut<PlayerState>,
) {
    match &evt.item {
        InventoryItem::Algorithm(algorithm) => {
            player_state.inventory.algorithms.push(algorithm.to_owned());
        }
    }
}

pub fn on_inventory_item_removed(
    evt: On<InventoryItemRemoved>,
    mut player_state: ResMut<PlayerState>,
) {
    match &evt.item {
        InventoryItem::Algorithm(algorithm) => {
            player_state.inventory.algorithms.retain(|algo| {
                let self_id = { algo.lock().unwrap().id.clone() };
                let target_id = { algorithm.lock().unwrap().id.clone() };
                self_id != target_id
            });
        }
    }
}