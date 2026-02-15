use bevy::prelude::Commands;
use bevy_egui::egui::{Context, Ui};
use crate::PlayerState;

pub mod active_exploit;

/// Any state that can be drawn as a floating window
pub trait Window {
    fn update(
        &mut self,
        commands: &mut Commands,
        ctx: &Context,
        player_state: &PlayerState
    );
}