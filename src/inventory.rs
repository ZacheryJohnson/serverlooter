use bevy::prelude::{Event, On, ResMut};
use crate::PlayerState;
use crate::script::{Algorithm, AlgorithmEffect};

pub enum InventoryItem {
    Algorithm(Algorithm),
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
    pub algorithms: Vec<Algorithm>,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            algorithms: vec![
                Algorithm {
                    instruction_count: 1_000_000,
                    instruction_effects: vec![
                        (1_000_000, vec![
                            AlgorithmEffect::Extract { potency: (5..10).into() },
                        ])
                    ],
                }
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
            player_state.inventory.algorithms.retain(|algo| algo != algorithm);
        }
    }
}