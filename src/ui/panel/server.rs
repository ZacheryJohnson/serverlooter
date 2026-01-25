use bevy::prelude::Commands;
use bevy_egui::egui::{Context, Ui};
use crate::{get_localized, loc, lock_and_clone, ActiveExploit, PlayerState};
use crate::macros::clock_speed_to_loc_args;
use crate::ui::panel::Panel;

pub struct ServersPanel {

}

impl Panel for ServersPanel {
    fn update(
        &mut self,
        _: &mut Commands,
        _: &Context,
        ui: &mut Ui,
        player_state: &PlayerState
    ) {
        for server_arc in &player_state.servers {
            let server = server_arc.lock().unwrap().clone();
            ui.group(|group_ui| {
                group_ui.vertical_centered(|vert_ui| {
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