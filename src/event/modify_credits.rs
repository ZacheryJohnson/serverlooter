use bevy::prelude::Event;
use crate::script::ScriptId;

pub enum ModificationSource {
    Script(#[allow(dead_code)] ScriptId),
}

#[derive(Event)]
pub struct ModifyCreditsEvent {
    pub credits: i64,
    #[allow(dead_code)] pub source: ModificationSource,
}