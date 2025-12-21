mod player;
mod ui;
mod script;
mod server;

mod macros;

use std::sync::{Arc, Mutex};
use crate::macros::get_localized;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use unic_langid::LanguageIdentifier;
use crate::script::Script;
use crate::server::Server;
use crate::ui::{Panel, MarketPanel, ServersPanel, ScriptsPanel};

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

#[derive(Resource)]
struct PlayerState {
    progression: TutorialProgression,
    language_identifier: LanguageIdentifier,
    credits: u128,
    servers: Vec<Arc<Mutex<Server>>>,
    scripts: Vec<Arc<Mutex<Script>>>,
}

#[derive(Resource)]
struct TestWindowState {
    open: bool,
}

enum ActivePanel {
    Home,
    Market,
    Servers,
    Scripts
}

#[derive(Resource)]
struct UiState {
    active_panel: ActivePanel,
    market_panel_state: MarketPanel,
    server_panel_state: ServersPanel,
    scripts_panel_state: ScriptsPanel,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, (update_ui, tutorial_ui_system))
        .insert_resource(PlayerState {
            progression: TutorialProgression::None,
            language_identifier: "en-US".parse().unwrap(),
            credits: 87,
            servers: vec![
                Arc::new(Mutex::new(Server {
                    name: "fe80:0070::".to_string(),
                    threads: 2,
                    clock_speed_hz: 2_000_000_000,
                    stats: vec![],
                }))
            ],
            scripts: vec![],
        })
        .insert_resource(TestWindowState { open: true })
        .insert_resource(UiState {
            active_panel: ActivePanel::Home,
            market_panel_state: MarketPanel {},
            server_panel_state: ServersPanel {},
            scripts_panel_state: ScriptsPanel {},
        })
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_ui(
    mut context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut player_state: ResMut<PlayerState>,
) -> Result {
    // Panels need the same context I guess?
    let ctx = context.ctx_mut()?;
    update_side_panel(ctx, &mut ui_state, &mut player_state)?;

    // Main panel must be last
    update_main_panel(ctx, &mut ui_state, &mut player_state)?;

    Ok(())
}

fn update_main_panel(
    ctx: &mut egui::Context,
    ui_state: &mut UiState,
    player_state: &mut PlayerState,
) -> Result {
    egui::CentralPanel::default().show(ctx, |ui| {
        match ui_state.active_panel {
            ActivePanel::Home => {
                // ZJ-TODO
            }
            ActivePanel::Market => {
                ui_state.market_panel_state.update(ui, player_state);
            }
            ActivePanel::Servers => {
                ui_state.server_panel_state.update(ui, player_state);
            }
            ActivePanel::Scripts => {
                ui_state.scripts_panel_state.update(ui, player_state);
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

                    // ZJ-TODO: show servers
                }
            });
        }

        if player_state.progression.is_complete() {
            ui.label(loc!(player_state, "ui_menu_sidebar_glossary_tab"));
        }
    });

    Ok(())
}

fn tutorial_ui_system(
    mut context: EguiContexts,
    mut window_state: ResMut<TestWindowState>,
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

                // ZJ-TODO: remove this once script logic implemented
                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
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

                if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
                    player_state.progression.advance();
                }
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
