use bevy_egui::egui;
use crate::l10n::Localizable;
use crate::player_state::state::PlayerState;

pub enum MixedTextNode<'s> {
    Text(String),
    Localizable(&'s dyn Localizable),
    Image(egui::ImageSource<'static>),
}

impl<'s> MixedTextNode<'s> {
    pub fn draw(ui: &mut egui::Ui, nodes: Vec<MixedTextNode>, player_state: &PlayerState) {
        ui.horizontal(|ui| {
            for node in nodes {
                match node {
                    MixedTextNode::Text(text) => {
                        ui.label(text);
                    }
                    MixedTextNode::Localizable(localizable) => {
                        let text = player_state.localize_dyn(localizable);
                        ui.label(text);
                    }
                    MixedTextNode::Image(image) => {
                        ui.add(
                            egui::Image::new(image)
                                .max_size([32.0, 32.0].into())
                                .maintain_aspect_ratio(true)
                        );
                    }
                }
            }
        });
    }
}