mod ui;
mod script;
mod server;
mod inventory;

mod macros;
mod event;
mod algorithm;
mod executor;

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
use crate::algorithm::effect::{AlgorithmEffect, AlgorithmEffectTarget};
use crate::algorithm::generator::AlgorithmGenerator;
use crate::event::request_pause_exploit::RequestPauseExploitEvent;
use crate::event::request_resume_exploit::RequestResumeExploitEvent;
use crate::event::request_start_exploit::RequestStartExploitEvent;
use crate::event::request_stop_exploit::RequestStopExploitEvent;
use crate::executor::Executor;
use crate::inventory::{on_inventory_item_added, on_inventory_item_removed, Inventory, InventoryItem, InventoryItemAdded};
use crate::script::{Script, ScriptCreatedEvent, ScriptExecutor};
use crate::server::{Server, ServerStatInstance, ServerStatSource, ServerStatType};
use crate::ui::panel::{Panel, exploit::*, market::*, script::*, server::*};

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

pub struct ExploitTarget {
    pub id: Uuid,
    pub server: Arc<Mutex<Server>>,
}

#[derive(Clone)]
pub struct ActiveExploit {
    pub target: Arc<Mutex<ExploitTarget>>,
    pub script: Arc<Mutex<Script>>,
    pub hosting_server: Arc<Mutex<Server>>,
    pub clock_allocation_hz: u64,

    id: Uuid,
    script_executor: ScriptExecutor,
}

#[derive(Clone)]
pub enum ActiveExploitStatus {
    Paused,
    Running,
}

impl ActiveExploit {
    pub fn new(
        target: Arc<Mutex<ExploitTarget>>,
        script: Arc<Mutex<Script>>,
        hosting_server: Arc<Mutex<Server>>,
        clock_allocation_hz: u64,
    ) -> ActiveExploit {
        let mut script_executor = script.lock().unwrap().clone().into_executor();
        script_executor.start_execution();

        let id = Uuid::new_v4();

        ActiveExploit {
            target,
            script,
            hosting_server,
            clock_allocation_hz,
            id,
            script_executor,
        }
    }

    pub fn progress(&self) -> u64 {
        self.script_executor.progress()
    }

    pub fn total_instructions(&self) -> u64 {
        self.script_executor.total_instructions()
    }

    pub fn status(&self) -> ActiveExploitStatus {
        if self.script_executor.is_paused() {
            ActiveExploitStatus::Paused
        } else {
            ActiveExploitStatus::Running
        }
    }
}

