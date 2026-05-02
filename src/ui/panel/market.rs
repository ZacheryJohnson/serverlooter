use bevy::asset::AssetServer;
use bevy::prelude::Commands;
use bevy_egui::egui;
use bevy_egui::egui::{Context, Ui};
use crate::event::request_purchase_unlock::RequestPurchaseUnlockEvent;
use crate::{loc, PlayerState, PlayerUnlock};
use crate::event::tutorial_data_dump_purchased::TutorialDataDumpPurchased;
use crate::l10n::message_id::MessageId;
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
        player_state: &PlayerState,
        _: &AssetServer,
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

        for (unlock, credit_cost) in PlayerUnlock::market_unlockable_unlocks() {
            let unlock_owned = player_state.player_unlocks.is_unlocked(unlock);

            let unlock_button_text = format!(
                "{}\n{}",
                player_state.localize(&unlock),
                loc!(player_state, MessageId::MarketUnlockCreditCost, [("credit_cost", credit_cost.into())].into()),
            );

            let unlock_button = egui::Button::new(unlock_button_text);

            let unlock_ui_response = ui
                .add_enabled(!unlock_owned, unlock_button)
                .on_hover_text(player_state.localize(&unlock.description()))
                .on_disabled_hover_text(loc!(player_state, MessageId::MarketUnlockAlreadyUnlocked));

            if unlock_ui_response.clicked() {
                commands.trigger(RequestPurchaseUnlockEvent {
                    unlock,
                    credit_cost,
                });
            }
        }
    }
}