mod ui;
mod script;
mod server;
mod inventory;

mod macros;
mod event;
mod algorithm;
mod executor;
mod active_exploit;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::macros::{clock_speed_to_loc_args, get_localized};
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
#[cfg(debug_assertions)] // debug/dev builds only
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_egui::egui::Widget;
use unic_langid::LanguageIdentifier;
use uuid::Uuid;
use crate::active_exploit::{ActiveExploit, ActiveExploitStatus, ExploitTarget};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::{AlgorithmEffect, AlgorithmEffectApplication, AlgorithmEffectTarget, AlgorithmEffectValue};
use crate::algorithm::generator::AlgorithmGenerator;
use crate::algorithm::id::AlgorithmId;
use crate::algorithm::procedure::AlgorithmProcedure;
use crate::event::modify_credits::{ModificationSource, ModifyCreditsEvent};
use crate::event::request_pause_exploit::RequestPauseExploitEvent;
use crate::event::request_purchase_unlock::RequestPurchaseUnlockEvent;
use crate::event::request_restart_exploit::RequestRestartExploitEvent;
use crate::event::request_resume_exploit::RequestResumeExploitEvent;
use crate::event::request_start_exploit::RequestStartExploitEvent;
use crate::event::request_stop_exploit::RequestStopExploitEvent;
use crate::inventory::{on_inventory_item_added, on_inventory_item_removed, Inventory, InventoryItem, InventoryItemAdded};
use crate::script::{Script, ScriptCreatedEvent, ScriptId};
use crate::server::{Server, ServerStatInstance, ServerStatInstances, ServerStatSource, ServerStatType, ServerStats};
use crate::ui::panel::{Panel, exploit::*, market::*, script::*, server::*};
use crate::ui::window::active_exploit::ActiveExploitWindow;
use crate::ui::window::Window;

const TICKS_PER_SECOND: u8 = 20;
const _: () = assert!(
    1000 % TICKS_PER_SECOND as u32 == 0,
    "TICKS_PER_SECOND must cleanly factor into 1000, such that TIME_BETWEEN_TICKS isn't fractional"
);
const TIME_BETWEEN_TICKS: Duration = Duration::from_millis(1000 / TICKS_PER_SECOND as u64);

enum TutorialProgression {
    /// The option to start with a tutorial hasn't been presented to the player yet.
    None,

    /// The player skipped the tutorial.
    /// This is equivalent to Complete, but allows us to be condescending if the player needs it.
    Skipped,

    /// The player has completed the tutorial.
    Complete,

    /// The player has started the tutorial, but has not yet completed any steps.
    Start,

    /// The server section is added to the menu sidebar,
    /// and the user is prompted to click the only server tab within.
    ServersTabIntroduced,

    ServerClicked,

    /// The develop section and scripts tab is added to the menu sidebar,
    /// and the user is prompted to click the "scripts" tab within.
    DevelopScriptsIntroduced,

    ScriptClicked,

    /// The market tab is added to the menu sidebar.
    MarketTabIntroduced,

    MarketTabClicked,

    /// The exploit tab is added to the menu sidebar.
    ExploitTabIntroduced,

    ExploitServersShown,

    ExploitCorpAClicked,

    ExploitCorpASuccess,

    ExploitCorpBClicked,

    MarketAlgorithmPrompted,
}

impl TutorialProgression {
    fn show_servers_tab(&self) -> bool {
        match self {
            TutorialProgression::None => false,
            TutorialProgression::Start => false,
            _ => true,
        }
    }

    fn show_develop_tab(&self) -> bool {
        self.show_servers_tab() && match self {
            TutorialProgression::ServersTabIntroduced => false,
            TutorialProgression::ServerClicked => false,
            _ => true,
        }
    }

    fn show_market_tab(&self) -> bool {
        self.show_develop_tab() && match self {
            TutorialProgression::DevelopScriptsIntroduced => false,
            TutorialProgression::ScriptClicked => false,
            _ => true,
        }
    }

    fn show_exploit_tab(&self) -> bool {
        self.show_market_tab() && match self {
            TutorialProgression::MarketTabIntroduced => false,
            TutorialProgression::MarketTabClicked => false,
            _ => true,
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            TutorialProgression::Complete | TutorialProgression::Skipped => true,
            _ => false,
        }
    }

