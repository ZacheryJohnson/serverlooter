use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bevy::prelude::Commands;
use bevy_egui::egui;
use bevy_egui::egui::{Align2, Color32, Context, FontId, RichText, Sense, StrokeKind, Ui};
use crate::{get_localized, loc, PlayerState};
use crate::algorithm::algorithm::Algorithm;
use crate::inventory::{InventoryItem, InventoryItemAdded, InventoryItemRemoved};
use crate::script::{ScriptBuilder, ScriptCreatedEvent, ScriptId};
use crate::ui::inventory::grid_item::{AlgorithmGridItem, InventoryGridItem, InventoryGridItemDisplay, ScriptGridItem};
use crate::ui::panel::Panel;

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
    fn update(
        &mut self,
        commands: &mut Commands,
        ctx: &Context,
        _: &mut Ui,
        player_state: &PlayerState
    ) {
        egui::SidePanel::left("scripts_menu").show(ctx, |ui| {

            egui::CollapsingHeader::new(loc!(player_state, "ui_algorithm_scripts_header"))
                .default_open(true)
                .show_unindented(ui, |ui| {
                    let grid_items = player_state.scripts.iter().map(|script| {
                        ScriptGridItem::from(Arc::downgrade(script))
                    }).collect::<Vec<_>>();

                    egui::Grid::new("script_panel_script_item_grid")
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.ctx().style_mut(|style| {
                                style.interaction.tooltip_delay = 0.0;
                                style.interaction.tooltip_grace_time = 0.0;
                                style.interaction.show_tooltips_only_when_still = false;
                            });

                            for grid_item in grid_items {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(35.0, 35.0),
                                    Sense::click() | Sense::hover() | Sense::drag(),
                                );

                                let resp = resp.on_hover_ui(|ui| {
                                    ui.label(grid_item.hover_text().to_owned());
                                });

                                let visuals = ui.style().interact(&resp);

                                ui
                                    .painter()
                                    .rect(rect, 5.0, visuals.bg_fill, visuals.fg_stroke, StrokeKind::Outside);

                                match grid_item.display() {
                                    InventoryGridItemDisplay::Text(display_text) => {
                                        ui
                                            .painter()
                                            .text(rect.center(), Align2::CENTER_CENTER, display_text, FontId::default(), Color32::GOLD);
                                    }
                                }
                            }
                        });
            });

            egui::CollapsingHeader::new(loc!(player_state, "ui_algorithm_algorithms_header"))
                .default_open(true)
                .show_unindented(ui, |ui| {
                    let grid_items = player_state.inventory.algorithms.iter().map(|algo| {
                        AlgorithmGridItem::from(Arc::downgrade(algo), player_state)
                    }).collect::<Vec<_>>();

                    egui::Grid::new("script_panel_algorithm_item_grid")
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.ctx().style_mut(|style| {
                                style.interaction.tooltip_delay = 0.0;
                                style.interaction.tooltip_grace_time = 0.0;
                                style.interaction.show_tooltips_only_when_still = false;
                            });

                            for (idx, grid_item) in grid_items.iter().enumerate() {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(35.0, 35.0),
                                    Sense::click() | Sense::hover() | Sense::drag(),
                                );

                                let resp = resp.on_hover_ui(|ui| {
                                    ui.label(grid_item.hover_text().to_owned());
                                });

                                if resp.clicked() {
                                    let algorithm = grid_item.algorithm.upgrade().unwrap();
                                    self.script_builder.add_algorithm(algorithm.clone());
                                    commands.trigger(InventoryItemRemoved {
                                        item: InventoryItem::Algorithm(algorithm),
                                    })
                                }

                                let visuals = ui.style().interact(&resp);

                                ui
                                    .painter()
                                    .rect(rect, 5.0, visuals.bg_fill, visuals.fg_stroke, StrokeKind::Outside);

                                match grid_item.display() {
                                    InventoryGridItemDisplay::Text(display_text) => {
                                        ui
                                            .painter()
                                            .text(rect.center(), Align2::CENTER_CENTER, display_text, FontId::default(), Color32::GOLD);
                                    }
                                }

                                const MAX_ENTRIES_PER_ROW: usize = 5;
                                if (idx + 1) % MAX_ENTRIES_PER_ROW == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let script = self.script_builder.current_script();

            let mut algorithm_to_remove: Option<Arc<Mutex<Algorithm>>> = None;
            for procedure in &script.procedures {
                ui.heading(loc!(player_state, "ui_algorithm_procedure_header"));
                let mut algorithms = procedure.algorithms();
                while let Some(algorithm) = algorithms.next() {
                    let algorithm_inner = algorithm.lock().unwrap();
                    let group = ui.group(|ui| {
                        ui.label(loc!(
                            player_state,
                            "ui_algorithm_instruction_count",
                            HashMap::from([("instruction_count".to_string(), algorithm_inner.instruction_count.into())])
                        ));

                        ui.label(loc!(player_state, "ui_algorithm_effects_header"));
                        for (_, effects) in &algorithm_inner.instruction_effects {
                            for effect in effects {
                                // ZJ-TODO: localize
                                ui.label(RichText::new(format!("{effect}")).color(Color32::GOLD));
                            }
                        }
                    });

                    if group.response.interact(Sense::click()).clicked() {
                        algorithm_to_remove = Some(algorithm.clone());
                    }
                };
            }

            if let Some(algorithm) = algorithm_to_remove {
                self.script_builder.remove_algorithm(algorithm.clone());
                commands.trigger(InventoryItemAdded {
                    item: InventoryItem::Algorithm(algorithm),
                });
            }

            if !self.script_builder.is_empty() && ui.button(loc!(player_state, "ui_confirmation_create")).clicked() {
                let mut script_builder = ScriptBuilder::new();
                std::mem::swap(&mut self.script_builder, &mut script_builder);
                let mut script = script_builder.finish();
                script.id = ScriptId::Id(player_state.scripts.iter().count() as u64 + 1); // ZJ-TODO

                commands.trigger(ScriptCreatedEvent {
                    script
                });
            }
        });
    }
}