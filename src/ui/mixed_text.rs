use bevy_egui::egui;

pub enum MixedTextNode {
    Text(String),
    Image(egui::ImageSource<'static>),
}

impl MixedTextNode {
    pub fn draw(ui: &mut egui::Ui, nodes: Vec<MixedTextNode>) {
        ui.horizontal(|ui| {
            for node in nodes {
                match node {
                    MixedTextNode::Text(text) => {
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