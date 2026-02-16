use bevy::app::{App, Plugin};
use bevy_egui::EguiPrimaryContextPass;
use crate::tutorial::systems::*;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(tutorial_on_script_created)
            .add_observer(on_tutorial_data_dump_purchased)
            .add_systems(EguiPrimaryContextPass, tutorial_ui_system);
    }
}