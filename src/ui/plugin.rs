use crate::{get_localized, lock_and_clone};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::AssetServer;
use bevy::audio::{AudioPlayer, GlobalVolume, PlaybackSettings};
use bevy::camera::Camera2d;
use bevy::prelude::{Commands, On, Res, ResMut};
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use bevy_egui::egui::Widget;
use crate::{loc, PlayerState};
use crate::event::exploit_event::ExploitEvent;
use crate::event::exploit_started::ExploitStarted;
use crate::event::request_start_exploit::RequestStartExploitEvent;
use crate::tutorial::progression::TutorialProgression;
use crate::ui::panel::exploit::ExploitPanel;
use crate::ui::panel::market::MarketPanel;
use crate::ui::panel::Panel;
use crate::ui::panel::script::ScriptsPanel;
use crate::ui::panel::server::ServersPanel;
use crate::ui::state::{ActivePanel, UiState};
use crate::ui::window::Window;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (setup_camera_system, setup_audio))
            .add_systems(EguiPrimaryContextPass, update_ui)
            .add_observer(on_active_exploit_event)
            .add_observer(on_active_exploit_started)
            .insert_resource(UiState {
                image_loaders_initialized: false,
                active_panel: ActivePanel::Home,
                market_panel_state: MarketPanel {},
                server_panel_state: ServersPanel {},
                scripts_panel_state: ScriptsPanel::new(),
                exploit_panel_state: ExploitPanel::new(),
                active_exploit_windows: vec![],
            });
    }
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_audio(mut volume: ResMut<GlobalVolume>) {
    volume.volume = volume.volume.decrease_by_percentage(75f32);
}

fn update_ui(
    mut commands: Commands,
    mut context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut player_state: ResMut<PlayerState>,
    asset_server: Res<AssetServer>,
) -> bevy::prelude::Result {
    // Panels need the same context I guess?
    let ctx = context.ctx_mut()?;

    if !ui_state.image_loaders_initialized
    {
        egui_extras::install_image_loaders(ctx);
        ui_state.image_loaders_initialized = true;
    }

    ctx.style_mut(|style| {
        style.interaction.selectable_labels = false;
    });

    update_side_panel(&mut commands, ctx, &mut ui_state, &mut player_state, &asset_server)?;

    for window in &mut ui_state.active_exploit_windows {
        window.update(&mut commands, ctx, &mut player_state, &asset_server);
    }

    // Main panel must be last
    update_main_panel(&mut commands, ctx, &mut ui_state, &mut player_state, &asset_server)?;

    Ok(())
}

fn update_main_panel(
    commands: &mut Commands,
    ctx: &mut egui::Context,
    ui_state: &mut UiState,
    player_state: &mut PlayerState,
    asset_server: &AssetServer,
) -> bevy::prelude::Result {
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
                ui_state.market_panel_state.update(commands, ctx, ui, player_state, asset_server);
            }
            ActivePanel::Servers => {
                ui_state.server_panel_state.update(commands, ctx, ui, player_state, asset_server);
            }
            ActivePanel::Scripts => {
                ui_state.scripts_panel_state.update(commands, ctx, ui, player_state, asset_server);
            }
            ActivePanel::Exploit => {
                ui_state.exploit_panel_state.update(commands, ctx, ui, player_state, asset_server);
            }
        }
    });

    Ok(())
}

fn update_side_panel(
    commands: &mut Commands,
    ctx: &mut egui::Context,
    ui_state: &mut UiState,
    player_state: &mut PlayerState,
    asset_server: &AssetServer,
) -> bevy::prelude::Result {
    let side_panel = egui::SidePanel::left("side_panel")
        .resizable(false);

    side_panel.show(ctx, |ui| {
        if player_state.progression.show_market_tab() {
            if ui.selectable_label(false, loc!(player_state, "ui_menu_sidebar_market_tab")).clicked() {
                if matches!(player_state.progression, TutorialProgression::MarketTabIntroduced) {
                    player_state.progression = TutorialProgression::MarketTabClicked;
                }

                ui_state.active_panel = ActivePanel::Market;

                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/click.ogg")),
                    PlaybackSettings::ONCE
                ));
            }
        }
        if player_state.progression.show_servers_tab() {
            let collapsing = ui.collapsing(loc!(player_state, "ui_menu_sidebar_servers_section"), |ui| {
                for server_arc in &player_state.servers {
                    let server = server_arc.lock().unwrap();
                    if ui.selectable_label(false, &server.name).clicked() {
                        if matches!(player_state.progression, TutorialProgression::ServersTabIntroduced) {
                            player_state.progression = TutorialProgression::ServerClicked;
                        }

                        ui_state.active_panel = ActivePanel::Servers;

                        commands.spawn((
                            AudioPlayer::new(asset_server.load("audio/click.ogg")),
                            PlaybackSettings::ONCE
                        ));
                    }
                }
            });

            if collapsing.header_response.clicked() {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/click.ogg")),
                    PlaybackSettings::ONCE
                ));
            }
        }
        if player_state.progression.show_develop_tab() {
            let collapsing = ui.collapsing(loc!(player_state, "ui_menu_sidebar_develop_section"), |ui| {
                if ui.selectable_label(false, loc!(player_state, "ui_menu_sidebar_scripts_tab")).clicked() {
                    if matches!(player_state.progression, TutorialProgression::DevelopScriptsIntroduced) {
                        player_state.progression = TutorialProgression::ScriptClicked;
                    }

                    ui_state.active_panel = ActivePanel::Scripts;

                    commands.spawn((
                        AudioPlayer::new(asset_server.load("audio/click.ogg")),
                        PlaybackSettings::ONCE
                    ));
                }
            });

            if collapsing.header_response.clicked() {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/click.ogg")),
                    PlaybackSettings::ONCE
                ));
            }
        }
        if player_state.progression.show_exploit_tab() {
            let collapsing = ui.collapsing(loc!(player_state, "ui_menu_sidebar_black_hat_section"), |ui| {
                if ui.selectable_label(false, loc!(player_state, "ui_menu_sidebar_exploit_tab")).clicked() {
                    if matches!(player_state.progression, TutorialProgression::ExploitTabIntroduced) {
                        player_state.progression = TutorialProgression::ExploitServersShown;
                    }

                    ui_state.active_panel = ActivePanel::Exploit;

                    commands.spawn((
                        AudioPlayer::new(asset_server.load("audio/click.ogg")),
                        PlaybackSettings::ONCE
                    ));
                }
            });

            if collapsing.header_response.clicked() {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/click.ogg")),
                    PlaybackSettings::ONCE
                ));
            }
        }

        if player_state.progression.is_complete() {
            ui.label(loc!(player_state, "ui_menu_sidebar_glossary_tab"));
        }
    });

    Ok(())
}

fn on_active_exploit_event(
    evt: On<ExploitEvent>,
    mut ui_state: ResMut<UiState>,
) {
    let Some(window) = ui_state
        .active_exploit_windows
        .iter_mut()
        .find(|window| lock_and_clone!(window.active_exploit, id) == evt.active_exploit_id)
    else {
        return;
    };

    window.event_log.push(evt.event().to_owned());
}

fn on_active_exploit_started(
    evt: On<ExploitStarted>,
    mut ui_state: ResMut<UiState>
) {
    let Some(window) = ui_state
        .active_exploit_windows
        .iter_mut()
        .find(|window| lock_and_clone!(window.active_exploit, id) == evt.exploit_id)
    else {
        return;
    };

    window.event_log.clear();
}