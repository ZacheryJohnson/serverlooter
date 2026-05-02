use bevy::prelude::Event;
use crate::inventory::InventoryItem;

#[derive(Event)]
pub struct InventoryItemRemoved {
    pub item: InventoryItem,
}