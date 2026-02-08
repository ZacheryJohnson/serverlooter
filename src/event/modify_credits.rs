use bevy::prelude::Event;
use crate::script::ScriptId;

pub enum ModificationSource {
    Script(ScriptId),
}

#[derive(Event)]
pub struct ModifyCreditsEvent {
    pub credits: i64,
    pub source: ModificationSource,
}