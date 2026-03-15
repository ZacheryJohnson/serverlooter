use bevy_egui::egui::text::LayoutJob;
use bevy_egui::egui::{Color32, TextFormat};
use crate::{loc, PlayerState};
use crate::algorithm::algorithm::Algorithm;
use crate::l10n::message_id::MessageId;
use crate::ui::hover_text::OnHoverText;

impl OnHoverText for Algorithm {
    type State = PlayerState;

    fn on_hover_text(&self, state: &Self::State) -> LayoutJob {
        let mut hover_text_layout_job = LayoutJob::default();
        hover_text_layout_job.append(
            &state.localize(&self.instruction_count),
            0.0,
            TextFormat::default(),
        );
        hover_text_layout_job.append("\n", 0.0, TextFormat::default());
        hover_text_layout_job.append(
            &loc!(state, MessageId::UiAlgorithmEffectsHeader),
            10.0,
            TextFormat::default(),
        );
        hover_text_layout_job.append("\n", 0.0, TextFormat::default());

        let mut effect_text_format = TextFormat::default();
        effect_text_format.color = Color32::GOLD;
        for (_, effects) in &self.instruction_effects {
            for effect in effects {
                hover_text_layout_job.append(
                    &state.localize(effect),
                    10.0,
                    effect_text_format.clone(),
                );

                hover_text_layout_job.append("\n", 10.0, TextFormat::default());
            }
        }
        
        // The last effect has an extra trailing newline
        hover_text_layout_job.text.truncate(hover_text_layout_job.text.len() - 1);

        hover_text_layout_job
    }
}