use crate::get_localized;
use bevy::prelude::{On, ResMut};
use bevy_egui::{egui, EguiContexts};
use crate::loc;
use crate::player_state::state::PlayerState;
use crate::script::ScriptCreatedEvent;
use crate::tutorial::progression::TutorialProgression;

pub fn tutorial_on_script_created(
    _: On<ScriptCreatedEvent>,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
    if matches!(player_state.progression, TutorialProgression::ScriptClicked) {
        player_state.progression.advance();
    }

    Ok(())
}

pub fn tutorial_ui_system(
    mut context: EguiContexts,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
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