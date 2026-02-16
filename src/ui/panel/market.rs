use bevy::prelude::Commands;
use bevy_egui::egui;
use bevy_egui::egui::{Context, Ui};
use crate::event::request_purchase_unlock::RequestPurchaseUnlockEvent;
use crate::{PlayerState, PlayerUnlock};
use crate::event::tutorial_data_dump_purchased::TutorialDataDumpPurchased;
use crate::tutorial::progression::TutorialProgression;
use crate::ui::panel::Panel;

pub struct MarketPanel {

}

impl Panel for MarketPanel {
    fn update(
        &mut self,
        commands: &mut Commands,
        _: &Context,
        ui: &mut Ui,
        player_state: &PlayerState
    ) {
        if matches!(player_state.progression, TutorialProgression::MarketTabClicked) {
            ui.heading("Data Dumps");
            ui.separator();
            let credit_cost = 25;
            if ui.button(format!("Tutorial Data Dump\n{credit_cost} credits")).clicked() {
                commands.trigger(TutorialDataDumpPurchased {
                    credit_cost,
                });
            }
        }

        ui.heading("Unlocks");
        ui.separator();

        // ZJ-TODO: don't use newlines, make custom button wrapper
        let has_unlock = player_state.player_unlocks.exploit_auto_reconnect;
        let credit_cost = 150;
        let exploit_auto_reconnect_purchase = egui::Button::new(format!("Exploit Auto-reconnect\n{credit_cost} credits"));

        let exploit_auto_reconnect_purchase = ui
            .add_enabled(!has_unlock, exploit_auto_reconnect_purchase)
            .on_hover_text("Automatically reconnect to a target server during exploits once disconnected.")
            .on_disabled_hover_text("Already unlocked");

        if exploit_auto_reconnect_purchase.clicked() {
            commands.trigger(RequestPurchaseUnlockEvent {
                unlock: PlayerUnlock::ExploitAutoReconnect,
                credit_cost,
            });
        }
    }
}