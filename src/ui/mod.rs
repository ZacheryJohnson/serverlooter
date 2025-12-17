use std::collections::HashMap;
use crate::get_localized;
use bevy_egui::egui::{Sense, Ui};
use crate::{loc, PlayerState};

/// Any state that can be drawn to the main panel
pub trait Panel {
    fn update(
        &mut self,
        ui: &mut Ui,
        player_state: &PlayerState
    );
}

pub struct MarketPanel {

}

impl Panel for MarketPanel {
    fn update(&mut self, ui: &mut Ui, player_state: &PlayerState) {
        ui.label("market test");
    }
}

pub struct ServersPanel {

}

impl Panel for ServersPanel {
    fn update(&mut self, ui: &mut Ui, player_state: &PlayerState) {
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

            if grouping.response.interact(Sense::click()).clicked() {
                println!("grouping");
            }
        }
    }
}

pub struct ScriptsPanel {

}

impl Panel for ScriptsPanel {
    fn update(&mut self, ui: &mut Ui, player_state: &PlayerState) {
        todo!()
    }
}