    fn advance(&mut self) {
        match self {
            TutorialProgression::None => {},
            TutorialProgression::Skipped => {},
            TutorialProgression::Complete => {},
            TutorialProgression::Start => *self = TutorialProgression::ServersTabIntroduced,
            TutorialProgression::ServersTabIntroduced => *self = TutorialProgression::ServerClicked,
            TutorialProgression::ServerClicked => *self = TutorialProgression::DevelopScriptsIntroduced,
            TutorialProgression::DevelopScriptsIntroduced => *self = TutorialProgression::ScriptClicked,
            TutorialProgression::ScriptClicked => *self = TutorialProgression::MarketTabIntroduced,
            TutorialProgression::MarketTabIntroduced => *self = TutorialProgression::MarketTabClicked,
            TutorialProgression::MarketTabClicked => *self = TutorialProgression::ExploitTabIntroduced,
            TutorialProgression::ExploitTabIntroduced => *self = TutorialProgression::ExploitServersShown,
            TutorialProgression::ExploitServersShown => *self = TutorialProgression::ExploitCorpAClicked,
            TutorialProgression::ExploitCorpAClicked => *self = TutorialProgression::ExploitCorpASuccess,
            TutorialProgression::ExploitCorpASuccess => *self = TutorialProgression::ExploitCorpBClicked,
            TutorialProgression::ExploitCorpBClicked => *self = TutorialProgression::MarketAlgorithmPrompted,
            TutorialProgression::MarketAlgorithmPrompted => *self = TutorialProgression::Complete,
        }
    }
}

pub enum PlayerUnlock {
    ExploitAutoReconnect,
}

pub struct PlayerUnlocks {
    exploit_auto_reconnect: bool,
}

#[derive(Resource)]
pub struct PlayerState {
    progression: TutorialProgression,
    language_identifier: LanguageIdentifier,
    credits: u128,
    inventory: Inventory,
    servers: Vec<Arc<Mutex<Server>>>,
    known_targets: Vec<Arc<Mutex<ExploitTarget>>>,
    active_exploits: Vec<Arc<Mutex<ActiveExploit>>>,
    scripts: Vec<Arc<Mutex<Script>>>,
    last_tick: Instant,
    player_unlocks: PlayerUnlocks,
}

enum ActivePanel {
    Home,
    Market,
    Servers,
    Scripts,
    Exploit,
}

#[derive(Resource)]
struct UiState {
    active_panel: ActivePanel,
    market_panel_state: MarketPanel,
    server_panel_state: ServersPanel,
    scripts_panel_state: ScriptsPanel,
    exploit_panel_state: ExploitPanel,

    active_exploit_windows: Vec<ActiveExploitWindow>,
}

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
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, (update_ui, tutorial_ui_system))
        .add_systems(FixedUpdate, tick_player_state)
        .add_systems(Update, handle_keyboard)
        .add_observer(tutorial_on_script_created)
        .add_observer(on_script_created)
        .add_observer(on_inventory_item_added)
        .add_observer(on_inventory_item_removed)
        .add_observer(on_request_start_exploit)
        .add_observer(on_request_stop_exploit)
        .add_observer(on_request_pause_exploit)
        .add_observer(on_request_restart_exploit)
        .add_observer(on_request_resume_exploit)
        .add_observer(on_modify_credits)
        .add_observer(on_request_purchase_unlock)
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
        })
        .insert_resource(UiState {
            active_panel: ActivePanel::Home,
            market_panel_state: MarketPanel {},
            server_panel_state: ServersPanel {},
            scripts_panel_state: ScriptsPanel::new(),
            exploit_panel_state: ExploitPanel::new(),
            active_exploit_windows: vec![],
        });

    #[cfg(debug_assertions)]
    {
        app.add_plugins(LogDiagnosticsPlugin::default());
    }

    app.run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_ui(
    mut commands: Commands,
    mut context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    // Panels need the same context I guess?
    let ctx = context.ctx_mut()?;
    update_side_panel(ctx, &mut ui_state, &mut player_state)?;

    for window in &mut ui_state.active_exploit_windows {
        window.update(&mut commands, ctx, &mut player_state);
    }

    // Main panel must be last
    update_main_panel(&mut commands, ctx, &mut ui_state, &mut player_state)?;

    Ok(())
}

fn update_main_panel(
    commands: &mut Commands,
    ctx: &mut egui::Context,
    ui_state: &mut UiState,
    player_state: &mut PlayerState,
) -> Result {
    egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            ui.horizontal(|ui| {
                egui::Label::new(format!("${}", player_state.credits))
                    .halign(egui::Align::RIGHT)
                    .ui(ui);
            });
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        match ui_state.active_panel {
            ActivePanel::Home => {
                // ZJ-TODO
            }
            ActivePanel::Market => {
                ui_state.market_panel_state.update(commands, ctx, ui, player_state);
            }
            ActivePanel::Servers => {
                ui_state.server_panel_state.update(commands, ctx, ui, player_state);
            }
            ActivePanel::Scripts => {
                ui_state.scripts_panel_state.update(commands, ctx, ui, player_state);
            }
            ActivePanel::Exploit => {
                ui_state.exploit_panel_state.update(commands, ctx, ui, player_state);
            }
        }
    });

    Ok(())
}

