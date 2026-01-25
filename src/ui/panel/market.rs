use bevy::prelude::Commands;
use bevy_egui::egui::{Context, Ui};
use crate::PlayerState;
use crate::ui::panel::Panel;

pub struct MarketPanel {

}

impl Panel for MarketPanel {
    fn update(
        &mut self,
        _: &mut Commands,
        _: &Context,
        ui: &mut Ui,
        _: &PlayerState
    ) {
        ui.label("market test");
    }
}