use bevy::prelude::Event;
use crate::inventory::InventoryItem;

#[derive(Event)]
pub struct InventoryItemAdded {
    pub item: InventoryItem,
}