fn update_side_panel(
    ctx: &mut egui::Context,
    ui_state: &mut UiState,
    player_state: &mut PlayerState,
) -> Result {
    let side_panel = egui::SidePanel::left("side_panel")
        .resizable(false);

    side_panel.show(ctx, |ui| {
        if player_state.progression.show_market_tab() {
            if ui.selectable_label(false, loc!(player_state, "ui_menu_sidebar_market_tab")).clicked() {
                if matches!(player_state.progression, TutorialProgression::MarketTabIntroduced) {
                    player_state.progression = TutorialProgression::MarketTabClicked;
                }

                ui_state.active_panel = ActivePanel::Market;
            }
        }
        if player_state.progression.show_servers_tab() {
            ui.collapsing(loc!(player_state, "ui_menu_sidebar_servers_section"), |ui| {
                for server_arc in &player_state.servers {
                    let server = server_arc.lock().unwrap();
                    if ui.selectable_label(false, &server.name).clicked() {
                        if matches!(player_state.progression, TutorialProgression::ServersTabIntroduced) {
                            player_state.progression = TutorialProgression::ServerClicked;
                        }

                        ui_state.active_panel = ActivePanel::Servers;
                    }
                }
            });
        }
        if player_state.progression.show_develop_tab() {
            ui.collapsing(loc!(player_state, "ui_menu_sidebar_develop_section"), |ui| {
               if ui.selectable_label(false, loc!(player_state, "ui_menu_sidebar_scripts_tab")).clicked() {
                   if matches!(player_state.progression, TutorialProgression::DevelopScriptsIntroduced) {
                       player_state.progression = TutorialProgression::ScriptClicked;
                   }

                   ui_state.active_panel = ActivePanel::Scripts;
               }
            });
        }
        if player_state.progression.show_exploit_tab() {
            ui.collapsing(loc!(player_state, "ui_menu_sidebar_black_hat_section"), |ui| {
                if ui.selectable_label(false, loc!(player_state, "ui_menu_sidebar_exploit_tab")).clicked() {
                    if matches!(player_state.progression, TutorialProgression::ExploitTabIntroduced) {
                        player_state.progression = TutorialProgression::ExploitServersShown;
                    }

                    ui_state.active_panel = ActivePanel::Exploit;
                }
            });
        }

        if player_state.progression.is_complete() {
            ui.label(loc!(player_state, "ui_menu_sidebar_glossary_tab"));
        }
    });

    Ok(())
}

fn on_request_start_exploit(
    evt: On<RequestStartExploitEvent>,
    mut player_state: ResMut<PlayerState>,
    mut ui_state: ResMut<UiState>,
) -> Result {
    let target = evt.target.clone();
    let script = evt.script.clone();
    let server = evt.server.clone();

    // ZJ-TODO: validate server can accommodate another process
    // ZJ-TODO: validate server can meets thread minimums

    // ZJ-TODO: actually implement way to shift resource allocation
    //          for now, just time share equally
    let target_server_name = lock_and_clone!(evt.server, name);
    let new_total_processes = player_state
        .active_exploits
        .iter()
        .filter(|exploit| lock_and_clone!(exploit, hosting_server, name) == target_server_name)
        .count() as u64 + 1;

    let new_clock_speed_per_process = lock_and_clone!(server, clock_speed_hz) / new_total_processes;

    for existing_exploit in &mut player_state.active_exploits {
        let mut existing_exploit = existing_exploit.lock().unwrap();
        existing_exploit.clock_allocation_hz = new_clock_speed_per_process;
    }

    let auto_reconnect = player_state.player_unlocks.exploit_auto_reconnect;
    let active_exploit = Arc::new(Mutex::new(ActiveExploit::new(
        target,
        script,
        server,
        new_clock_speed_per_process,
        auto_reconnect,
    )));

    ui_state.active_exploit_windows.push(ActiveExploitWindow::new(active_exploit.clone()));
    player_state.active_exploits.push(active_exploit);

    if matches!(player_state.progression, TutorialProgression::ExploitServersShown) {
        player_state.progression.advance();
    }

    Ok(())
}

fn on_request_stop_exploit(
    evt: On<RequestStopExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    player_state.active_exploits.retain(|exploit| {
        lock_and_clone!(exploit, id) != evt.exploit_id
    });

    Ok(())
}

