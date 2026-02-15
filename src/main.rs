mod ui;
mod script;
mod server;
mod inventory;

mod macros;
mod event;
mod algorithm;
mod executor;
mod active_exploit;
mod player_state;
mod tutorial;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::macros::get_localized;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
#[cfg(debug_assertions)] // debug/dev builds only
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy_egui::EguiPlugin;
use uuid::Uuid;
use crate::active_exploit::{ActiveExploit, ExploitTarget};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::{AlgorithmEffect, AlgorithmEffectTarget, AlgorithmEffectValue};
use crate::algorithm::id::AlgorithmId;
use crate::algorithm::procedure::AlgorithmProcedure;
use crate::inventory::plugin::InventoryPlugin;
use crate::player_state::plugin::PlayerStatePlugin;
use crate::player_state::state::{PlayerState, PlayerUnlock};
use crate::script::{Script, ScriptId};
use crate::server::{Server, ServerStatInstance, ServerStatInstances, ServerStatSource, ServerStatType};
use crate::tutorial::plugin::TutorialPlugin;
use crate::ui::plugin::UiPlugin;

const TICKS_PER_SECOND: u8 = 20;
const _: () = assert!(
    1000 % TICKS_PER_SECOND as u32 == 0,
    "TICKS_PER_SECOND must cleanly factor into 1000, such that TIME_BETWEEN_TICKS isn't fractional"
);
const TIME_BETWEEN_TICKS: Duration = Duration::from_millis(1000 / TICKS_PER_SECOND as u64);

fn make_exploit_target() -> Arc<Mutex<ExploitTarget>> {
    Arc::new(Mutex::new(ExploitTarget::new(
        Arc::new(Mutex::new(Server {
            name: "KawaiiCo".to_string(),
            threads: 1,
            clock_speed_hz: 1_600_000,
            stats: ServerStatInstances::from(&[
                ServerStatInstance::new(ServerStatSource::Innate, ServerStatType::SiphonResist, 3),
                ServerStatInstance::new(ServerStatSource::Innate, ServerStatType::ExfilResist, 8),
            ])
        })),
        Arc::new(Mutex::new(Script {
            id: ScriptId::Invalid,
            procedures: vec![
                AlgorithmProcedure::from(&[
                    Arc::new(Mutex::new(Algorithm {
                        id: AlgorithmId::Id(Uuid::new_v4()),
                        instruction_count: 1_000_000,
                        instruction_effects: vec![
                            (250_000, vec![AlgorithmEffect::Terminate { potency: AlgorithmEffectValue::Static(1) } ]),
                            (500_000, vec![AlgorithmEffect::Terminate { potency: AlgorithmEffectValue::Static(1) } ]),
                            (750_000, vec![AlgorithmEffect::Terminate { potency: AlgorithmEffectValue::Static(1) } ]),
                            (1_000_000, vec![AlgorithmEffect::Terminate { potency: AlgorithmEffectValue::Static(1) } ]),
                        ],
                    }))
                ]),
                AlgorithmProcedure::from(&[
                    Arc::new(Mutex::new(Algorithm {
                        id: AlgorithmId::Id(Uuid::new_v4()),
                        instruction_count: 1_000_000,
                        instruction_effects: vec![
                            (1_000_000, vec![
                                // ZJ-TODO: would be nice to have a PurgeAll
                                AlgorithmEffect::Purge {
                                    potency: AlgorithmEffectValue::Static(1),
                                    target: AlgorithmEffectTarget::SelfServer,
                                    stat: ServerStatType::SiphonResist,
                                },
                                AlgorithmEffect::Purge {
                                    potency: AlgorithmEffectValue::Static(1),
                                    target: AlgorithmEffectTarget::SelfServer,
                                    stat: ServerStatType::ExfilResist,
                                }
                            ])
                        ]
                    }))
                ])
            ],
        }))
    )))
}

fn main() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins((
            InventoryPlugin,
            PlayerStatePlugin,
            TutorialPlugin,
            UiPlugin,
        ))
        .add_systems(Update, handle_keyboard);

    #[cfg(debug_assertions)]
    {
        app.add_plugins(LogDiagnosticsPlugin::default());
    }

    app.run();
}

fn handle_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    // Cheat: auto-unlock everything
    if keys.just_pressed(KeyCode::Digit0) {
        player_state.player_unlocks.exploit_auto_reconnect = true;
    }

    Ok(())
}