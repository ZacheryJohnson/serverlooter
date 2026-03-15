use bevy::prelude::{On, ResMut};
use bevy_egui::{egui, EguiContexts};
use crate::event::tutorial_data_dump_purchased::TutorialDataDumpPurchased;
use crate::l10n::message_id::MessageId;
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
    let window = egui::Window::new(loc!(player_state, MessageId::UiWindowTutorialTitle))
        .default_pos(egui::pos2(
            context.ctx_mut()?.content_rect().width() / 2.0,
            context.ctx_mut()?.content_rect().height() / 2.0,
        ))
        .pivot(egui::Align2::CENTER_CENTER)
        .resizable(true)
        .constrain(true);

    match player_state.progression {
        TutorialProgression::Skipped | TutorialProgression::Complete => { /* no-op */ },
        TutorialProgression::None => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialAskStart));
                if ui.button(loc!(player_state, MessageId::UiConfirmationYes)).clicked() {
                    player_state.progression = TutorialProgression::Start;
                }
                if ui.button(loc!(player_state, MessageId::UiConfirmationNo)).clicked() {
                    player_state.progression = TutorialProgression::Skipped;
                }
            });
        }
        TutorialProgression::Start => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage1));
                if ui.button(loc!(player_state, MessageId::UiConfirmationNext)).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::ServersTabIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage2));
            });
        }
        TutorialProgression::ServerClicked => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage3));
                if ui.button(loc!(player_state, MessageId::UiConfirmationNext)).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        TutorialProgression::DevelopScriptsIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage4));
            });
        }
        TutorialProgression::ScriptClicked => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage5));
            });
        }
        TutorialProgression::MarketTabIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage6));
            });
        }
        TutorialProgression::MarketTabClicked => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage7));
            });
        }
        TutorialProgression::ExploitTabIntroduced => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage8));
            });
        }
        TutorialProgression::ExploitServersShown => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage9));
            });
        }
        TutorialProgression::ExploitCreditsReceived => {
            window.show(context.ctx_mut()?, |ui| {
                ui.label(loc!(player_state, MessageId::TutorialStage10));

                if ui.button(loc!(player_state, MessageId::UiConfirmationNext)).clicked() {
                    player_state.progression.advance();
                }
            });
        }
        // ZJ-TODO: finish tutorial
        // TutorialProgression::ExploitCorpASuccess => {
        //     window.show(context.ctx_mut()?, |ui| {
        //         ui.label(loc!(player_state, "tutorial_stage_11"));
        //
        //         if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
        //             player_state.progression.advance();
        //         }
        //     });
        // }
        // TutorialProgression::ExploitCorpBClicked => {
        //     window.show(context.ctx_mut()?, |ui| {
        //         ui.label(loc!(player_state, "tutorial_stage_12"));
        //
        //         if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
        //             player_state.progression.advance();
        //         }
        //     });
        // }
        // TutorialProgression::MarketAlgorithmPrompted => {
        //     window.show(context.ctx_mut()?, |ui| {
        //         ui.label(loc!(player_state, "tutorial_stage_13"));
        //
        //         if ui.button(loc!(player_state, "ui_confirmation_next")).clicked() {
        //             player_state.progression.advance();
        //         }
        //     });
        // }
    }

    Ok(())
}

pub fn on_tutorial_data_dump_purchased(
    evt: On<TutorialDataDumpPurchased>,
    mut player_state: ResMut<PlayerState>,
) {
    if matches!(player_state.progression, TutorialProgression::MarketTabClicked) {
        player_state.credits -= evt.credit_cost as u128;
        player_state.progression.advance();
    }
}