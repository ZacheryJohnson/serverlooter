use bevy_egui::egui::text::LayoutJob;
use bevy_egui::egui::{Color32, TextFormat};
use crate::script::Script;
use crate::ui::hover_text::OnHoverText;

impl OnHoverText for Script {
    type State = ();

    fn on_hover_text(&self, _: &Self::State) -> LayoutJob {
        let mut hover_text_layout_job = LayoutJob::default();
        hover_text_layout_job.append(
            // ZJ-TODO: localize
            "Procedure\n",
            0.0,
            TextFormat::default(),
        );

        let mut effect_text_format = TextFormat::default();
        effect_text_format.color = Color32::GOLD;

        let mut thread_id = 'a';
        for procedure in &self.procedures {
            let mut algorithm_id = 1;
            for algorithm in &procedure.algorithms {
                let algorithm = algorithm.lock().unwrap();
                let prefix = format!("{thread_id}.{algorithm_id}:\n");
                hover_text_layout_job.append(
                    // ZJ-TODO: localize
                    &prefix,
                    0.0,
                    TextFormat::default(),
                );
                for (_, effects) in &algorithm.instruction_effects {
                    for effect in effects {
                        hover_text_layout_job.append(
                            // ZJ-TODO: localize
                            &format!("{effect}\n"),
                            10.0,
                            effect_text_format.clone(),
                        );
                    }
                }

                algorithm_id += 1;
            }

            // ZJ-TODO: refactor to be more Rusty
            //          this just "increments" the letter
            //          eg. 'a' + 1 = 'b'
            thread_id = ((thread_id as u8) + 1) as char;
        }

        hover_text_layout_job
    }
}