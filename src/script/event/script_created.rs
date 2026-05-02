use bevy::prelude::Event;
use crate::script::Script;

#[derive(Event)]
pub struct ScriptCreatedEvent {
    pub script: Script
}