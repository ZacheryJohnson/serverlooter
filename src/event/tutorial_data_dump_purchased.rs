use bevy::prelude::Event;

#[derive(Event)]
pub struct TutorialDataDumpPurchased {
    pub credit_cost: u32,
}
