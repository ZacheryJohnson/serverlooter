use bevy::app::{App, Plugin};
use crate::inventory::systems::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_inventory_item_added)
            .add_observer(on_inventory_item_removed);
    }
}