fn on_request_pause_exploit(
    evt: On<RequestPauseExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    if let Some(exploit) = player_state
        .active_exploits
        .iter_mut()
        .find(|exploit| {
            lock_and_clone!(exploit, id) == evt.exploit_id
        })
    {
        exploit.lock().unwrap().stop_execution();
    }

    Ok(())
}

fn on_request_restart_exploit(
    evt: On<RequestRestartExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    if let Some(exploit) = player_state
        .active_exploits
        .iter_mut()
        .find(|exploit| {
            lock_and_clone!(exploit, id) == evt.exploit_id
        })
    {
        exploit.lock().unwrap().restart();
    }

    Ok(())
}

fn on_request_resume_exploit(
    evt: On<RequestResumeExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    if let Some(exploit) = player_state
        .active_exploits
        .iter_mut()
        .find(|exploit| {
            lock_and_clone!(exploit, id) == evt.exploit_id
        })
    {
        exploit.lock().unwrap().start_execution();
    }

    Ok(())
}

fn on_modify_credits(
    evt: On<ModifyCreditsEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    player_state.credits = player_state.credits.saturating_add_signed(evt.credits as i128);

    Ok(())
}

fn on_request_purchase_unlock(
    evt: On<RequestPurchaseUnlockEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    if evt.credit_cost >= player_state.credits {
        // ZJ-TODO: error messaging
        return Ok(());
    }

    player_state.credits -= evt.credit_cost;

    // ZJ-TODO: rather than having a separate list of bools,
    //          it'd be nice to just have a collection of enums

    match evt.unlock {
        PlayerUnlock::ExploitAutoReconnect => {
            player_state.player_unlocks.exploit_auto_reconnect = true;
        }
    }

    Ok(())
}

fn on_script_created(
    evt: On<ScriptCreatedEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    let script = evt.script.to_owned();
    player_state.scripts.push(Arc::new(Mutex::new(script)));

    Ok(())
}

fn tutorial_on_script_created(
    _: On<ScriptCreatedEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    if matches!(player_state.progression, TutorialProgression::ScriptClicked) {
        player_state.progression.advance();
    }

    Ok(())
}

