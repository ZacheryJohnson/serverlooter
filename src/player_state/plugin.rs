use std::sync::{Arc, Mutex};
use std::time::Instant;
use bevy::app::{App, FixedUpdate, Plugin};
use crate::inventory::Inventory;
use crate::server::{Server, ServerStatInstances};
use crate::make_exploit_target;
use crate::player_state::state::{PlayerState, PlayerUnlocks};
use crate::player_state::systems::*;
use crate::tutorial::progression::TutorialProgression;

pub struct PlayerStatePlugin;

impl Plugin for PlayerStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_script_created)
            .add_observer(on_request_start_exploit)
            .add_observer(on_request_stop_exploit)
            .add_observer(on_request_pause_exploit)
            .add_observer(on_request_restart_exploit)
            .add_observer(on_request_resume_exploit)
            .add_observer(on_modify_credits)
            .add_observer(on_request_purchase_unlock)
            .add_systems(FixedUpdate, tick_active_exploits)
            .insert_resource(PlayerState {
                progression: TutorialProgression::None,
                language_identifier: "en-US".parse().unwrap(),
                credits: 87,
                inventory: Inventory::new(),
                servers: vec![
                    Arc::new(Mutex::new(Server {
                        name: "fe80:0070::".to_string(),
                        threads: 2,
                        clock_speed_hz: 2_000_000,
                        stats: ServerStatInstances::new(),
                    }))
                ],
                known_targets: vec![
                    make_exploit_target(),
                ],
                active_exploits: vec![],
                scripts: vec![],
                last_tick: Instant::now(),
                player_unlocks: PlayerUnlocks {
                    exploit_auto_reconnect: false,
                }
            });
    }
}