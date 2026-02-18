use bevy_egui::egui::text::LayoutJob;
use bevy_egui::egui::{Color32, TextFormat};
use crate::executor::Executor;
use crate::script::Script;
use crate::ui::hover_text::OnHoverText;

impl OnHoverText for Script {
    type State = ();

    fn on_hover_text(&self, _: &Self::State) -> LayoutJob {
        let mut hover_text_layout_job = LayoutJob::default();

        let mut ic_text_format = TextFormat::default();
        ic_text_format.color = Color32::DARK_GRAY;

        hover_text_layout_job.append(
            // ZJ-TODO: localize
            "Procedure ",
            0.0,
            TextFormat::default(),
        );

        hover_text_layout_job.append(
            &format!("(IC {})\n", self.executor().total_instructions()),
            0.0,
            ic_text_format.clone(),
        );

        let mut effect_text_format = TextFormat::default();
        effect_text_format.color = Color32::GOLD;

        let mut thread_id = 'a';
        for procedure in &self.procedures {
            let mut algorithm_id = 1;
            for algorithm in procedure.algorithms() {
                let algorithm = algorithm.lock().unwrap();
                hover_text_layout_job.append(
                    &format!("{thread_id}.{algorithm_id}: "),
                    0.0,
                    TextFormat::default(),
                );

                hover_text_layout_job.append(
                    &format!("(IC {})\n", algorithm.instruction_count),
                    0.0,
                    ic_text_format.clone(),
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

            // The last effect has an extra trailing newline
            hover_text_layout_job.text.truncate(hover_text_layout_job.text.len() - 1);

            // ZJ-TODO: refactor to be more Rusty
            //          this just "increments" the letter
            //          eg. 'a' + 1 = 'b'
            thread_id = ((thread_id as u8) + 1) as char;
        }

        hover_text_layout_job
    }
}