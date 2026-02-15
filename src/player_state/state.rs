use std::sync::{Arc, Mutex};
use bevy::prelude::Resource;
use fluent_templates::LanguageIdentifier;
use crate::active_exploit::{ActiveExploit, ExploitTarget};
use crate::inventory::Inventory;
use crate::script::Script;
use crate::server::Server;
use crate::tutorial::progression::TutorialProgression;

pub enum PlayerUnlock {
    ExploitAutoReconnect,
}

pub struct PlayerUnlocks {
    pub exploit_auto_reconnect: bool,
}

#[derive(Resource)]
pub struct PlayerState {
    pub progression: TutorialProgression,
    pub language_identifier: LanguageIdentifier,
    pub credits: u128,
    pub inventory: Inventory,
    pub servers: Vec<Arc<Mutex<Server>>>,
    pub known_targets: Vec<Arc<Mutex<ExploitTarget>>>,
    pub active_exploits: Vec<Arc<Mutex<ActiveExploit>>>,
    pub scripts: Vec<Arc<Mutex<Script>>>,
    pub last_tick: web_time::Instant,
    pub player_unlocks: PlayerUnlocks,
}