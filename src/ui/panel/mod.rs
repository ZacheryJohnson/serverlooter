use bevy::prelude::Commands;
use bevy_egui::egui::{Context, Ui};
use crate::player_state::state::PlayerState;

pub mod market;
pub mod server;
pub mod script;
pub mod exploit;

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