fn tutorial_ui_system(
    mut context: EguiContexts,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    let window = egui::Window::new(loc!(player_state, "ui_window_tutorial_title"))
        .default_pos(egui::pos2(
            context.ctx_mut()?.content_rect().width() / 2.0,
            context.ctx_mut()?.content_rect().height() / 2.0,
        ))
        .resizable(true)
        .constrain(true);

    match player_state.progression {
        TutorialProgression::Skipped | TutorialProgression::Complete => { /* no-op */ },
        TutorialProgression::None => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_ask_start"));
                if ui.button(loc!(player_state, "ui_confirmation_yes")).clicked() {
                    player_state.progression = TutorialProgression::Start;
                }
                if ui.button(loc!(player_state, "ui_confirmation_no")).clicked() {
                    player_state.progression = TutorialProgression::Skipped;
                }
            });
        }
        TutorialProgression::Start => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_1"));
                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::ServersTabIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_2"));
            });
        }
        TutorialProgression::ServerClicked => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_3"));
                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::DevelopScriptsIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_4"));
            });
        }
        TutorialProgression::ScriptClicked => {
            window.show(context.ctx_mut()?, |ui| {
               ui.label(loc!(player_state, "tutorial_stage_5"));
            });
        }
        TutorialProgression::MarketTabIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
               ui.label(loc!(player_state, "tutorial_stage_6"));
            });
        }
        TutorialProgression::MarketTabClicked => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_7"));

                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::ExploitTabIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_8"));
            });
        }
        TutorialProgression::ExploitServersShown => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_9"));
            });
        }
        TutorialProgression::ExploitCorpAClicked => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_10"));

                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::ExploitCorpASuccess => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, "tutorial_stage_11"));

                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::ExploitCorpBClicked => {
            window.show(context.ctx_mut()?, |ui| {
               ui.label(loc!(player_state, "tutorial_stage_12"));

                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::MarketAlgorithmPrompted => {
            window.show(context.ctx_mut()?, |ui| {
               ui.label(loc!(player_state, "tutorial_stage_13"));

                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
            });
        }
    }

    Ok(())
}

fn tick_player_state(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
) {
    // Only tick at a fixed rate
    let time_since_last_tick = Instant::now().duration_since(player_state.last_tick);
    if time_since_last_tick < TIME_BETWEEN_TICKS {
        return;
    }

    player_state.last_tick = Instant::now();

    let mut pending_effects = vec![];
    for active_exploit in &player_state.active_exploits {
        let (new_host_effects, new_target_effects) = {
            let mut active_exploit = active_exploit.lock().unwrap();
            // ZJ-TODO: compared allocated speed vs server's current capacity
            //          this should probably be refactored
            let server_speed = active_exploit.clock_allocation_hz;
            let ticks_since_last = (server_speed as f64 * time_since_last_tick.as_secs_f64()).floor() as u64;

            let target_server_speed = active_exploit.target.lock().unwrap().server.lock().unwrap().clock_speed_hz;
            let target_ticks_since_last = (target_server_speed as f64 * time_since_last_tick.as_secs_f64()).floor() as u64;

            active_exploit.tick(ticks_since_last, target_ticks_since_last)
        };

        pending_effects.push((active_exploit.clone(), new_host_effects));
        pending_effects.push((active_exploit.clone(), new_target_effects));
    }

    for (active_exploit, pending_effects) in pending_effects {
        for pending_effect in pending_effects {
            process_algorithm_effect_application(
                &mut commands,
                pending_effect,
                active_exploit.clone(),
            );
        }
    }

    for active_exploit in &player_state.active_exploits {
        let active_exploit = active_exploit.lock().unwrap();
        if matches!(active_exploit.status(), ActiveExploitStatus::Disconnected) {
            if *active_exploit.auto_reconnect.lock().unwrap() {
                commands.trigger(RequestRestartExploitEvent { exploit_id: active_exploit.id })
            }
        }
    }
}

fn process_algorithm_effect_application(
    commands: &mut Commands,
    application: AlgorithmEffectApplication,
    active_exploit: Arc<Mutex<ActiveExploit>>,
) {
    let mut active_exploit = active_exploit.lock().unwrap();
    match application.effect {
        AlgorithmEffect::Terminate { potency }  => {
            let value = potency.make_value();
            let old_health = active_exploit.connection_current_health.lock().unwrap().clone();
            let new_health = old_health.saturating_sub(value.abs() as u32);
            *active_exploit.connection_current_health.lock().unwrap() = new_health;

            if new_health == 0 {
                active_exploit.stop_execution();
            }
        }
        AlgorithmEffect::Siphon { potency } => {
            let value = potency.make_value();
            let target_server = application.target_server.lock().unwrap();
            let target_stats = &target_server.stats;

            let target_defense = target_stats.value_of(ServerStatType::SiphonResist);
            let siphon_value = (value - target_defense).max(0) as i64;

            commands.trigger(ModifyCreditsEvent {
                credits: siphon_value,
                source: ModificationSource::Script(application.script.lock().unwrap().id.clone()),
            });
        }
        AlgorithmEffect::Exfil { potency } => {
            let value = potency.make_value();
            let target_server = application.target_server.lock().unwrap();
            let target_stats = &target_server.stats;
            let target_defense = target_stats.value_of(ServerStatType::ExfilResist);
            let exfil_value = value - target_defense;
            if exfil_value <= 0 {
                return;
            }

            // ZJ-TODO: pass potency to generator
            let algorithm = AlgorithmGenerator::generate();

            commands.trigger(InventoryItemAdded {
                item: InventoryItem::Algorithm(algorithm),
            });
        }
        AlgorithmEffect::Modify { target, stat, potency } => {
            let server = match target {
                AlgorithmEffectTarget::SelfServer => application.host_server,
                AlgorithmEffectTarget::TargetServer => application.target_server,
            };

            let script_id = application.script.lock().unwrap().id.clone();

            let purged_stats = server.lock().unwrap().stats.apply_and_purge(
                ServerStatInstance::new(
                    ServerStatSource::Script(script_id),
                    stat.to_owned(),
                    potency.make_value()
                )
            );
        }
        AlgorithmEffect::Purge { target, stat, potency } => {
            let (server, is_self) = match target {
                AlgorithmEffectTarget::SelfServer => (application.host_server, true),
                AlgorithmEffectTarget::TargetServer => (application.target_server, false),
            };

            let mut server = server.lock().unwrap();
            let script_id = active_exploit.script.lock().unwrap().id.clone();

            let server_stat_value = server.stats.modification_of(stat.to_owned());
            if server_stat_value >= 0 && is_self || server_stat_value <= 0 && !is_self {
                return;
            }

            let potency_roll = {
                if is_self {
                    potency.make_value().max(0)
                } else {
                    potency.make_value().min(0)
                }
            };

            let purged_stats = server.stats.apply_and_purge(
                ServerStatInstance::new(
                    ServerStatSource::Script(script_id),
                    stat.to_owned(),
                    potency_roll
                )
            );
        }
    }
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