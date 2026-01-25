mod script;
mod algorithm;

use bevy_egui::egui::text::LayoutJob;

/// When hovered, displays pre-formatted text.
pub trait OnHoverText {
    type State;
    fn on_hover_text(&self, state: &Self::State) -> LayoutJob;
}