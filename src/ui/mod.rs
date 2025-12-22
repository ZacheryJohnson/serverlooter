use std::collections::HashMap;
use bevy::prelude::Commands;
use crate::get_localized;
use bevy_egui::egui::{Color32, Context, RichText, Sense, Ui};
use crate::{loc, PlayerState};
use crate::inventory::{InventoryItem, InventoryItemAdded, InventoryItemRemoved};
use crate::script::{Algorithm, ScriptBuilder, ScriptCreatedEvent};

/// Any state that can be drawn to the main panel
pub trait Panel {
    fn update(
        &mut self,
        commands: &mut Commands,
        ctx: &Context,
        ui: &mut Ui,
        player_state: &PlayerState
    );
}

pub struct MarketPanel {

}

impl Panel for MarketPanel {
    fn update(&mut self, commands: &mut Commands, ctx: &Context, ui: &mut Ui, player_state: &PlayerState) {
        ui.label("market test");
    }
}

pub struct ServersPanel {

}

impl Panel for ServersPanel {
    fn update(&mut self, commands: &mut Commands, ctx: &Context, ui: &mut Ui, player_state: &PlayerState) {
        for server_arc in &player_state.servers {
            let server = server_arc.lock().unwrap();
            let grouping = ui.group(|group_ui| {
                let vert = group_ui.vertical_centered(|vert_ui| {
                    vert_ui.heading(&server.name);
                    vert_ui.label(loc!(
                        player_state,
                        "ui_server_thread_count",
                        HashMap::from([("thread_count".to_string(), server.threads.into())])
                    ));
                    vert_ui.label(loc!(
                        player_state,
                        "ui_server_clock_speed_ghz",
                        HashMap::from([("clock_speed_ghz".to_string(), (server.clock_speed_hz as f32 / 1_000_000_000.0).into())])
                    ));
                });
            });
        }
    }
}

pub struct ScriptsPanel {
    script_builder: ScriptBuilder,
}

impl ScriptsPanel {
    pub fn new() -> Self {
        ScriptsPanel {
            script_builder: ScriptBuilder::new(),
        }
    }
}

impl Panel for ScriptsPanel {
    fn update(&mut self, commands: &mut Commands, ctx: &Context, ui: &mut Ui, player_state: &PlayerState) {
        bevy_egui::egui::SidePanel::left("scripts_menu").show(ctx, |ui| {
            ui.collapsing(loc!(player_state, "ui_algorithm_scripts_header"), |ui| {
                for script in &player_state.scripts {
                    let script = script.lock().unwrap();

                    // ZJ-TODO: localize
                    // ZJ-TODO: this logic breaks if procedures can merge
                    ui.label(format!("Required Threads: {}", script.procedures.len()));
                    for procedure in &script.procedures {
                        for algorithm in &procedure.algorithms {
                            for (_, effects) in &algorithm.instruction_effects {
                                for effect in effects {
                                    // ZJ-TODO: localize
                                    ui.label(format!("{effect:?}"));
                                }
                            }
                        }
                    }
                }
            });

            ui.collapsing(loc!(player_state, "ui_algorithm_algorithms_header"), |ui| {
                for algorithm in &player_state.inventory.algorithms {
                    ui.scope(|ui| {
                        if ui.response().hovered() || ui.response().dragged() {
                            ui.style_mut().visuals.widgets.noninteractive.bg_stroke.color = Color32::YELLOW;
                        }

                        let group = ui.group(|ui| {
                            let style = ui.style_mut();
                            style.interaction.selectable_labels = false;

                            ui.label(loc!(
                                player_state,
                                "ui_algorithm_instruction_count",
                                HashMap::from([("instruction_count".to_string(), algorithm.instruction_count.into())])
                            ));

                            ui.label(loc!(player_state, "ui_algorithm_effects_header"));
                            for (_, effects) in &algorithm.instruction_effects {
                                for effect in effects {
                                    // ZJ-TODO: localize
                                    ui.label(format!("{effect:?}"));
                                }
                            }
                        });

                        if group.response.interact(Sense::click()).clicked() {
                            self.script_builder.add_algorithm(algorithm.clone());
                            commands.trigger(InventoryItemRemoved {
                                item: InventoryItem::Algorithm(algorithm.clone()),
                            })
                        }
                    });
                }
            });
        });

        bevy_egui::egui::CentralPanel::default().show(ctx, |ui| {
            let script = self.script_builder.current_script();

            let mut algorithm_to_remove: Option<Algorithm> = None;
            for procedure in &script.procedures {
                ui.heading(loc!(player_state, "ui_algorithm_procedure_header"));
                let mut procedure = procedure.clone();
                while let Some(algorithm) = procedure.algorithms.pop_back() {
                    let group = ui.group(|ui| {
                        ui.label(loc!(
                            player_state,
                            "ui_algorithm_instruction_count",
                            HashMap::from([("instruction_count".to_string(), algorithm.instruction_count.into())])
                        ));

                        ui.label(loc!(player_state, "ui_algorithm_effects_header"));
                        for (_, effects) in &algorithm.instruction_effects {
                            for effect in effects {
                                // ZJ-TODO: localize
                                ui.label(RichText::new(format!("{effect:?}")).color(Color32::GOLD));
                            }
                        }
                    });

                    if group.response.interact(Sense::click()).clicked() {
                        algorithm_to_remove = Some(algorithm);
                    }
                };
            }

            if let Some(algorithm) = algorithm_to_remove {
                self.script_builder.remove_algorithm(algorithm.clone());
                commands.trigger(InventoryItemAdded {
                    item: InventoryItem::Algorithm(algorithm),
                });
            }

            // ZJ-TODO: localize
            if !self.script_builder.is_empty() && ui.button(loc!(player_state, "ui_confirmation_create")).clicked() {
                let mut script_builder = ScriptBuilder::new();
                std::mem::swap(&mut self.script_builder, &mut script_builder);
                let script = script_builder.finish();

                commands.trigger(ScriptCreatedEvent {
                    script
                });
            }
        });
    }
}