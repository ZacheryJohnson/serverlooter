use bevy::prelude::Event;
use crate::PlayerUnlock;

#[derive(Event)]
pub struct RequestPurchaseUnlockEvent {
    pub unlock: PlayerUnlock,
    pub credit_cost: u128,
}