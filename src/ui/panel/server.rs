use bevy::asset::AssetServer;
use bevy::prelude::Commands;
use bevy_egui::egui;
use bevy_egui::egui::{Context, Ui};
use crate::{loc, lock_and_clone, PlayerState};
use crate::l10n::message_id::MessageId;
use crate::ui::clock_speed::ClockSpeed;
use crate::ui::panel::Panel;

pub struct ServersPanel {

}

impl Panel for ServersPanel {
    fn update(
        &mut self,
        _: &mut Commands,
        _: &Context,
        ui: &mut Ui,
        player_state: &PlayerState,
        _: &AssetServer,
    ) {
        for server_arc in &player_state.servers {
            let server = server_arc.lock().unwrap().clone();
            ui.group(|group_ui| {
                group_ui.vertical_centered(|vert_ui| {
                    vert_ui.heading(&server.name);

                    vert_ui.label(player_state.localize(&server.clock_speed));

                    vert_ui.label(loc!(
                        player_state,
                        MessageId::UiServerThreadCount,
                        [("thread_count", server.threads.into())].into()
                    ));

                    let active_exploits_on_this_server: Vec<_> = player_state
                        .active_exploits
                        .iter()
                        .filter(|exploit| {
                            lock_and_clone!(exploit, hosting_server, name) == server.name
                        })
                        .collect();

                    if !active_exploits_on_this_server.is_empty() {
                        vert_ui.heading("Active Exploits");
                    }

                    for exploit in active_exploits_on_this_server {
                        let hosting_server = lock_and_clone!(exploit, hosting_server);
                        let target_server = lock_and_clone!(exploit, target, server);
                        let target_name = lock_and_clone!(target_server, name);

                        vert_ui.horizontal(|horiz_ui| {
                            let mut clock_allocation = lock_and_clone!(exploit, clock_allocation);

                            horiz_ui.label(target_name);

                            let mut slider = egui::Slider::new(
                                &mut *clock_allocation,
                                // ZJ-TODO: this should be available CPU, not total clock speed
                                0 ..= *lock_and_clone!(hosting_server, clock_speed)
                            );

                            slider = slider.custom_formatter(|val, _| {
                                player_state.localize(&ClockSpeed::new(val.round() as u64))
                            });

                            horiz_ui.add(slider);

                            exploit.lock().unwrap().clock_allocation = clock_allocation;
                        });
                    }
                });
            });
        }
    }
}