#[derive(Resource)]
pub struct PlayerState {
    progression: TutorialProgression,
    language_identifier: LanguageIdentifier,
    credits: u128,
    inventory: Inventory,
    servers: Vec<Arc<Mutex<Server>>>,
    known_targets: Vec<Arc<Mutex<ExploitTarget>>>,
    active_exploits: Vec<ActiveExploit>,
    scripts: Vec<Arc<Mutex<Script>>>,
    last_tick: Instant,
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
        .add_observer(tutorial_on_script_created)
        .add_observer(on_script_created)
        .add_observer(on_inventory_item_added)
        .add_observer(on_inventory_item_removed)
        .add_observer(on_request_start_exploit)
        .add_observer(on_request_stop_exploit)
        .add_observer(on_request_pause_exploit)
        .add_observer(on_request_resume_exploit)
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
                    stats: vec![],
                }))
            ],
            known_targets: vec![Arc::new(Mutex::new(ExploitTarget {
                id: Uuid::new_v4(),
                server: Arc::new(Mutex::new(Server {
                    name: "KawaiiCo".to_string(),
                    threads: 1,
                    clock_speed_hz: 1_600_000,
                    stats: vec![
                        ServerStatInstance::new(ServerStatSource::Innate, ServerStatType::SiphonResist, 3),
                    ]
                }))
            }))],
            active_exploits: vec![],
            scripts: vec![],
            last_tick: Instant::now(),
        })
        .insert_resource(UiState {
            active_panel: ActivePanel::Home,
            market_panel_state: MarketPanel {},
            server_panel_state: ServersPanel {},
            scripts_panel_state: ScriptsPanel::new(),
            exploit_panel_state: ExploitPanel::new(),
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

    for active_exploit in &player_state.active_exploits {
        let mut xd = true;
        let window = egui::Window::new("Active Exploit")
            .id(format!("active_exploit_{}", active_exploit.id.to_string()).into())
            .fade_in(true)
            .fade_out(true)
            .open(&mut xd);
        window.show(&ctx, |ui| {
            ui.label(format!("Your Server: {}", lock_and_clone!(active_exploit.hosting_server, name)));
            ui.label(format!("Target Server: {}", lock_and_clone!(active_exploit.target, server, name)));
            ui.label(format!("Allocated CPU: {}",
                loc!(
                    player_state,
                    "ui_server_clock_speed",
                    clock_speed_to_loc_args(active_exploit.clock_allocation_hz)
                )
            ));
            egui::widgets::ProgressBar::new(active_exploit.progress() as f32 / active_exploit.total_instructions() as f32)
                .desired_width(ui.available_width() / 2.0)
                .show_percentage()
                .ui(ui);
            match active_exploit.status() {
                ActiveExploitStatus::Paused => {
                    if ui.button("Resume").clicked() {
                        commands.trigger(RequestResumeExploitEvent { exploit_id: active_exploit.id });
                    }
                }
                ActiveExploitStatus::Running => {
                    if ui.button("Pause").clicked() {
                        commands.trigger(RequestPauseExploitEvent { exploit_id: active_exploit.id });
                    }
                }
            }
            if ui.button(loc!(player_state, "ui_confirmation_stop")).clicked() {
                commands.trigger(RequestStopExploitEvent {
                    exploit_id: active_exploit.id,
                });
            }
        });
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
        .filter(|exploit| lock_and_clone!(exploit.hosting_server, name) == target_server_name)
        .count() as u64 + 1;

    let new_clock_speed_per_process = lock_and_clone!(server, clock_speed_hz) / new_total_processes;

    for existing_exploit in &mut player_state.active_exploits {
        existing_exploit.clock_allocation_hz = new_clock_speed_per_process;
    }

    player_state.active_exploits.push(ActiveExploit::new(target, script, server, new_clock_speed_per_process));

    if matches!(player_state.progression, TutorialProgression::ExploitServersShown) {
        player_state.progression.advance();
    }

    Ok(())
}

fn on_request_stop_exploit(
    evt: On<RequestStopExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    player_state.active_exploits.retain(|exploit| exploit.id != evt.exploit_id);

    Ok(())
}

fn on_request_pause_exploit(
    evt: On<RequestPauseExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    if let Some(exploit) = player_state
        .active_exploits
        .iter_mut()
        .find(|exploit| exploit.id == evt.exploit_id)
    {
        exploit.script_executor.stop_execution();
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
        .find(|exploit| exploit.id == evt.exploit_id)
    {
        exploit.script_executor.start_execution();
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
    for active_exploit in &mut player_state.active_exploits {
        // ZJ-TODO: compared allocated speed vs server's current capacity
        //          this should probably be refactored
        let server_speed = active_exploit.clock_allocation_hz;
        let ticks_since_last = (server_speed as f64 * time_since_last_tick.as_secs_f64()).floor() as u64;

        let new_effects = active_exploit.script_executor.tick_execution(ticks_since_last);
        if active_exploit.script_executor.is_complete() {
            active_exploit.script_executor = active_exploit.script.lock().unwrap().executor();
            active_exploit.script_executor.start_execution();
        }
        pending_effects.push((active_exploit.clone(), new_effects));
    }

    for (active_exploit, pending_effects) in pending_effects {
        for pending_effect in &pending_effects {
            let active_exploit = active_exploit.clone();

            match pending_effect {
                AlgorithmEffect::Siphon { potency } => {
                    let value = potency.make_value();
                    let active_exploit = active_exploit.clone();
                    let target_server = lock_and_clone!(active_exploit.target, server);
                    let target_server = target_server.lock().unwrap();
                    let target_stats = target_server.stats();

                    // ZJ-TODO: for now, don't let resists fall below 0
                    //          there's design space to let them do so, but too much trivializes content
                    let target_defense = target_stats.value_of(ServerStatType::SiphonResist).max(0);
                    let siphon_value = (value - target_defense).max(0) as u128;

                    // ZJ-TODO: shoot off an event to do this rather than applying it ourselves
                    player_state.credits += siphon_value;
                }
                AlgorithmEffect::Exfil { .. } => {
                    // ZJ-TODO: pass potency to generator
                    let algorithm = AlgorithmGenerator::generate();

                    commands.trigger(InventoryItemAdded {
                        item: InventoryItem::Algorithm(algorithm),
                    });
                }
                AlgorithmEffect::Modify { target, stat, potency } => {
                    let server = match target {
                        AlgorithmEffectTarget::Host => active_exploit.clone().hosting_server.clone(),
                        AlgorithmEffectTarget::Remote => active_exploit.clone().target.lock().unwrap().server.clone(),
                    };

                    let script_id = active_exploit.script.lock().unwrap().id.clone();

                    server.lock().unwrap().stats_mut().apply(
                        ServerStatInstance::new(
                            ServerStatSource::Script(script_id),
                            stat.to_owned(),
                            potency.make_value()
                        )
                    )
                }
            }
        }
    }
}