use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bevy::prelude::{Commands, Event};
use bevy_egui::egui;
use crate::{get_localized, lock_and_clone, ActiveExploit, ExploitTarget};
use bevy_egui::egui::{Color32, Context, RichText, Sense, Ui, Widget};
use uuid::Uuid;
use crate::{loc, PlayerState};
use crate::inventory::{InventoryItem, InventoryItemAdded, InventoryItemRemoved};
use crate::macros::clock_speed_to_loc_args;
use crate::script::{Algorithm, Script, ScriptBuilder, ScriptCreatedEvent, ScriptId};
use crate::server::Server;

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
            let server = server_arc.lock().unwrap().clone();
            let grouping = ui.group(|group_ui| {
                let vert = group_ui.vertical_centered(|vert_ui| {
                    vert_ui.heading(&server.name);

                    vert_ui.label(loc!(
                        player_state,
                        "ui_server_clock_speed",
                        clock_speed_to_loc_args(server.clock_speed_hz)
                    ));

                    vert_ui.label(loc!(
                        player_state,
                        "ui_server_thread_count",
                        [("thread_count".to_string(), server.threads.into())].into()
                    ));

                    let active_exploits_on_this_server: Vec<&ActiveExploit> = player_state
                        .active_exploits
                        .iter()
                        .filter(|exploit| exploit.hosting_server.lock().unwrap().name == server.name)
                        .collect();

                    if !active_exploits_on_this_server.is_empty() {
                        vert_ui.heading("Active Exploits");
                    }

                    for exploit in active_exploits_on_this_server {
                        let target_name = lock_and_clone!(exploit.target, server, name);
                        vert_ui.label(format!(
                            "{} at {}",
                            target_name,
                            loc!(
                                player_state,
                                "ui_server_clock_speed",
                                clock_speed_to_loc_args(exploit.clock_allocation_hz)
                            )
                        ));
                    }
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
        egui::SidePanel::left("scripts_menu").show(ctx, |ui| {
            ui.collapsing(loc!(player_state, "ui_algorithm_scripts_header"), |ui| {
                for script in &player_state.scripts {
                    let script = script.lock().unwrap();

                    // ZJ-TODO: localize
                    // ZJ-TODO: this logic breaks if procedures can merge
                    ui.group(|ui| {
                        ui.label(format!("Required Threads: {}", script.procedures.len()));
                        for procedure in &script.procedures {
                            for algorithm in &procedure.algorithms {
                                for (_, effects) in &algorithm.instruction_effects {
                                    for effect in effects {
                                        // ZJ-TODO: localize
                                        ui.label(format!("{effect}"));
                                    }
                                }
                            }
                        }
                    });
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
                                    ui.label(format!("{effect}"));
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

        egui::CentralPanel::default().show(ctx, |ui| {
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
                                ui.label(RichText::new(format!("{effect}")).color(Color32::GOLD));
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

            if !self.script_builder.is_empty() && ui.button(loc!(player_state, "ui_confirmation_create")).clicked() {
                let mut script_builder = ScriptBuilder::new();
                std::mem::swap(&mut self.script_builder, &mut script_builder);
                let mut script = script_builder.finish();
                script.id = ScriptId::Id(1); // ZJ-TODO

                commands.trigger(ScriptCreatedEvent {
                    script
                });
            }
        });
    }
}

#[derive(Event)]
pub struct RequestStartExploitEvent {
    pub target: Arc<Mutex<ExploitTarget>>,
    pub script: Arc<Mutex<Script>>,
    pub server: Arc<Mutex<Server>>,
}

#[derive(Event)]
pub struct RequestStopExploitEvent {
    pub exploit_id: Uuid,
}

pub struct ExploitPanel {
    pub selected_exploit_target: Option<Arc<Mutex<ExploitTarget>>>,
    pub selected_script: Option<Arc<Mutex<Script>>>,
    pub selected_server: Option<Arc<Mutex<Server>>>,
}

impl ExploitPanel {
    pub fn new() -> Self {
        ExploitPanel {
            selected_exploit_target: None,
            selected_script: None,
            selected_server: None,
        }
    }
}

impl Panel for ExploitPanel {
    fn update(&mut self, commands: &mut Commands, ctx: &Context, ui: &mut Ui, player_state: &PlayerState) {
        ui.heading("Targets");
        ui.horizontal(|ui| {
           for exploit_target in &player_state.known_targets {
               let exploit_target_id = exploit_target.lock().unwrap().id;
               let is_selected = match self.selected_exploit_target {
                   Some(ref target) => target.lock().unwrap().id == exploit_target_id,
                   None => false,
               };
               if ui.selectable_label(is_selected, format!("{}", lock_and_clone!(exploit_target, server, name))).clicked() {
                    self.selected_exploit_target = Some(exploit_target.to_owned());
               }
           }
        });

        ui.separator();
        ui.heading("Script");
        let empty_script = Arc::new(Mutex::new(Script::empty()));
        let selected_script = self.selected_script.as_ref().unwrap_or(&empty_script);
        let selected_script = selected_script.clone();
        egui::ComboBox::from_id_salt("scripts")
            .selected_text(format!("{}", selected_script.lock().unwrap().id))
            .show_ui(ui, |ui| {
                let mut selected_script_id = selected_script.lock().unwrap().id.clone();
                for script in &player_state.scripts {
                    let script_id = script.lock().unwrap().id.clone();
                    ui.selectable_value(
                        &mut selected_script_id,
                        script_id.clone(),
                        format!("{script_id}")
                    );

                    if selected_script_id == script_id {
                        self.selected_script = Some(script.clone());
                    }
                }
            });

        // ZJ-TODO: server selection
        ui.separator();
        ui.heading("Server");
        let empty_server = Arc::new(Mutex::new(Server::empty()));
        let selected_server = self.selected_server.as_ref().unwrap_or(&empty_server);
        let selected_server = selected_server.clone();
        egui::ComboBox::from_id_salt("servers")
            .selected_text(format!("{}", selected_server.lock().unwrap().name))
            .show_ui(ui, |ui| {
                let mut selected_server_name = selected_server.lock().unwrap().name.clone();
                for server in &player_state.servers {
                    let server_name = server.lock().unwrap().name.clone();
                    ui.selectable_value(
                        &mut selected_server_name,
                        server_name.clone(),
                        format!("{server_name}")
                    );

                    if selected_server_name == server_name {
                        self.selected_server = Some(server.clone());
                    }
                }
            });

        ui.separator();
        let required_fields_set = self.selected_exploit_target.is_some() && self.selected_script.is_some();
        if ui.add_enabled(required_fields_set, egui::Button::new("Run")).clicked() {
            commands.trigger(RequestStartExploitEvent {
                target: self.selected_exploit_target.as_ref().unwrap().clone(),
                script: self.selected_script.as_ref().unwrap().clone(),
                server: self.selected_server.as_ref().unwrap().clone(),
            });

            self.selected_exploit_target = None;
            self.selected_script = None;
            self.selected_server = None;
        }
    }
}